# Live Steel Panel patch

Adds a separate `live_steel_panel.html` workshop for `mf_proto_live_steel.html` without touching canonical proof files.

## Usage

Place `live_steel_panel.html` next to `mf_proto_live_steel.html` and open it. If direct file loading is blocked by the browser, click **Pick mf_proto_live_steel.html**.

For a fully inlined panel artifact:

```bash
python3 tools/build_live_steel_panel.py --source mf_proto_live_steel.html --out live_steel_panel_built.html
```

## Implemented

- free text input and field rebuild
- multi-phrase morph: separate up to 4 phrases with `|`, mode `morph` cycles them
- play, pause, reset, manual lock, auto-lock by milliseconds
- seed control
- PNG export from the canvas
- live, locked, and poster presets
- render toggles for glints, shadows, dust, trails, causal traces, debug overlays
- text controls for uppercase, scale, tracking, word gap, line gap, x/y position, line-break mode
- per-letter selection from list or canvas click
- per-letter scale, x/y offset, rotation, spacing push, density, hide/show, reset
- per-letter scope is single-phrase; the section auto-disables under `|` and says why
- JSON preset save, load, copy; shareable `#preset=` links (work through the loader iframe too)
- seed dice; custom seed survives window resize (template rebuild is wrapped)
- debug overlay exposes all nine field layers
- executable smoke verifier: the command below builds the inlined panel, drives
  it through Playwright, checks free text, per-letter controls, multi-phrase
  morph, PNG export, and nonblank canvas signal

Run the smoke verifier without changing the release manifest:

```bash
python tools/build_live_steel_panel.py --source mf_proto_live_steel.html --out .tmp_live_steel_panel_built.html
node tools/smoke_live_steel_panel.cjs --html=.tmp_live_steel_panel_built.html --out=.tmp_live_steel_panel_smoke.png
```

The same smoke runs inside the scheduled/manual GitHub Actions `capture` job,
after Playwright install and before canonical proof capture.

## v2 review notes

v1 reviewed against ratified `mf_proto_live_steel.html` (commit 557427b): zero page errors,
ESLint clean, canonical evidence untouched. Three real defects found and fixed in v2:
dead `#preset=` hash in the iframe path, silent seed reset to FIXED_SEED on window resize,
and `morph` mode being a no-op with a single phrase. Smoke evidence in `panel_smoke/`.
This integration adds a machine smoke gate over those claims instead of leaving
them as screenshots only.

## v4 — Truth Lab

The panel must prove its knobs reach the metal, and the test must catch its own
staged corpse. `tools/panel_truth_probe.cjs` reads phrase-0 field alpha straight
from WASM memory and gates on differential hashes: every per-letter knob (scale,
x, y, rotation, spacing, density, hidden) must change the field; render controls
must NOT; re-applying a patch must reproduce the hash byte-exact; the morph walk
is stepped deterministically. `tools/panel_sabotage_per_letter.cjs` builds a dead
panel copy for the CI negative control — healthy pass + corpse fail, every run.
`tools/panel_truth_candidate.json` pins the host-deterministic values for the
capture job. Contract change in v4: per-letter is hard-disabled under multi-phrase
(`aria-disabled`, real `disabled` controls, `setLetter` API refuses mutation), and
`textStyle.scale` now applies after autofit so it works on wide text. Model and
first-autopsy record: `docs/PANEL_TRUTH_LAB.md`.
