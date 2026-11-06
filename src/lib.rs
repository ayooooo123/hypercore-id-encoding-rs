#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::fmt;

/// The key text was not a 32-byte key in any accepted form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidKey;

impl fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Invalid Hypercore key")
    }
}

impl std::error::Error for InvalidKey {}

/// A key as z-base-32 (52 characters), as `hypercore-id-encoding` encodes it.
pub fn encode(key: &[u8; 32]) -> String {
    z32::encode(key)
}

/// `pear://<id>[/…]`, 52 z-base-32 characters or 64 hex characters.
pub fn decode(id: &str) -> Result<[u8; 32], InvalidKey> {
    let id = match id.strip_prefix("pear://") {
        Some(rest) => rest.split('/').next().unwrap_or(""),
        None => id,
    };
    let raw = match id.len() {
        52 => z32::decode(id).map_err(|_| InvalidKey)?,
        64 => (0..64)
            .step_by(2)
            .map(|i| u8::from_str_radix(id.get(i..i + 2)?, 16).ok())
            .collect::<Option<Vec<u8>>>()
            .ok_or(InvalidKey)?,
        _ => return Err(InvalidKey),
    };
    raw.try_into().map_err(|_| InvalidKey)
}

/// Any accepted form, re-encoded as z-base-32.
pub fn normalize(id: &str) -> Result<String, InvalidKey> {
    decode(id).map(|k| encode(&k))
}

pub fn is_valid(id: &str) -> bool {
    decode(id).is_ok()
}

/// z-base-32 exactly as npm `z32` 1.x: MSB-first quintets, no padding, and a
/// decoder that reads missing trailing characters as zero.
pub mod z32 {
    pub const ALPHABET: &[u8; 32] = b"ybndrfg8ejkmcpqxot1uwisza345h769";

    /// A character outside the alphabet, and its position.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InvalidChar(pub usize);

    pub fn encode(data: &[u8]) -> String {
        let bits = data.len() * 8;
        let mut out = String::with_capacity(bits.div_ceil(5));
        let byte = |i: usize| data.get(i).copied().unwrap_or(0) as u32;
        for p in (0..bits).step_by(5) {
            let (i, j) = (p >> 3, p & 7);
            let v = if j <= 3 {
                (byte(i) >> (3 - j)) & 31
            } else {
                let of = j - 3;
                ((byte(i) << of) & 31) | (byte(i + 1) >> (8 - of))
            };
            out.push(ALPHABET[v as usize] as char);
        }
        out
    }

    fn quintet(s: &[u8], i: usize) -> Result<u32, InvalidChar> {
        let Some(&c) = s.get(i) else { return Ok(0) };
        ALPHABET
            .iter()
            .position(|&a| a == c)
            .map(|v| v as u32)
            .ok_or(InvalidChar(i))
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, InvalidChar> {
        let s = s.as_bytes();
        let mut out = Vec::with_capacity((s.len() * 5).div_ceil(8));
        let q = |i| quintet(s, i);
        let mut ps = 0;
        let full = s.len() / 8;
        for _ in 0..full {
            let v: [u32; 8] = std::array::from_fn(|k| q(ps + k).unwrap_or(0));
            for k in 0..8 {
                q(ps + k)?;
            }
            ps += 8;
            out.extend_from_slice(&group(v, 5));
        }
        // Bytes produced for a trailing partial group of r characters.
        let r = s.len() % 8;
        let n = match r {
            0 => 0,
            1 | 2 => 1,
            3 | 4 => 2,
            5 => 3,
            _ => 4,
        };
        if n > 0 {
            let v: [u32; 8] = std::array::from_fn(|k| q(ps + k).unwrap_or(0));
            for k in 0..r {
                q(ps + k)?;
            }
            out.extend_from_slice(&group(v, n));
        }
        Ok(out)
    }

    fn group([a, b, c, d, e, f, g, h]: [u32; 8], n: usize) -> Vec<u8> {
        let bytes = [
            (a << 3) | (b >> 2),
            ((b & 3) << 6) | (c << 1) | (d >> 4),
            ((d & 15) << 4) | (e >> 1),
            ((e & 1) << 7) | (f << 2) | (g >> 3),
            ((g & 7) << 5) | h,
        ];
        bytes[..n].iter().map(|&x| x as u8).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn matches_npm_fixtures() {
        let g: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/golden.json")).unwrap();
        for c in g["encode"].as_array().unwrap() {
            let bytes = unhex(c["bytes"].as_str().unwrap());
            assert_eq!(z32::encode(&bytes), c["z32"].as_str().unwrap());
            assert_eq!(z32::decode(c["z32"].as_str().unwrap()).unwrap(), bytes);
        }
        for c in g["decode"].as_array().unwrap() {
            let s = c["z32"].as_str().unwrap();
            assert_eq!(
                z32::decode(s).unwrap(),
                unhex(c["bytes"].as_str().unwrap()),
                "{s}"
            );
        }
        for c in g["ids"].as_array().unwrap() {
            let input = c["input"].as_str().unwrap();
            let want = c["key"].as_str().map(unhex);
            assert_eq!(decode(input).ok().map(|k| k.to_vec()), want, "{input:?}");
        }
        let key = decode(g["ids"][0]["input"].as_str().unwrap()).unwrap();
        assert_eq!(
            normalize(&hex(&key)).unwrap(),
            g["normalized"].as_str().unwrap()
        );
        assert_eq!(z32::decode("yb0"), Err(z32::InvalidChar(2)));
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
}
