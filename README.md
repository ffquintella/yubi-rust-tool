# ykchalresp (Rust)

A minimal Rust `ykchalresp` implementation for HMAC-SHA1 challenges. By default it talks directly to a real YubiKey via the Rust `yubikey-hmac-otp` crate; with `-s` it simulates the response in software using a configured secret.

## Features
- Flags: `-1`/`-2` select slot (default: slot 2), `-x` uses hex for challenge and output, `-s` simulates without hardware.
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

## Configuration (simulation mode)
Provide the slot secret as hex (20 bytes typical for HMAC-SHA1):
- Env vars: `YKCHALRESP_SLOT1_KEY`, `YKCHALRESP_SLOT2_KEY`
- Files: `~/.config/ykchalresp/slot1.key`, `~/.config/ykchalresp/slot2.key`

## Usage
- Hardware (default): `printf 'abcdef' | ykchalresp -2`
- Hardware hex: `ykchalresp -2 -x deadbeef`
- Simulation: `YKCHALRESP_SLOT2_KEY=001122... ykchalresp -s -2 -x deadbeef`
- From stdin: `printf 'abcdef' | ykchalresp -s -2 -x`
  - Help: `ykchalresp -h`

Exit codes: `0` success, `1` secret/IO error, `2` invalid input/usage.

## Man Page
- View locally: `man ./man/ykchalresp.1` (or copy to your manpath)
- Installed by `cargo install-with-man` to `${PREFIX}/share/man/man1/ykchalresp.1`.

## Notes & Compatibility
- Hardware mode requires a connected YubiKey and OS support for USB HID via `rusb` (used by `yubikey-hmac-otp`). On failure, use `-s` for software simulation.
- Simulation uses in-tree HMAC-SHA1 (no external crates). For production hardening, consider audited crypto crates.

## Security
- Never commit secrets. Prefer env vars or a protected config file with strict permissions.
- Avoid `unwrap()` in production paths; this tool returns errors for invalid input and missing secrets.

## License
See `LICENSE`.
