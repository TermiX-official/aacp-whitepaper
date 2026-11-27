#!/usr/bin/env python3
"""Run synthetic acceptance/rejection cases in the actual SP1 guest."""
import argparse
import hashlib
import json
import pathlib
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("runner", type=pathlib.Path)
    parser.add_argument("elf", type=pathlib.Path)
    parser.add_argument("--output", type=pathlib.Path, default=pathlib.Path("evidence.local.json"))
    args = parser.parse_args()
    elf_bytes = args.elf.read_bytes()
    assert elf_bytes[:5] == b"\x7fELF\x02", "Expected ELF64"
    assert int.from_bytes(elf_bytes[18:20], "little") == 243, "Expected RISC-V"
    elf_hash = hashlib.sha256(elf_bytes).hexdigest()
    cases = [
        ("threshold-20", b" ".join([b"word"] * 20), True, 0, 20),
        ("threshold-19", b" ".join([b"word"] * 19), False, 1, 19),
        ("empty", b"", False, 1, 0),
        ("invalid-utf8", b"\xff", False, 3, 0),
        ("limit-65536", b"a" * 65536, False, 1, 1),
        ("oversized-65537", b"a" * 65537, False, 2, 0),
        ("unicode-nbsp", "one\u00a0two".encode(), False, 1, 1),
    ]
    rows = []
    for name, data, accepted, code, words in cases:
        digest = hashlib.sha256(data).hexdigest()
        result = subprocess.run([str(args.runner.resolve()), str(args.elf.resolve()), elf_hash, digest], input=data, capture_output=True, timeout=120, check=False)
        assert result.returncode == (0 if accepted else 1), (name, "wrong exit status")
        row = json.loads(result.stdout)
        assert row["backend"] == "sp1-6.10.0-light-execution"
        assert row["proofGenerated"] is False
        assert (row["accepted"], row["code"], row["words"]) == (accepted, code, words), name
        assert row["inputSha256"] == digest and row["elfSha256"] == elf_hash
        assert 0 < row["instructions"] <= 200_000_000
        rows.append({"case": name, **row})
        print(f"PASS {name}: {row['instructions']} guest instructions", flush=True)

    for field, expected in [("elf", "ELF_HASH_MISMATCH"), ("input", "INPUT_HASH_MISMATCH")]:
        data = b" ".join([b"word"] * 20)
        digest = hashlib.sha256(data).hexdigest()
        command = [str(args.runner.resolve()), str(args.elf.resolve()), "0" * 64 if field == "elf" else elf_hash, "0" * 64 if field == "input" else digest]
        result = subprocess.run(command, input=data, capture_output=True, timeout=15, check=False)
        assert result.returncode == 1 and result.stdout == b""
        assert result.stderr.decode().strip() == expected
        rows.append({"case": f"changed-{field}-pin", "rejectedBeforeExecution": True, "code": expected})
        print(f"PASS changed-{field}-pin", flush=True)

    data = b"x" * 65538
    result = subprocess.run([str(args.runner.resolve()), str(args.elf.resolve()), elf_hash, hashlib.sha256(data).hexdigest()], input=data, capture_output=True, timeout=15, check=False)
    assert result.returncode == 1 and result.stdout == b""
    assert result.stderr.decode().strip() == "INPUT_FRAME_TOO_LARGE"
    rows.append({"case": "oversized-host-frame", "rejectedBeforeExecution": True, "code": "INPUT_FRAME_TOO_LARGE"})
    print("PASS oversized-host-frame", flush=True)

    args.output.write_text(json.dumps({"schemaVersion": 1, "mode": "real-zkvm-execution-only", "proofGenerated": False, "paidServiceUsed": False, "elfArchitecture": "riscv64", "elfSha256": elf_hash, "cases": rows}, indent=2) + "\n")


if __name__ == "__main__":
    main()
