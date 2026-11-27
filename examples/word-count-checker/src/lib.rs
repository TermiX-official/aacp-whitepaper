#![no_std]

/// Policy is compiled into the verifier, not supplied by the deliverable.
pub const MIN_WORDS: usize = 20;
pub const MAX_BYTES: usize = 65_536;

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted { words: usize },
    TooShort { words: usize },
    TooLarge,
    InvalidUtf8,
}

/// A word is a nonempty run separated by ASCII whitespace. This is a length
/// check, not a tokenizer, semantic evaluator, or originality check.
pub fn evaluate(bytes: &[u8]) -> Verdict {
    if bytes.len() > MAX_BYTES {
        return Verdict::TooLarge;
    }
    let Ok(text) = core::str::from_utf8(bytes) else {
        return Verdict::InvalidUtf8;
    };
    let words = text.split_ascii_whitespace().count();
    if words >= MIN_WORDS {
        Verdict::Accepted { words }
    } else {
        Verdict::TooShort { words }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec;

    #[test]
    fn threshold_is_fixed_and_exact() {
        assert_eq!(
            evaluate(b"a a a a a a a a a a a a a a a a a a a"),
            Verdict::TooShort { words: 19 }
        );
        assert_eq!(
            evaluate(b"a a a a a a a a a a a a a a a a a a a a"),
            Verdict::Accepted { words: 20 }
        );
    }

    #[test]
    fn empty_and_whitespace_are_not_words() {
        assert_eq!(evaluate(b""), Verdict::TooShort { words: 0 });
        assert_eq!(evaluate(b" \t\r\n\x0c"), Verdict::TooShort { words: 0 });
        assert_eq!(
            evaluate(b"one\ttwo\r\nthree\x0cfour"),
            Verdict::TooShort { words: 4 }
        );
    }

    #[test]
    fn unicode_content_is_valid_but_ascii_separators_define_words() {
        assert_eq!(
            evaluate("سلام world".as_bytes()),
            Verdict::TooShort { words: 2 }
        );
        assert_eq!(
            evaluate("one\u{a0}two".as_bytes()),
            Verdict::TooShort { words: 1 }
        );
    }

    #[test]
    fn malformed_utf8_and_oversized_input_fail_closed() {
        assert_eq!(evaluate(&[0xff]), Verdict::InvalidUtf8);
        assert_eq!(
            evaluate(&vec![b'a'; MAX_BYTES]),
            Verdict::TooShort { words: 1 }
        );
        assert_eq!(evaluate(&vec![b'a'; MAX_BYTES + 1]), Verdict::TooLarge);
    }
}
