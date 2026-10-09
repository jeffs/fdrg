//! Error types.

use std::{
    fmt,
    path::{Path, PathBuf},
    process::ExitStatus,
};

/// The top-level error type exposed by this crate.
#[derive(Debug)]
pub enum Error {
    /// The argument was not recognized as a long option.
    Flag(String),
    /// The character was not recognized as a short option.
    FlagChar(char),
    /// The named command returned bad status.
    Status {
        name: &'static Path,
        status: ExitStatus,
    },
}

/// None of the named commands was executable in `PATH`.
pub struct CommandNotFound<'a>(pub &'a [PathBuf]);

/// The named command returned the status.
pub struct CommandStatus {
    /// The name of the command's executable.
    pub name: &'static Path,
    /// The code returned by the command.
    pub status: ExitStatus,
    /// Whether this status indicates successful execution.
    ///
    /// This may differ from [`ExitStatus::success`], as ripgrep returns 1 if a
    /// search finds no matches (and 2 if anything actually goes wrong).
    pub ok: bool,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flag(s) => write!(f, "bad flag: {s}"),
            Self::FlagChar(c) => write!(f, "bad flag char: {c}"),
            Self::Status { name, status } => write!(f, "{} returned {status}", name.display()),
        }
    }
}
