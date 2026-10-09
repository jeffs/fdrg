#![doc = include_str!("../README.md")]

// # TODO
//
// - Never run FDClone (which is installed on Debian as `/usr/bin/fd`). We could
//   check for `fdfind` before `fd`, but that would still hose any user who had
//   FDClone installed but not `fd-find`.
//
// - Support setting FD_NAME and RG_NAME from config or env vars.

use std::{
    env,
    path::PathBuf,
    process::{Command, ExitCode},
    sync::LazyLock,
};

pub mod error;

use error::{CommandNotFound, CommandStatus, Error};

/// Names and paths to try for fd-find, per `$FDRG_FD_PATH` or defaults.
pub static FD_NAMES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    // Debian install FDClone as `/usr/bin/fd`, and fd as `fdfind`.
    #[cfg(target_os = "linux")]
    const DEFAULT: [&str; 2] = ["fdfind", "fd"];
    #[cfg(not(target_os = "linux"))]
    const DEFAULT: [&str; 2] = ["fd", "fdfind"];
    env::var_os("FDRG_FD_PATH")
        .map(|s| env::split_paths(&s).collect())
        .unwrap_or_else(|| DEFAULT.map(<&str>::into).to_vec())
});

/// Names and paths to try for ripgrep, per `$FDRG_RG_PATH` or defaults.
pub static RG_NAMES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    env::var_os("FDRG_RG_PATH")
        .map(|s| env::split_paths(&s).collect())
        .unwrap_or_else(|| vec!["rg".into()])
});

/// Command option.
pub enum Flag {
    CaseSensitive,
    FilesWithMatches,
    FixedStrings,
    Unrestricted,
}

/// Command arguments.
pub struct Args {
    /// Options to fd-find and ripgrep; see [`Flag::as_fd`] and [`Flag::as_rg`].
    pub flags: Vec<Flag>,
    /// Search pattern and paths passed to both fd-find and ripgrep.
    pub targets: Vec<String>,
}

impl Flag {
    /// Parses one character from a flag set. `-` itself (as in the traditional
    /// "end of flags" argument `--`) is not recognized as a flag character.
    pub fn short(c: char) -> Option<Self> {
        Some(match c {
            'l' => Flag::FilesWithMatches,
            'F' => Flag::FixedStrings,
            's' => Flag::CaseSensitive,
            'u' => Flag::Unrestricted,
            _ => return None,
        })
    }

    /// Matches the long flag name **without** the leading `--`. The empty
    /// string is not a valid flag name.
    pub fn long(s: &str) -> Option<Self> {
        Some(match s {
            "files-with-matches" => Flag::FilesWithMatches,
            "fixed-strings" => Flag::FixedStrings,
            "ignore-case" => Flag::CaseSensitive,
            "unrestricted" => Flag::Unrestricted,
            _ => return None,
        })
    }

    /// Returns an argument to be passed to the `fd(1)` CLI.
    pub fn as_fd(&self) -> Option<&'static str> {
        Some(match self {
            Self::CaseSensitive => "--case-sensitive",
            Self::FilesWithMatches => return None,
            Self::FixedStrings => "--fixed-strings",
            Self::Unrestricted => "--unrestricted",
        })
    }

    /// Returns an argument to be passed to the `rg(1)` CLI.
    pub fn as_rg(&self) -> Option<&'static str> {
        match self {
            Self::FilesWithMatches => Some("--files-with-matches"),
            // The other rg flags we currently support all match fd.
            _ => self.as_fd(),
        }
    }
}

impl Args {
    pub fn from_env() -> Result<Self, Error> {
        let mut flags = Vec::new();
        let mut targets = Vec::new();
        let mut argv = env::args().skip(1);
        for arg in argv.by_ref() {
            if arg == "--" {
                break;
            } else if let Some(flag_name) = arg.strip_prefix("--") {
                flags.push(Flag::long(flag_name).ok_or(Error::Flag(arg))?);
            } else if let Some(flag_chars) = arg.strip_prefix('-') {
                for c in flag_chars.chars() {
                    flags.push(Flag::short(c).ok_or(Error::FlagChar(c))?);
                }
            } else {
                targets.push(arg);
            }
        }
        targets.extend(argv);
        Ok(Args { flags, targets })
    }
}

/// Spawns fd-find, blocks on its completion, and returns its status.
pub fn fd(args: &Args) -> Result<CommandStatus, CommandNotFound<'_>> {
    let flags: Vec<_> = args.flags.iter().flat_map(Flag::as_fd).collect();
    for name in &*FD_NAMES {
        if let Ok(status) = Command::new(name).args(&flags).args(&args.targets).status() {
            let ok = status.success();
            return Ok(CommandStatus { name, status, ok });
        }
    }
    Err(CommandNotFound(&FD_NAMES))
}

/// Invokes ripgrep, blocks on its completion, and returns its status.
///
/// # Notes
///
/// - Ripgrep returns 1 if nothing is found, and 2 if any actual error occurs.
///   Check [`CommandStatus::ok`] to disinguish success from failure.
///
/// - Ripgrep (unlike fd-find) requires at least one argument (a search pattern).
///   For consistency, if [`Args::targets`] is empty, we pass an empty string
///   (which matches anything).
pub fn rg(args: &Args) -> Result<CommandStatus, CommandNotFound<'_>> {
    let flags: Vec<_> = args.flags.iter().flat_map(Flag::as_rg).collect();
    for name in &*RG_NAMES {
        let targets = if args.targets.is_empty() {
            &[".".to_owned()]
        } else {
            args.targets.as_slice()
        };
        if let Ok(status) = Command::new(name).args(&flags).args(targets).status() {
            let ok = status.success() || status.code() == Some(1);
            return Ok(CommandStatus { name, status, ok });
        }
    }
    Err(CommandNotFound(&RG_NAMES))
}

/// Launches fd-find, then ripgrep, with the specified arguments.
pub fn main_with_args(args: &Args) -> Result<(), Error> {
    for f in [fd, rg] {
        match f(args) {
            Ok(CommandStatus { ok: true, .. }) => {}
            Ok(CommandStatus { name, status, .. }) => {
                return Err(Error::Status { name, status });
            }
            Err(CommandNotFound(names)) => eprintln!("warning: can't find any of {names:?}"),
        }
    }
    Ok(())
}

/// Launches fd-find, then ripgrep, with arguments from [`std::env::args`].
pub fn main() -> ExitCode {
    if let Err(e) = Args::from_env().and_then(|args| main_with_args(&args)) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
