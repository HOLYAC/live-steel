# Live Steel Panel Truth Lab

The panel should not merely expose knobs. It should prove that its knobs reach the metal.

The old smoke was a polite doorman: it checked that the UI accepted a per-letter value. The truth lab is the furnace gauge. It checks the field alpha buffer that feeds WASM physics, before render shimmer, trails, glints, browser timing, or screenshots can lie.

## What it measures

`tools/panel_truth_probe.cjs` opens a built `live_steel_panel` artifact with Playwright and reads phrase-0 alpha straight from the standalone's `views().alpha` memory. It computes an FNV-1a hash over the raw field bytes and pairs that with mass, active-cell count, bounds, current phrase index, and non-gating canvas signal.

The probe is intentionally differential:

1. Neutral single-phrase field for `codex trace`.
2. Aggregate per-letter patch: rotation, scale, x/y, density.
3. Re-apply the same patch and require byte-identical hash.
4. Individual per-letter probes for scale, x, y, rotation, spacing, density, and hidden.
5. Render-control invariance: glints, shadows, dust, trails, brightness, and fade must not change field alpha.
6. Text-style effect: scale must change field alpha.
7. Multi-phrase morph disable: visual class, `aria-disabled`, real disabled controls, blocked API mutation, and unchanged alpha.
8. Morph continuity by deterministic `running=false` + `__reset(seed)` + `__steps(...)`, reading phrase index from `views().perf[2]`.
9. PNG export and zero page errors.

Screenshots are still captured, but their hash is explicitly `NonGating`. The gate trusts deterministic physics input, not phase-weather.

## Negative control

`tools/panel_sabotage_per_letter.cjs` creates a dead panel copy by removing the two actual effect paths:

- glyph overrides are replaced with `defaultLetter(...)`;
- spacing is removed from layout.

State plumbing remains alive. A weak smoke will still brag that `rotation = -18`. The truth probe must fail with `NO effect on the field alpha`. CI should run the negative control after the healthy probe. A test suite that cannot catch its own staged corpse is a lantern painted on a wall.

## Candidate policy

`tools/panel_truth_candidate.json` pins host-deterministic hashes for the scheduled/manual capture job. The first local run can omit `--candidate` to print observed values. Commit the candidate only after a clean, repeated capture on the same CI image.

The important shape is not the particular hex string. The important shape is:

```text
neutral != aggregate
aggregate == aggregateAgain
renderInvariant == neutral
morph walk includes both phrase indices
sabotaged panel exits non-zero
```

## Commands

```bash
python3 tools/build_live_steel_panel.py \
  --source mf_proto_live_steel.html \
  --out .tmp_live_steel_panel_built.html

node tools/panel_truth_probe.cjs \
  --html=.tmp_live_steel_panel_built.html \
  --out=.tmp_live_steel_panel_truth.png \
  --json=.tmp_live_steel_panel_truth.json \
  --candidate=tools/panel_truth_candidate.json

node tools/panel_sabotage_per_letter.cjs \
  --in=.tmp_live_steel_panel_built.html \
  --out=.tmp_live_steel_panel_sabotaged.html

if node tools/panel_truth_probe.cjs \
  --html=.tmp_live_steel_panel_sabotaged.html \
  --json=.tmp_live_steel_panel_sabotaged_truth.json; then
  echo "sabotaged panel passed the truth probe" >&2
  exit 1
else
  echo "negative control failed as expected"
fi
```

## First autopsy (2026-07-04, Linux canon host)

The lab caught a live defect on its first healthy run: `textStyle.scale` was a
silent no-op for wide single-line text — the autofit width clamp renormalized
font size and ate the user multiplier entirely. UI moved, metal never heard it.
Fixed by applying user scale after autofit. All five candidate pins written
before any Chromium run reproduced exactly on the canon host (neutral
`93c63a8f:112896`, aggregate `f47e94a2:112896`, render-invariant == neutral,
walk `[0,1,0]`, steppedLitRatio `0.028125` at 1e-12 tolerance), and the staged
corpse fails with `scale per-letter patch had NO effect on the field alpha`.
