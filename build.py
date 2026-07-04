import argparse
import base64
import hashlib
import json
import os
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

CRATE = Path(__file__).resolve().parent
SRC = CRATE / "src" / "lib.rs"
CARGO_TOML = CRATE / "Cargo.toml"
CARGO_LOCK = CRATE / "Cargo.lock"
RUST_TOOLCHAIN = CRATE / "rust-toolchain.toml"
TEMPLATE = CRATE / "template.html"
BUILD_SCRIPT = CRATE / "build.py"
CAPTURE_SCRIPT = CRATE / "capture_live_steel_proof.js"
VERIFY_SCRIPT = CRATE / "verify_live_steel.py"
ARTIFACT_VERIFY_SCRIPT = CRATE / "verify_artifact.py"
PACKAGE_JSON = CRATE / "package.json"
PACKAGE_LOCK = CRATE / "package-lock.json"
GATE_CONFIG = CRATE / "gate_config.json"
MANIFEST_SCHEMA = CRATE / "review_manifest.schema.json"
FONT_LICENSE = CRATE / "FONT_LICENSE.md"
LOCAL_FONT = CRATE / "assets" / "LiveSteelProof.woff2"
DEFAULT_OUT = CRATE / "mf_proto_live_steel.html"
TARGET = "wasm32-unknown-unknown"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def run_text(command: list[str]) -> str:
    try:
        return subprocess.run(command, check=True, capture_output=True, text=True).stdout.strip()
    except (FileNotFoundError, subprocess.CalledProcessError) as exc:
        return f"unavailable: {exc}"


def require_file(path: Path, label: str) -> None:
    if not path.exists():
        raise SystemExit(f"missing required {label}: {path}")


def rel(path: Path) -> str:
    return path.resolve().relative_to(CRATE.resolve()).as_posix()


def optional_hash(path: Path) -> str | None:
    return sha256_file(path) if path.exists() else None


def font_format(path: Path) -> tuple[str, str]:
    suffix = path.suffix.lower()
    if suffix == ".woff2":
        return "font/woff2", "woff2"
    if suffix == ".otf":
        return "font/otf", "opentype"
    if suffix == ".ttf":
        return "font/truetype", "truetype"
    raise SystemExit(f"unsupported proof font format: {path}")


def font_face(font: Path) -> tuple[str, dict[str, str | int]]:
    data = font.read_bytes()
    mime, fmt = font_format(font)
    b64 = base64.b64encode(data).decode("ascii")
    css = (
        "@font-face{font-family:'LiveSteelProof';"
        f"src:url(data:{mime};base64,{b64}) format('{fmt}');"
        "font-weight:900;font-style:normal;font-display:block}"
    )
    return css, {
        "path": rel(font) if font.resolve().is_relative_to(CRATE.resolve()) else str(font),
        "sha256": sha256_bytes(data),
        "bytes": len(data),
        "format": fmt,
        "license_status": "bundled-ofl-1.1",
    }


def package_version(package: str) -> str:
    return run_text(["node", "-p", f"require('{package}/package.json').version"])


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default=str(DEFAULT_OUT), help="Standalone HTML output path")
    parser.add_argument("--manifest", default=None, help="Manifest output path")
    parser.add_argument("--font", default=str(LOCAL_FONT), help="Bundled proof font path")
    parser.add_argument("--allow-host-font", action="store_true", help="Allow non-release host font for local exploration")
    args = parser.parse_args()

    out = Path(args.out).resolve()
    manifest_path = Path(args.manifest).resolve() if args.manifest else out.with_name("review_manifest.json")
    font_path = Path(args.font).resolve()
    bundled_font = font_path == LOCAL_FONT.resolve()

    for path, label in [
        (SRC, "Rust source"),
        (CARGO_TOML, "Cargo.toml"),
        (CARGO_LOCK, "Cargo.lock"),
        (RUST_TOOLCHAIN, "rust-toolchain.toml"),
        (TEMPLATE, "template"),
        (PACKAGE_JSON, "package.json"),
        (PACKAGE_LOCK, "package-lock.json"),
        (GATE_CONFIG, "gate_config.json"),
        (MANIFEST_SCHEMA, "review_manifest.schema.json"),
        (FONT_LICENSE, "FONT_LICENSE.md"),
    ]:
        require_file(path, label)

    if not font_path.exists():
        raise SystemExit(f"missing bundled release font: {font_path}")
    if not bundled_font and not args.allow_host_font:
        raise SystemExit("host font requires --allow-host-font and is not release eligible")

    cargo_command = [
        "cargo",
        "build",
        "--release",
        "--target",
        TARGET,
        "--manifest-path",
        str(CARGO_TOML),
    ]
    subprocess.run(cargo_command, check=True)

    wasm = CRATE / "target" / TARGET / "release" / "mf_live_steel_wasm.wasm"
    wasm_bytes = wasm.read_bytes()
    font_css, font_manifest = font_face(font_path)
    html = TEMPLATE.read_text(encoding="utf-8")
    html = html.replace("__FONT_FACE__", font_css)
    html = html.replace("__WASM_B64__", base64.b64encode(wasm_bytes).decode("ascii"))

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(html, encoding="utf-8", newline="\n")

    release_eligible = bundled_font
    paths = {
        "src_lib": rel(SRC),
        "cargo_toml": rel(CARGO_TOML),
        "cargo_lock": rel(CARGO_LOCK),
        "rust_toolchain": rel(RUST_TOOLCHAIN),
        "template": rel(TEMPLATE),
        "font": rel(font_path) if bundled_font else str(font_path),
        "font_license": rel(FONT_LICENSE),
        "standalone_html": rel(out) if out.is_relative_to(CRATE) else str(out),
        "proof_stats": "proof_stats.json",
        "gate_config": rel(GATE_CONFIG),
        "package_json": rel(PACKAGE_JSON),
        "package_lock": rel(PACKAGE_LOCK),
        "build_script": rel(BUILD_SCRIPT),
        "capture_script": rel(CAPTURE_SCRIPT),
        "verify_script": rel(VERIFY_SCRIPT),
        "artifact_verify_script": rel(ARTIFACT_VERIFY_SCRIPT) if ARTIFACT_VERIFY_SCRIPT.exists() else "verify_artifact.py",
        "manifest_schema": rel(MANIFEST_SCHEMA),
    }
    hashes = {
        "src_lib_sha256": sha256_file(SRC),
        "cargo_toml_sha256": sha256_file(CARGO_TOML),
        "cargo_lock_sha256": sha256_file(CARGO_LOCK),
        "rust_toolchain_sha256": sha256_file(RUST_TOOLCHAIN),
        "template_sha256": sha256_file(TEMPLATE),
        "font_license_sha256": sha256_file(FONT_LICENSE),
        "embedded_font_sha256": font_manifest["sha256"],
        "build_py_sha256": sha256_file(BUILD_SCRIPT),
        "capture_script_sha256": sha256_file(CAPTURE_SCRIPT),
        "verify_script_sha256": sha256_file(VERIFY_SCRIPT),
        "artifact_verify_script_sha256": optional_hash(ARTIFACT_VERIFY_SCRIPT),
        "manifest_schema_sha256": sha256_file(MANIFEST_SCHEMA),
        "package_json_sha256": sha256_file(PACKAGE_JSON),
        "package_lock_sha256": sha256_file(PACKAGE_LOCK),
        "gate_config_sha256": sha256_file(GATE_CONFIG),
        "embedded_wasm_sha256": sha256_bytes(wasm_bytes),
        "standalone_html_sha256": sha256_file(out),
    }
    manifest = {
        "schema": "live-steel-review-manifest-v2",
        "release_eligible": release_eligible,
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "paths": paths,
        "hashes": hashes,
        "font": font_manifest,
        "toolchain": {
            "rustc_version": run_text(["rustc", "--version"]),
            "cargo_version": run_text(["cargo", "--version"]),
            "target": TARGET,
            "node_version": run_text(["node", "--version"]),
            "playwright_version": package_version("playwright"),
            "browser_source": "not-captured",
            "browser_version": "not-captured",
        },
        "build": {
            "cargo_command": cargo_command,
            "build_command": [sys.executable, *sys.argv],
        },
        "origin": {
            "os": platform.platform(),
            "cpu": platform.processor(),
            "paths": {
                key: str((CRATE / value).resolve()) if not Path(value).is_absolute() else value
                for key, value in paths.items()
            },
        },
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"wasm {len(wasm_bytes)} B -> {out}")
    print(f"manifest -> {manifest_path}")


if __name__ == "__main__":
    main()
