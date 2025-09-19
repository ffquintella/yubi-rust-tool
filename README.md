# ykchalresp (Rust)

A minimal, dependency-free Rust implementation that mimics the YubiKey `ykchalresp` command for HMAC-SHA1 challenges. It is intended as a drop-in replacement for common CLI usage where the secret is supplied via environment or config file instead of a physical device.

## Features
- Flags: `-1`/`-2` select slot (default: slot 2), `-x` uses hex for challenge and output.
- Challenge from arg or stdin.
- Output: hex with `-x`, modhex otherwise.
- Secrets from env or config files.

## Install
- Build locally: `cargo build --release`
- Binary at `target/release/ykchalresp`
- Optional man page: `man/ykchalresp.1` (see below).

### Install via Cargo (no Makefile)
- Install binary to Cargo bin dir: `cargo install --path .`
  - This places `ykchalresp` in `~/.cargo/bin` (ensure it’s on PATH).
- Uninstall binary: `cargo uninstall yubi-rust-tool`
- Optional: install man page into system manpath:
  - `sudo -E cargo man-install` (uses `PREFIX`=/usr/local by default)
  - `sudo -E PREFIX=/opt cargo man-install`
  - Remove man page: `sudo -E cargo man-uninstall`

## Configuration
Provide the slot secret as hex (20 bytes for HMAC-SHA1 typical):
- Env vars: `YKCHALRESP_SLOT1_KEY`, `YKCHALRESP_SLOT2_KEY`
- Files: `~/.config/ykchalresp/slot1.key`, `~/.config/ykchalresp/slot2.key`

## Usage
- Hex mode: `YKCHALRESP_SLOT2_KEY=001122... ykchalresp -2 -x deadbeef`
- From stdin: `printf 'abcdef' | ykchalresp -2 -x`
- Modhex output (no -x): `printf 'abcdef' | ykchalresp -2`
- Help: `ykchalresp -h`

Exit codes: `0` success, `1` secret/IO error, `2` invalid input/usage.

## Man Page
- View locally: `man ./man/ykchalresp.1` (or copy to your manpath)
- Installed by `cargo install-with-man` to `${PREFIX}/share/man/man1/ykchalresp.1`.

## Notes & Compatibility
- This tool does not talk to a YubiKey; it computes HMAC using the provided secret. It matches hardware responses only if the secret matches the device configuration.
- Uses in-tree HMAC-SHA1 (no external crates). For production hardening, consider audited crypto crates.

## Security
- Never commit secrets. Prefer env vars or a protected config file with strict permissions.
- Avoid `unwrap()` in production paths; this tool returns errors for invalid input and missing secrets.

## License
See `LICENSE`.
