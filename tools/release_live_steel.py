import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CAPTURE_DIR = ROOT / "mf_live_steel_review_pack"


def run(command: list[str]) -> None:
    executable = shutil.which(command[0]) or command[0]
    resolved = [executable, *command[1:]]
    print("+ " + " ".join(command), flush=True)
    subprocess.run(resolved, cwd=ROOT, check=True)


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def make_zip(source: Path, out: Path) -> None:
    if out.exists():
        out.unlink()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
        for file in sorted(source.rglob("*")):
            if file.is_dir():
                continue;
            rel = file.relative_to(source).as_posix()
            if rel.startswith("node_modules/") or rel.startswith("target/"):
                continue
            archive.write(file, rel)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=["v17-clean"], default="v17-clean")
    parser.add_argument("--out", default="live_steel_v17_clean.zip")
    args = parser.parse_args()

    out = Path(args.out)
    if not out.is_absolute():
        out = ROOT / out

    run(["npm", "ci"])
    run(["npx", "playwright", "install", "chromium"])
    run(["python", "verify_live_steel.py"])
    run(["npm", "run", "check:js"])
    run(["npm", "run", "lint:js"])
    run(["cargo", "fmt", "--check"])
    run(["cargo", "test"])
    run(["cargo", "clippy", "--target", "wasm32-unknown-unknown", "--release", "--", "-D", "warnings"])
    run(["python", "build.py", "--out", "mf_proto_live_steel.html", "--manifest", "review_manifest.json"])
    run(["python", "verify_artifact.py", "--root", ".", "--manifest", "review_manifest.json", "--mode", "pre-capture"])
    run(["npm", "run", "capture"])
    run([
        "python",
        "verify_artifact.py",
        "--root",
        str(CAPTURE_DIR),
        "--manifest",
        "review_manifest.json",
        "--mode",
        "release",
    ])
    run([
        "node",
        "tools/audit_causal_letters.cjs",
        "--html=mf_live_steel_review_pack/mf_proto_live_steel.html",
        "--source=mf_live_steel_review_pack/template.html",
        "--out=mf_live_steel_review_pack/causal_audit.json",
    ])
    make_zip(CAPTURE_DIR, out)
    print(f"release_zip={out}")
    print(f"release_sha256={sha256_file(out)}")


if __name__ == "__main__":
    main()
