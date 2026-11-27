use aacp_word_count_checker::{evaluate, Verdict, MAX_BYTES, MIN_WORDS};
use std::io::{self, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    // Read at most one byte beyond the limit; never buffer arbitrary stdin.
    let mut input = Vec::new();
    let read = io::stdin()
        .lock()
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut input);
    let (accepted, code, words) = match read {
        Err(_) => (false, "INPUT_READ_FAILED", None),
        Ok(_) => match evaluate(&input) {
            Verdict::Accepted { words } => (true, "MINIMUM_MET", Some(words)),
            Verdict::TooShort { words } => (false, "TOO_FEW_WORDS", Some(words)),
            Verdict::TooLarge => (false, "INPUT_TOO_LARGE", None),
            Verdict::InvalidUtf8 => (false, "INVALID_UTF8", None),
        },
    };
    let words = words.map_or_else(|| "null".to_owned(), |n| n.to_string());
    // Only fixed codes and counts are emitted: never echo document content.
    let result = writeln!(io::stdout().lock(),
        "{{\"version\":1,\"accepted\":{accepted},\"code\":\"{code}\",\"words\":{words},\"minWords\":{MIN_WORDS},\"maxBytes\":{MAX_BYTES}}}");
    if accepted && result.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
