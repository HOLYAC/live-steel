import argparse
import base64
import hashlib
import json
import re
import sys
from pathlib import Path


class VerificationError(Exception):
    pass


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def rel_path(root: Path, value: str) -> Path:
    path = Path(value)
    require(not path.is_absolute(), f"absolute path in manifest paths block: {value}")
    require(".." not in path.parts, f"parent traversal in manifest paths block: {value}")
    return root / path


def exactly_one(items: list[dict], predicate, label: str) -> dict:
    matches = [item for item in items if predicate(item)]
    require(len(matches) == 1, f"expected exactly one {label}, got {len(matches)}")
    return matches[0]


def extract_embedded_wasm(html: str) -> bytes:
    match = re.search(
        r'<script id="wasmb64" type="application/octet-stream">([A-Za-z0-9+/=\s]+)</script>',
        html,
    )
    require(match is not None, "embedded WASM block missing")
    return base64.b64decode(re.sub(r"\s+", "", match.group(1)))


def extract_embedded_font(html: str) -> bytes:
    match = re.search(r"url\(data:font/[^;]+;base64,([A-Za-z0-9+/=\s]+)\)", html)
    require(match is not None, "embedded font block missing")
    return base64.b64decode(re.sub(r"\s+", "", match.group(1)))


def recompute_gates(stats: dict, config: dict) -> dict:
    thresholds = config["thresholds"]
    dwell_frame = config["proofFrames"][-1]
    dwell = exactly_one(
        stats["proof"],
        lambda item: item["frame"] == dwell_frame and item["report"]["phase"] == "dwell",
        f"dwell proof frame {dwell_frame}",
    )
    morph_first_frame = config["morphFrames"][0]
    morph_last_frame = config["morphFrames"][-1]
    morph_first = exactly_one(
        stats["morph"],
        lambda item: item["frame"] == morph_first_frame,
        f"morph frame {morph_first_frame}",
    )
    morph_last = exactly_one(
        stats["morph"],
        lambda item: item["frame"] == morph_last_frame,
        f"morph frame {morph_last_frame}",
    )
    activation0_falls = morph_last["activation"][0]["sum"] < morph_first["activation"][0]["sum"]
    activation1_rises = morph_last["activation"][1]["sum"] > morph_first["activation"][1]["sum"]
    perf_values = sorted(run["msPerStep"] for run in stats["simOnly"])
    p95_pos = 0.95 * (len(perf_values) - 1)
    p95_lo = int(p95_pos)
    p95_hi = min(len(perf_values) - 1, p95_lo + 1)
    p95_t = p95_pos - p95_lo
    perf_max = max(perf_values)
    perf_p95 = perf_values[p95_lo] + (perf_values[p95_hi] - perf_values[p95_lo]) * p95_t
    return {
        "deterministic": all(item["equal"] for item in stats["determinism"]),
        "noPageErrors": len(stats["pageErrors"]) == 0,
        "denseStrokes": dwell["coverage"]["all"]["coveredRatio"] >= thresholds["dwellCoveredRatioMin"]
        and dwell["coverage"]["all"]["holePreservation"] >= thresholds["dwellHolePreservationMin"],
        "readableMetalTypography": dwell["pixel"]["over128Ratio"] <= thresholds["dwellOver128RatioMax"]
        and dwell["pixel"]["over180Ratio"] <= thresholds["dwellOver180RatioMax"]
        and dwell["pixel"]["blueDominantRatio"] <= thresholds["dwellBlueDominantRatioMax"]
        and dwell["report"]["jsGlintRatio"] >= thresholds["dwellJsGlintRatioMin"]
        and dwell["report"]["jsGlintRatio"] <= thresholds["dwellJsGlintRatioMax"],
        "temporalLife": stats["temporal"]["dwell1"]["active"]["avg"] > thresholds["temporalDwell1ActiveAvgMin"]
        and stats["temporal"]["dwell1"]["active"]["p95"] < thresholds["temporalDwell1ActiveP95Max"]
        and stats["temporal"]["morph1"]["active"]["p95"] < thresholds["temporalMorph1ActiveP95Max"],
        "morphContinuity": activation0_falls
        and activation1_rises
        and morph_last["report"]["avgActive"] > thresholds["morphLastAvgActiveMin"],
        "simOnlyPerfMaxMs": round(perf_max, 4),
        "simOnlyPerfP95Ms": round(perf_p95, 4),
        "simOnlyPerfWithin8ms": perf_p95 <= thresholds["simOnlyP95MsPerStepMax"],
    }


def compare_hash(root: Path, paths: dict, hashes: dict, key: str, path_key: str, rows: list[tuple[str, str]]) -> None:
    expected = hashes.get(key)
    require(expected, f"manifest hash missing: {key}")
    path = rel_path(root, paths[path_key])
    require(path.exists(), f"manifest path missing: {path_key}: {path}")
    actual = sha256_file(path)
    require(actual == expected, f"{key} mismatch: {actual} != {expected}")
    rows.append((key, "PASS"))


def verify_png_hashes(root: Path, stats: dict, rows: list[tuple[str, str]]) -> None:
    checked = 0
    for collection in ["proof", "proofRepeat", "debug", "morph"]:
        for item in stats[collection]:
            image = item["image"]
            path = root / image["file"]
            require(path.exists(), f"PNG missing: {image['file']}")
            actual = sha256_file(path)
            require(actual == image["sha256"], f"PNG hash mismatch: {image['file']}")
            checked += 1
    for item in stats["determinism"]:
        require(item["first"] == item["second"] and item["equal"], f"repeat mismatch at frame {item['frame']}")
    rows.append((f"png_hashes:{checked}", "PASS"))


def verify(root: Path, manifest_path: Path, mode: str) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    manifest = read_json(manifest_path)
    require(manifest.get("schema") == "live-steel-review-manifest-v2", "manifest schema is not v2")
    rows.append(("manifest_schema", "PASS"))

    paths = manifest["paths"]
    hashes = manifest["hashes"]
    for value in paths.values():
        rel_path(root, value)
    rows.append(("relative_paths", "PASS"))

    release_mode = mode == "release"
    if release_mode:
        require(manifest.get("release_eligible") is True, "release manifest is not eligible")
        require(manifest["toolchain"].get("browser_source") == "playwright-managed", "release browser is not Playwright-managed")
        require((root / paths["package_lock"]).exists(), "package-lock.json missing")
        require((root / paths["rust_toolchain"]).exists(), "rust-toolchain.toml missing")
        require((root / paths["font_license"]).exists(), "FONT_LICENSE.md missing")
        rows.append(("release_eligible", "PASS"))

    hash_pairs = [
        ("src_lib_sha256", "src_lib"),
        ("cargo_toml_sha256", "cargo_toml"),
        ("cargo_lock_sha256", "cargo_lock"),
        ("rust_toolchain_sha256", "rust_toolchain"),
        ("template_sha256", "template"),
        ("font_license_sha256", "font_license"),
        ("build_py_sha256", "build_script"),
        ("capture_script_sha256", "capture_script"),
        ("verify_script_sha256", "verify_script"),
        ("artifact_verify_script_sha256", "artifact_verify_script"),
        ("manifest_schema_sha256", "manifest_schema"),
        ("package_json_sha256", "package_json"),
        ("package_lock_sha256", "package_lock"),
        ("gate_config_sha256", "gate_config"),
        ("standalone_html_sha256", "standalone_html"),
    ]
    for hash_key, path_key in hash_pairs:
        if hashes.get(hash_key) is not None:
            compare_hash(root, paths, hashes, hash_key, path_key, rows)

    html_path = rel_path(root, paths["standalone_html"])
    html = html_path.read_text(encoding="utf-8")
    require(sha256_bytes(extract_embedded_wasm(html)) == hashes["embedded_wasm_sha256"], "embedded WASM hash mismatch")
    require(sha256_bytes(extract_embedded_font(html)) == hashes["embedded_font_sha256"], "embedded font hash mismatch")
    rows.append(("embedded_assets", "PASS"))

    if mode == "pre-capture":
        return rows

    proof_stats_path = rel_path(root, paths["proof_stats"])
    require(proof_stats_path.exists(), "proof_stats.json missing")
    require(sha256_file(proof_stats_path) == hashes["proof_stats_sha256"], "proof_stats hash mismatch")
    stats = read_json(proof_stats_path)
    config = read_json(rel_path(root, paths["gate_config"]))
    recomputed = recompute_gates(stats, config)
    for key, value in recomputed.items():
        require(stats["gates"].get(key) == value, f"stored gate differs from recomputed gate: {key}")
    require(all(value is True for key, value in recomputed.items() if isinstance(value, bool)), "one or more proof gates failed")
    require(stats["htmlSha256"] == hashes["standalone_html_sha256"], "stats html hash mismatch")
    if manifest.get("alpha_hashes"):
        require(manifest["alpha_hashes"] == stats["alphaHashes"], "alpha hashes mismatch")
    verify_png_hashes(root, stats, rows)
    rows.append(("recomputed_gates", "PASS"))
    return rows


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", required=True)
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--mode", choices=["pre-capture", "release"], default="release")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    manifest_path = (root / args.manifest).resolve() if not Path(args.manifest).is_absolute() else Path(args.manifest)
    try:
        rows = verify(root, manifest_path, args.mode)
    except VerificationError as exc:
        print(f"FAIL: {exc}")
        sys.exit(1)

    width = max(len(name) for name, _status in rows)
    for name, status in rows:
        print(f"{name.ljust(width)}  {status}")


if __name__ == "__main__":
    main()
