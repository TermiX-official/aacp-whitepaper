use std::io::Write;
use std::process::{Command, Stdio};

fn run(input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_aacp-word-count-checker"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env(
            "ZKVM_JOB_DESCRIPTION",
            "untrusted metadata must not change policy",
        )
        .env("ZKVM_DELIVERABLE_HASH", "not-a-verified-hash")
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn supplied_pass_and_fail_fixtures_use_stdin_and_exit_codes() {
    let pass = run(include_bytes!("../fixtures/pass.txt"));
    assert_eq!(pass.status.code(), Some(0));
    assert_eq!(String::from_utf8(pass.stdout).unwrap(), "{\"version\":1,\"accepted\":true,\"code\":\"MINIMUM_MET\",\"words\":20,\"minWords\":20,\"maxBytes\":65536}\n");
    let fail = run(include_bytes!("../fixtures/fail.txt"));
    assert_eq!(fail.status.code(), Some(1));
    assert!(String::from_utf8(fail.stdout)
        .unwrap()
        .contains("\"code\":\"TOO_FEW_WORDS\""));
    assert!(pass.stderr.is_empty() && fail.stderr.is_empty());
}

#[test]
fn malformed_and_oversized_documents_are_rejected_without_echo() {
    for (input, code) in [
        (vec![0xff], "INVALID_UTF8"),
        (vec![b'x'; 65_537], "INPUT_TOO_LARGE"),
    ] {
        let output = run(&input);
        assert_eq!(output.status.code(), Some(1));
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains(code));
        assert!(text.len() < 200);
        assert!(output.stderr.is_empty());
    }
}
