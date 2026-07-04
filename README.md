# Live Steel

Live Steel is a standalone Rust/WASM + HTML prototype for particle-letter
"living metal" typography.

The release artifact is `mf_proto_live_steel.html`. It embeds the WASM kernel
and uses the bundled `assets/LiveSteelProof.woff2` font.

## Verification

The archive this repository was restored from marked the package as release
eligible in `review_manifest.json`.

Run the local verification gate from the repository root:

```bash
python verify_artifact.py --root . --manifest review_manifest.json --mode release
```

Useful source files:

- `src/lib.rs` - Rust/WASM kernel
- `template.html` - HTML template used by the build script
- `build.py` - standalone HTML builder
- `capture_live_steel_proof.js` - Playwright proof capture
- `proof_stats.json` and `causal_audit.json` - captured verification evidence
