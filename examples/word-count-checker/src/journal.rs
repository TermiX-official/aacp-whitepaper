use crate::{Verdict, MAX_BYTES, MIN_WORDS};

pub const JOURNAL_LEN: usize = 54;
const MAGIC: &[u8; 8] = b"AACPWCV1";

/// Public result bound to SHA-256 of the exact input bytes.
/// Codes: 0 accepted, 1 too short, 2 too large, 3 invalid UTF-8.
#[derive(Debug, PartialEq, Eq)]
pub struct Journal {
    pub input_sha256: [u8; 32],
    pub words: u32,
    pub accepted: bool,
    pub code: u8,
}

impl Journal {
    pub fn new(input_sha256: [u8; 32], verdict: Verdict) -> Self {
        let (words, accepted, code) = match verdict {
            Verdict::Accepted { words } => (words as u32, true, 0),
            Verdict::TooShort { words } => (words as u32, false, 1),
            Verdict::TooLarge => (0, false, 2),
            Verdict::InvalidUtf8 => (0, false, 3),
        };
        Self {
            input_sha256,
            words,
            accepted,
            code,
        }
    }

    pub fn encode(&self) -> [u8; JOURNAL_LEN] {
        let mut bytes = [0; JOURNAL_LEN];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..40].copy_from_slice(&self.input_sha256);
        bytes[40..44].copy_from_slice(&(MIN_WORDS as u32).to_le_bytes());
        bytes[44..48].copy_from_slice(&(MAX_BYTES as u32).to_le_bytes());
        bytes[48..52].copy_from_slice(&self.words.to_le_bytes());
        bytes[52] = u8::from(self.accepted);
        bytes[53] = self.code;
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != JOURNAL_LEN || &bytes[..8] != MAGIC {
            return None;
        }
        let u32_at = |i| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
        if u32_at(40) != MIN_WORDS as u32 || u32_at(44) != MAX_BYTES as u32 {
            return None;
        }
        let words = u32_at(48);
        let valid = match (bytes[52], bytes[53]) {
            (1, 0) => (MIN_WORDS as u32..=MAX_BYTES as u32).contains(&words),
            (0, 1) => words < MIN_WORDS as u32,
            (0, 2 | 3) => words == 0,
            _ => false,
        };
        valid.then(|| Self {
            input_sha256: bytes[8..40].try_into().unwrap(),
            words,
            accepted: bytes[52] == 1,
            code: bytes[53],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_all_verdicts_and_input_binding() {
        for verdict in [
            Verdict::Accepted { words: 20 },
            Verdict::TooShort { words: 19 },
            Verdict::TooLarge,
            Verdict::InvalidUtf8,
        ] {
            let journal = Journal::new([7; 32], verdict);
            assert_eq!(Journal::decode(&journal.encode()), Some(journal));
        }
    }
    #[test]
    fn malformed_length_policy_and_verdict_fail_closed() {
        let original = Journal::new([7; 32], Verdict::Accepted { words: 20 }).encode();
        assert!(Journal::decode(&original[..53]).is_none());
        for index in [0, 40, 44, 52, 53] {
            let mut bytes = original;
            bytes[index] ^= 0xff;
            assert!(Journal::decode(&bytes).is_none());
        }
    }
}
