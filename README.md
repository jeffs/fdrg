# fdrg

A CLI and library to search for files by both name and content.

`fdrg` works by forwarding your search patterns to both the [fd][] and [ripgrep
(rg)][] commands on the host system, if available. This approximates how search
works in most desktop applications; for example, if you hit ⌘K in Notion or ⇧⌘F
in Obsidian, they search for notes whose names *or* contents match your pattern.
Searching from the command line should just as convenient.

By default, `fdrg` looks for `fd` (or `fdfind`) and `rg` in your `PATH`. You can
alternatively assign names or paths to `FDRG_FD_PATH` and `FDRG_RG_PATH`.

This crate has no library dependencies, nor does it automatically install
fd-find or ripgrep. Install those using your system package manager; for
example:

```sh
sudo apt-get install fd-find ripgrep     # Debian
brew install fd ripgrep                  # macOS
```

Or, install from source:

```sh
cargo install --locked fd-find ripgrep
```

Both fd-find and ripgrep are also available via
[`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall).

## Usage

```sh
fdrg [OPTIONS...] [PATTERN] [PATHS...]
```

The following options are accepted and forwarded directly to both fd-find and
ripgrep:

```sh
-F | --fixed-strings
-s | --ignore-case
-u | --unrestricted
```

Additionally, the following option is forwarded (if specified) to `rg` only:

```sh
-l | --files-with-matches
```

## Exit status

Returns success unless fd-find or ripgrep returns bad status. For fd-find, "bad"
means nonzero, but for ripgrep it means "greater than 1" because ripgrep returns
1 if no match is found (and 2 if anything actually went wrong). The exit status
of `fdrg` does not necessarily  match the exit code of the failed subprocess.

If either tool is not found, a warning is printed to stderr, but this is not
considered failure: A missing command does not cause nonzero exit status.
