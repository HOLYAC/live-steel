import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SRC = ROOT / "src" / "lib.rs"
TPL = ROOT / "template.html"
BUILD = ROOT / "build.py"
CAPTURE = ROOT / "capture_live_steel_proof.js"
ARTIFACT = ROOT / "verify_artifact.py"
PACKAGE = ROOT / "package.json"
LOCK = ROOT / "package-lock.json"
GATE_CONFIG = ROOT / "gate_config.json"
RUST_TOOLCHAIN = ROOT / "rust-toolchain.toml"
FONT_LICENSE = ROOT / "FONT_LICENSE.md"
FONT = ROOT / "assets" / "LiveSteelProof.woff2"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def fail(message: str) -> None:
    print(f"FAIL: {message}")
    sys.exit(1)


def assert_absent(pattern: str, text: str, label: str) -> None:
    if re.search(pattern, text):
        fail(f"{label}: pattern present: {pattern}")


def assert_present(pattern: str, text: str, label: str) -> None:
    if not re.search(pattern, text):
        fail(f"{label}: pattern missing: {pattern}")


def main() -> None:
    src = read(SRC)
    tpl = read(TPL)
    build = read(BUILD)
    capture = read(CAPTURE)
    artifact = read(ARTIFACT)
    both = src + "\n" + tpl
    scripts = build + "\n" + capture + "\n" + artifact

    for required in [ARTIFACT, PACKAGE, LOCK, GATE_CONFIG, RUST_TOOLCHAIN, FONT_LICENSE, FONT]:
        if not required.exists():
            fail(f"required release file missing: {required.relative_to(ROOT)}")

    assert_absent(r"seat|sampleSeats|MAX_SEATS", both, "destination architecture")
    assert_absent(r"\bsmooth\(", src, "ambiguous scalar falloff")
    assert_absent(r"globalCompositeOperation\s*=\s*['\"]screen['\"]", tpl, "screen haze")
    assert_absent(r"lineCap\s*=\s*['\"]round['\"]", tpl, "round body rods")
    assert_absent(r"OUT_STRIDE\s*=\s*11", both, "old output stride")
    assert_absent(r"search_reach\s*=", src, "conditional hash reach")
    assert_absent(r"sample\.potential\s*\*\s*0\.30", src, "global potential activation path")
    assert_absent(r"phrase\s*==\s*s\.cur\s*\|\|\s*phrase\s*==\s*s\.nxt", src, "wand state leak")
    assert_absent(r"density_error", tpl + "\n" + capture, "false density debug label")
    assert_absent(r"mf_live_steel_review_pack_v11", capture, "stale capture output")
    assert_absent(r"ROOT,\s*['\"]\.\.['\"],\s*['\"]mf_proto_live_steel\.html['\"]", capture, "parent HTML default")
    assert_absent(r"C:\\\\Windows\\\\Fonts|C:\\Windows\\Fonts|Program Files/Google/Chrome|/usr/bin/google-chrome|/usr/bin/chromium", scripts, "host fallback path")
    assert_absent(r"stats\.proof\[3\]", capture, "array-position proof gate")
    assert_absent(r"let\s+d2\s*=\s*r_x\s*\*\s*r_x\s*\+\s*r_y\s*\*\s*r_y\s*\+\s*SOFTEN", src, "softened endpoint reach gate")
    assert_absent(r"buildLockedGlyphLayer|drawLockedFilings|lockedGlyphCache|glyphLayouts", tpl, "render-only glyph layer")
    assert_absent(r"drawImage\(\s*cached|destination-in", tpl, "cached glyph mask compositing")

    assert_present(r"const OUT_STRIDE:\s*usize\s*=\s*14", src, "rust output stride")
    assert_present(r"OUT_STRIDE!==14", tpl, "js stride guard")
    assert_present(r"PERF_STRIDE!==16", tpl, "strict perf ABI guard")
    assert_present(r"fn smooth_up", src, "smooth_up")
    assert_present(r"fn smooth_down", src, "smooth_down")
    assert_present(r"endpoint_force_scalar", src, "endpoint dipole sign helper")
    assert_present(r"axis_delta", src, "axis-periodic torque")
    assert_present(r"activation:\s*Vec<f32>", src, "phrase-indexed activation storage")
    assert_present(r"fn phrase_writes_wand", src, "phase-owned wand writer")
    assert_present(r"write_wand:\s*bool", src, "explicit wand side-effect flag")
    assert_present(r"fn endpoint_broad_reach", src, "endpoint-complete broadphase")
    assert_present(r"r2_raw\s*=\s*r_x\s*\*\s*r_x\s*\+\s*r_y\s*\*\s*r_y", src, "raw endpoint distance")
    assert_present(r"d2_force\s*=\s*r2_raw\s*\+\s*SOFTEN", src, "softened endpoint denominator")
    assert_present(r"endpoint_gate_uses_raw_distance_for_chain_far", src, "raw endpoint reach regression test")
    assert_present(r"PAIR_BROAD_REACH\s*/\s*cell_w", src, "broad hash reach x")
    assert_present(r"PAIR_BROAD_REACH\s*/\s*cell_h", src, "broad hash reach y")
    assert_present(r"center_possible", src, "center body broadphase split")
    assert_present(r"endpoint_possible", src, "endpoint broadphase split")
    assert_present(r"window\.__captureProofSet", tpl, "proof harness")
    assert_present(r"window\.__shot", tpl, "single-frame capture helper")
    assert_present(r"window\.__debugField\s*=\s*0", tpl, "debug field default off")
    assert_present(r"window\.__captureDebugSet", tpl, "debug capture harness")
    assert_present(r"window\.__coverage", tpl, "coverage proof helper")
    assert_present(r"window\.__slotAudit", tpl, "slot identity audit helper")
    assert_present(r"window\.__staticLatticeAudit", tpl, "static lattice audit helper")
    assert_present(r"window\.__primaryCoverage", tpl, "primary rods coverage helper")
    assert_present(r"window\.__secondaryCoverage", tpl, "secondary halo coverage helper")
    assert_present(r"window\.__secondaryHalo", tpl, "secondary surplus halo audit helper")
    assert_present(r"window\.__particleDependencyAudit", tpl, "particle buffer dependency audit helper")
    assert_present(r"window\.__debugTargetIsolationAudit", tpl, "debug target isolation audit helper")
    assert_present(r"window\.__identityShuffleAudit", tpl, "particle-slot identity shuffle audit helper")
    assert_present(r"window\.__renderVariantAudit", tpl, "effect-off render variant audit helper")
    assert_present(r"window\.__visibleProvenanceAudit", tpl, "visible particle provenance audit helper")
    assert_present(r"window\.__continuityAudit", tpl, "no-teleport continuity audit helper")
    assert_present(r"window\.__glyphFieldPoisonAudit", tpl, "hidden glyph field poison audit helper")
    assert_present(r"window\.__letterAudit", tpl, "per-letter glyph audit helper")
    assert_present(r"letterAuditFromOcc", tpl, "per-letter occupancy audit implementation")
    assert_present(r"canvasShapeAudit", tpl, "rendered canvas shape audit helper")
    assert_present(r"window\.__motionProbe", tpl, "temporal life proof helper")
    assert_present(r"window\.__activationSums", tpl, "morph activation proof helper")
    assert_present(r"window\.__pixelStats", tpl, "canvas pixel proof helper")
    assert_present(r"window\.__simOnly", tpl, "sim-only perf helper")
    assert_present(r"drawDebugTangent", tpl, "tangent debug mode")
    assert_present(r"kind\s*==\s*11", src, "chain debug field")
    assert_present(r"s\.rho\[i\]", src, "current density debug field")
    assert_present(r"activated_potential\s*=\s*sample\.potential\s*\*\s*sample\.activation", src, "activation-gated potential")
    assert_present(r"pairSaturationCount", tpl, "pair saturation metric")
    assert_present(r"__FONT_FACE__", tpl, "font placeholder")
    assert_present(r"LiveSteelProof", tpl + "\n" + build, "embedded proof font family")
    assert_present(r"slot_count", src + "\n" + tpl, "slot count ABI")
    assert_present(r"slot_stride", src + "\n" + tpl, "slot stride ABI")
    assert_present(r"slot_fx_ptr", src + "\n" + tpl, "slot x ABI")
    assert_present(r"field_sdf_ptr", src + "\n" + tpl, "field sdf poison ABI")
    assert_present(r"field_target_rho_ptr", src + "\n" + tpl, "field target density poison ABI")
    assert_present(r"activation_ptr", src + "\n" + tpl, "activation poison ABI")
    assert_present(r"heap_x_ptr", src + "\n" + tpl, "birth x ABI")
    assert_present(r"heap_y_ptr", src + "\n" + tpl, "birth y ABI")
    assert_present(r"capture_s_ptr", src + "\n" + tpl, "capture time ABI")
    assert_present(r"trace_len_ptr", src + "\n" + tpl, "causal path ABI")
    assert_present(r"rotation_trace_ptr", src + "\n" + tpl, "causal rotation ABI")
    assert_present(r"rotationRad", tpl, "rotation provenance audit metric")
    assert_present(r"particle_state_ptr", src + "\n" + tpl, "particle state ABI")
    assert_present(r"LOCAL_FONT\s*=\s*CRATE\s*/\s*['\"]assets['\"]\s*/\s*['\"]LiveSteelProof\.woff2['\"]", build, "local font asset default")
    assert_present(r"--allow-host-font", build, "explicit host font escape hatch")
    assert_present(r"document\.fonts\)\s*await\s+document\.fonts\.ready", tpl, "font-ready raster gate")
    assert_present(r"window\.__abi", tpl, "ABI export")
    assert_present(r"density_void", tpl + "\n" + capture, "density void debug label")
    assert_present(r"density_pressure", tpl + "\n" + capture, "density pressure debug label")
    assert_present(r"review_manifest\.json", scripts, "review manifest path")
    assert_present(r"live-steel-review-manifest-v2", build + "\n" + artifact, "manifest v2")
    assert_present(r"release_eligible", build + "\n" + capture + "\n" + artifact, "release eligibility")
    assert_present(r"playwright-managed", capture + "\n" + artifact, "release browser source")
    assert_present(r"--release-browser", capture, "release browser flag")
    assert_present(r"exactlyOne|exactly_one", capture + "\n" + artifact, "frame uniqueness helper")
    assert_present(r"gate_config\.json", scripts, "gate config")
    assert_present(r"src_lib_sha256", build, "source hash manifest")
    assert_present(r"template_sha256", build, "template hash manifest")
    assert_present(r"cargo_lock_sha256", build, "cargo lock hash manifest")
    assert_present(r"rust_toolchain_sha256", build, "rust toolchain hash manifest")
    assert_present(r"package_lock_sha256", build, "package lock hash manifest")
    assert_present(r"gate_config_sha256", build + "\n" + capture, "gate config hash manifest")
    assert_present(r"embedded_font_sha256", build, "font hash manifest")
    assert_present(r"build_py_sha256", build, "build script hash manifest")
    assert_present(r"capture_script_sha256", build, "capture script hash manifest")
    assert_present(r"verify_script_sha256", build, "verify script hash manifest")
    assert_present(r"embedded_wasm_sha256", build, "wasm hash manifest")
    assert_present(r"standalone_html_sha256", build, "html hash manifest")
    assert_present(r"rustc_version", build, "rustc version manifest")
    assert_present(r"cargo_version", build, "cargo version manifest")
    assert_present(r"proof_stats_sha256", capture, "proof stats hash manifest")
    assert_present(r"alpha_hashes", capture, "alpha hash manifest")
    assert_present(r"argValue\(['\"]--html['\"]", capture, "portable html argument")
    assert_present(r"CHROME_PATH", capture, "explicit non-release chrome override")
    assert_present(r"recompute_gates", artifact, "independent gate recomputation")
    assert_present(r"embedded WASM hash mismatch", artifact, "embedded wasm verifier")
    assert_present(r"embedded font hash mismatch", artifact, "embedded font verifier")
    assert_present(r"absolute path in manifest paths block", artifact, "relative path verifier")

    print("PASS: live steel source gates")


if __name__ == "__main__":
    main()
