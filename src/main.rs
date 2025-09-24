use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

fn usage() -> &'static str {
    "ykchalresp [-1|-2] [-x] [-s] [challenge]\n\n\
    -1        use slot 1 (default: slot 2)\n\
    -2        use slot 2\n\
    -x        challenge and response are hex-encoded\n\
    -s        simulate in software (no hardware)\n\
    If no challenge is provided, read from stdin.\n\
    Default: use a real YubiKey directly via the Rust 'yubikey-hmac-otp' crate.\n\
    Simulation (-s): compute HMAC using a secret loaded from:\n\
      env:  YKCHALRESP_SLOT1_KEY / YKCHALRESP_SLOT2_KEY (hex)\n\
      file: ~/.config/ykchalresp/slot1.key or slot2.key (hex)\n"
}

fn main() {
    let mut slot: u8 = 2; // default slot 2 to match common usage
    let mut hex_mode = false;
    let mut simulate = false;
    let mut challenge_arg: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-1" => slot = 1,
            "-2" => slot = 2,
            "-x" => hex_mode = true,
            "-s" => simulate = true,
            "-h" | "--help" => {
                eprint!("{}", usage());
                return;
            }
            "-V" | "--version" => {
                println!("ykchalresp (yubi-rust-tool) {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            _ => {
                // First non-flag is the challenge; pass the rest through unchanged
                if challenge_arg.is_none() {
                    challenge_arg = Some(a);
                } else {
                    // Unexpected extra arg; treat as error for clarity
                    eprintln!("Unexpected argument: {}\n\n{}", a, usage());
                    std::process::exit(2);
                }
            }
        }
    }

    // In hardware mode we don't need to load the secret.

    // Read challenge from arg or stdin
    let challenge_bytes = match challenge_arg {
        Some(s) => parse_challenge(&s, hex_mode),
        None => {
            let mut buf = String::new();
            if io::stdin().read_to_string(&mut buf).is_err() {
                eprintln!("Failed to read challenge from stdin");
                std::process::exit(1);
            }
            parse_challenge(buf.trim_end(), hex_mode)
        }
    };

    if simulate {
        // Simulation: load secret and compute locally
        let secret = match load_slot_secret(slot) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };

        // Compute HMAC-SHA1
        let mac = hmac_sha1(&secret, &challenge_bytes);

        if hex_mode {
            println!("{}", to_hex(&mac));
        } else {
            // Default ykchalresp uses modhex for output when not using -x
            let hex = to_hex(&mac);
            println!("{}", to_modhex(&hex));
        }
    } else {
        // Hardware: invoke system ykchalresp tool and forward the challenge
        match run_hardware(slot, hex_mode, &challenge_bytes) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

fn parse_challenge(input: &str, hex_mode: bool) -> Vec<u8> {
    if hex_mode {
        match from_hex(input) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Invalid hex challenge: {}", e);
                std::process::exit(2);
            }
        }
    } else {
        // Take raw bytes; trim trailing newline already done by caller
        input.as_bytes().to_vec()
    }
}

fn load_slot_secret(slot: u8) -> Result<Vec<u8>, String> {
    // 1) Env var takes precedence
    let env_name = match slot {
        1 => "YKCHALRESP_SLOT1_KEY",
        2 => "YKCHALRESP_SLOT2_KEY",
        _ => return Err("Invalid slot; must be 1 or 2".to_string()),
    };
    if let Ok(val) = env::var(env_name) {
        return from_hex(&val).map_err(|e| format!("{} contains invalid hex: {}", env_name, e));
    }

    // 2) Config file under ~/.config/ykchalresp
    let mut path = dirs_home().ok_or_else(|| "Cannot resolve home directory".to_string())?;
    path.push(".config/ykchalresp");
    let file = match slot {
        1 => "slot1.key",
        2 => "slot2.key",
        _ => unreachable!(),
    };
    path.push(file);
    let content = fs::read_to_string(&path).map_err(|_| {
        format!(
            "Missing secret for slot {}. Set {} or create {} with hex key.",
            slot,
            env_name,
            path.display()
        )
    })?;
    let trimmed = content.trim();
    from_hex(trimmed).map_err(|e| format!("{} contains invalid hex: {}", path.display(), e))
}

fn dirs_home() -> Option<PathBuf> {
    // Minimal home dir resolution without external crates
    if let Ok(home) = env::var("HOME") {
        return Some(PathBuf::from(home));
    }
    None
}

// ----- Encoding helpers -----

fn to_hex(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(data.len() * 2);
    for &b in data {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err("odd-length hex string".into());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let val = |c: u8| -> Result<u8, String> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(10 + c - b'a'),
            b'A'..=b'F' => Ok(10 + c - b'A'),
            _ => Err(format!("invalid hex digit: {}", c as char)),
        }
    };
    let mut i = 0;
    while i < bytes.len() {
        let hi = val(bytes[i])?;
        let lo = val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn to_modhex(hex: &str) -> String {
    // Map nibbles 0..15 to modhex characters
    const MODHEX: [char; 16] = [
        'c', 'b', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'n', 'r', 't', 'u', 'v',
    ];
    let mut out = String::with_capacity(hex.len());
    for b in hex.bytes() {
        let v = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => 10 + b - b'a',
            b'A'..=b'F' => 10 + b - b'A',
            _ => {
                // Ignore non-hex (shouldn't happen with our to_hex)
                continue;
            }
        } as usize;
        out.push(MODHEX[v]);
    }
    out
}

// ----- HMAC-SHA1 (no external crates) -----

fn hmac_sha1(key: &[u8], message: &[u8]) -> [u8; 20] {
    const BLOCK: usize = 64;
    let mut k = if key.len() > BLOCK {
        sha1(key).to_vec()
    } else {
        key.to_vec()
    };
    k.resize(BLOCK, 0);

    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] = k[i] ^ 0x36;
        opad[i] = k[i] ^ 0x5c;
    }

    let mut inner = Vec::with_capacity(BLOCK + message.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(message);
    let inner_hash = sha1(&inner);

    let mut outer = Vec::with_capacity(BLOCK + inner_hash.len());
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_hash);
    sha1(&outer)
}

fn sha1(message: &[u8]) -> [u8; 20] {
    // Minimal SHA-1 implementation sufficient for HMAC
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    // Pre-processing: padding
    let ml = (message.len() as u64) * 8;
    let mut data = message.to_vec();
    data.push(0x80);
    while (data.len() % 64) != 56 {
        data.push(0);
    }
    data.extend_from_slice(&ml.to_be_bytes());

    for chunk in data.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            let j = i * 4;
            w[i] = ((chunk[j] as u32) << 24)
                | ((chunk[j + 1] as u32) << 16)
                | ((chunk[j + 2] as u32) << 8)
                | (chunk[j + 3] as u32);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for i in 0..80 {
            let (f, k) = if i < 20 {
                ((b & c) | ((!b) & d), 0x5A827999)
            } else if i < 40 {
                (b ^ c ^ d, 0x6ED9EBA1)
            } else if i < 60 {
                ((b & c) | (b & d) | (c & d), 0x8F1BBCDC)
            } else {
                (b ^ c ^ d, 0xCA62C1D6)
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

// ----- Hardware via yubikey crate (OTP HMAC-SHA1 challenge-response) -----

fn run_hardware(slot: u8, hex_mode: bool, challenge: &[u8]) -> Result<String, String> {
    use yubikey_hmac_otp::config::{Config, Mode, Slot};
    use yubikey_hmac_otp::Yubico;

    // Discover a YubiKey
    let mut y = Yubico::new();
    let yk = y
        .find_yubikey()
        .map_err(|e| format!("Failed to find YubiKey: {}", e))?;

    let slot = match slot {
        1 => Slot::Slot1,
        2 => Slot::Slot2,
        _ => return Err("Invalid slot; must be 1 or 2".to_string()),
    };

    let conf = Config::new_from(yk).set_mode(Mode::Sha1).set_slot(slot);
    let hmac = y
        .challenge_response_hmac(challenge, conf)
        .map_err(|e| format!("YubiKey HMAC-SHA1 challenge failed: {}", e))?;

    if hex_mode {
        Ok(to_hex(&hmac[..]))
    } else {
        Ok(to_modhex(&to_hex(&hmac[..])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_known_vector() {
        let d = sha1(b"The quick brown fox jumps over the lazy dog");
        assert_eq!(to_hex(&d), "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
    }

    #[test]
    fn hmac_sha1_known_vector() {
        let d = hmac_sha1(b"key", b"The quick brown fox jumps over the lazy dog");
        assert_eq!(to_hex(&d), "de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9");
    }

    #[test]
    fn modhex_maps_hex_nibbles() {
        // deadbeef -> d e a d b e e f -> t u l t n u u v
        assert_eq!(to_modhex("deadbeef"), "tultnuuv");
    }
}
