"""Run the isolated board-v2 benchmark and preserve its reproducibility data.

The frozen cross-engine v1 benchmark has a different contract.  This runner
records the exact local build inputs and all individual v2 samples so a later
comparison never mistakes a changed backend, compiler, or feature set for a
board result.
"""

import argparse
import hashlib
import json
import os
import platform
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
INPUTS = (
    "Cargo.lock",
    "Cargo.toml",
    "benches/board_v2.rs",
    "tests/data/board-v2.tsv",
    "tests/data/board-v2-oracle.tsv",
)


def command(*args: str) -> str:
    return subprocess.run(
        args, cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.strip()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)

    result = subprocess.run(
        ["cargo", "bench", "--bench", "board_v2"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    (output / "board-v2.txt").write_text(result.stdout, encoding="utf-8")
    manifest = {
        "command": "cargo bench --bench board_v2",
        "git_head": command("git", "rev-parse", "HEAD"),
        "git_status": command("git", "status", "--porcelain"),
        "rustc_vv": command("rustc", "-Vv"),
        "cargo_version": command("cargo", "-V"),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "processor": os.environ.get("PROCESSOR_IDENTIFIER", ""),
        "rustflags": os.environ.get("RUSTFLAGS", ""),
        "cargo_encoded_rustflags": os.environ.get("CARGO_ENCODED_RUSTFLAGS", ""),
        "inputs_sha256": {item: digest(ROOT / item) for item in INPUTS},
    }
    (output / "board-v2-manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
