# Live Steel V17 / Brake Surfacer - финальный план лучше V16

Дата: 2026-06-30

Объект: `ca1d0754-eadd-4c95-a18e-006d1eeb2532.zip`

Архив SHA256: `5d45cc9031c7c0079a47d960ee62059f2204d0875f8398746a2d5967568ddd35`

Статус ответа: да, лучше можно

Смысл улучшения: не больше идей, а меньше недоказанных щелей

Финальная стратегия: V16 сохранить как физический baseline, V17 сделать release-hardening слоем

Запрет: не переписывать Live Steel заново

Запрет: не возвращать старый CURB как главный объект

Запрет: не тюнить картинку до закрытия replay/provenance

---

# ДЕФЕКТ

## D0 - V16-план был хорошим, но не самым жёстким

V16-план верно выбрал направление: сохранить V16 и закрыть воспроизводимость.

V16-план оставил несколько мест с мягкими формулировками.

V16-план не поймал один математический дефект endpoint reach.

V16-план не заставил capture использовать pinned browser by default.

V16-план не запретил host-font fallback в release build.

V16-план не потребовал `rust-toolchain.toml` как enforce gate.

V16-план не вынес thresholds в versioned gate config.

V16-план не требовал lint, который ловит duplicate JS object keys.

V17 должен исправить именно это.

## D1 - REVIEW_BRIEF overclaims portability

Brief говорит: build/capture portable after unzip.

Архив не содержит `package.json`.

Архив не содержит JS lockfile.

`capture_live_steel_proof.js` импортирует `playwright`.

Чистый unzip не может гарантированно выполнить capture без внешнего знания dependency graph.

Значит claim portability сейчас не доказан.

Верный статус: evidence bundle is internally consistent, but replay package is incomplete.

## D2 - Browser capture сейчас не pinned достаточно жёстко

Capture script сначала ищет explicit `--chrome` или `CHROME_PATH`.

Потом capture script автоматически выбирает system Chrome paths.

Текущий `proof_stats.json` записал browser executable: `C:/Program Files/Google/Chrome/Application/chrome.exe`.

Текущий browser version: `149.0.7827.201`.

Это доказывает конкретный Windows Chrome run.

Это не доказывает Playwright-managed pinned Chromium replay.

Release capture должен по умолчанию использовать Playwright-managed Chromium.

System Chrome должен быть explicit non-release mode.

## D3 - Rust toolchain recorded, но не enforced

Manifest записал `rustc 1.95.0`.

Manifest записал `cargo 1.95.0`.

Архив не содержит `rust-toolchain.toml`.

Clean checkout может собрать другим Rust.

Другой Rust может дать другой WASM hash.

Source-to-WASM claim без enforced toolchain остаётся мягким.

## D4 - Manifest всё ещё смешивает portable identity и origin evidence

`review_manifest.json` содержит абсолютные Windows paths.

Абсолютные paths полезны как origin evidence.

Абсолютные paths вредны как portable release identity.

Release manifest должен хранить relative paths.

Origin paths должны жить отдельно.

Portable verifier должен fail на absolute artifact paths в `paths` block.

## D5 - Font fallback опасен

`build.py` использует bundled `assets/LiveSteelProof.ttf`, если он есть.

Если bundled font отсутствует, `build.py` может fallback-нуться на host font.

Host font меняет glyph alpha.

Host font меняет alpha hashes.

Host font может создать license bomb.

Release build должен fail-closed, если bundled release font отсутствует.

Release build не должен silently embed `C:\Windows\Fonts\arialbd.ttf`.

## D6 - Endpoint reach чуть-чуть сжат из-за SOFTEN

В `pair_interaction` endpoint distance сейчас считается как `d2 = r_x*r_x + r_y*r_y + SOFTEN`.

Потом check использует `if d2 > CHAIN_FAR^2`.

Это означает, что геометрический reach меньше `CHAIN_FAR`.

Фактический max raw endpoint gap: `sqrt(18.5^2 - 14.0) = 18.1177`.

Контракт говорит endpoint reach `18.5`.

Код фактически даёт примерно `18.12`.

Ошибка мала визуально.

Ошибка принципиальна для proof claim.

Правильная модель: raw distance gates reach, softened distance gates force denominator.

## D7 - Gate summary fragile by array position

`gateSummary` берёт `stats.proof[3]` как dwell evidence.

Это верно только пока proof array order не поменялся.

Gate должен выбирать frame by `frame === 330` and phase by `phase === dwell`.

Gate должен fail на missing expected frame.

Gate должен fail на duplicate expected frame.

Proof gate не должен зависеть от array luck.

## D8 - JS syntax gate не ловит duplicate object keys

В `capture_live_steel_proof.js` object literal содержит duplicate `buildManifest` key.

JS syntax check это пропускает.

Duplicate key сейчас harmless.

Но release discipline не должна пропускать такие вещи.

Нужен lint gate: `no-dupe-keys` минимум.

Нужен static gate для capture schema.

## D9 - Thresholds hard-coded inside capture script

Порог `coveredRatio >= 0.78` живёт в JS.

Порог `holePreservation >= 0.9` живёт в JS.

Порог `simOnlyPerf <= 8.0` живёт в JS.

Thresholds должны быть versioned artifact data.

Иначе stats нельзя корректно интерпретировать без конкретной версии script.

Нужен `gate_config.json` с schema version.

## D10 - Proof stats пока не independent-computed evidence

`proof_stats.json` содержит metrics.

`capture_live_steel_proof.js` сам вычисляет gates.

Нужен независимый `verify_artifact.py`, который recompute gates from stored raw metrics.

Нужен verifier, который не доверяет готовому `gates` object.

Stored gates должны быть результатом, а не authority.

## D11 - Performance claim ещё тоньше, чем казалось

`simOnlyPerfWithin8ms` true.

В provided stats max sim-only: `7.4645 ms/step`.

Sample count всего 3.

Render frame time не измерен как release KPI.

Proof frame report perf на frame 330: `123.29 ms`.

Это не равно sim-only budget.

Нельзя claim 60 FPS UI.

Можно claim только provided host sim-only smoke pass.

## D12 - Current local verification boundary

`verify_live_steel.py` локально PASS.

`node --check capture_live_steel_proof.js` локально PASS.

Extracted template script syntax локально PASS.

Embedded WASM instantiates in Node.

Embedded WASM exports required ABI.

`out_stride()` возвращает `13`.

`perf_stride()` возвращает `16`.

`field_w()` возвращает `224`.

`field_h()` возвращает `126`.

Proof PNG hashes match `proof_stats.json`.

Repeat proof PNG hashes equal original proof PNG hashes.

Cargo gates локально не воспроизведены.

Browser recapture локально не воспроизведён.

Причина: в этой среде нет `cargo`, `rustc`, packaged `playwright` dependency.

---

# ИДЕАЛ

## I0 - Лучший возможный claim

Live Steel V16 является proof-bearing visual physics prototype.

Live Steel V17 должен стать reproducible proof-bearing release candidate.

Production UI claim запрещён до render budget и integration gates.

Public proof claim запрещён до CI artifact, clean rebuild, clean recapture, cross-platform metric replay.

## I1 - Что считается доказанным сейчас

Current archive internal consistency доказана.

Current embedded WASM ABI smoke доказан.

Current provided screenshots/hash repeat deterministic within captured run доказан.

Current source static gates доказаны.

Current browser recapture на clean machine не доказан.

Current source-to-WASM rebuild не доказан здесь.

Current cross-platform determinism не доказан.

## I2 - Что должно считаться правильностью V17

Правильность равна replayable artifact, not pretty screenshot.

Правильность равна pinned Rust toolchain.

Правильность равна pinned JS dependencies.

Правильность равна Playwright-managed browser by default.

Правильность равна fail-closed bundled font.

Правильность равна manifest v2 with relative paths.

Правильность равна independent artifact verifier.

Правильность равна rebuild WASM hash or explicitly downgraded semantic provenance.

Правильность равна recapture proof pack from clean unzip.

Правильность равна cross-platform alpha/hash/metric matrix.

## I3 - Status ladder

Level 0: evidence bundle.

Level 0 current status: achieved.

Level 1: clean unzip verifies hashes and stored stats.

Level 1 current status: almost achieved, missing verifier.

Level 2: clean unzip recaptures proof.

Level 2 current status: not achieved, missing JS lock and browser pin.

Level 3: clean checkout rebuilds WASM and matches hash.

Level 3 current status: not achieved here.

Level 4: cross-platform metrics pass.

Level 4 current status: not achieved.

Level 5: production component passes render/integration.

Level 5 current status: not claimed.

## I4 - Best strategy

Keep V16 physics frozen.

Patch only proof/release correctness first.

Patch endpoint raw-distance reach because it is contract-level and tiny.

Do not change visual constants.

Do not change phrase list.

Do not change particle count.

Do not change colors.

Do not change font until font-license gate forces replacement.

Regenerate proof pack only after all release-shell patches land.

---

# МОДЕЛЬ

## M0 - Artifact graph

`src/lib.rs` is simulation source.

`Cargo.toml` is crate configuration.

`Cargo.lock` is Rust dependency lock.

`rust-toolchain.toml` must become Rust toolchain lock.

`assets/LiveSteelProof.ttf` is proof font asset.

`FONT_LICENSE.md` must become font distribution proof.

`template.html` is standalone browser template.

`build.py` is source-to-HTML builder.

`mf_proto_live_steel.html` is embedded WASM + embedded font artifact.

`capture_live_steel_proof.js` is browser proof generator.

`package.json` must become JS command contract.

`package-lock.json` must become JS dependency lock.

`gate_config.json` must become threshold authority.

`review_manifest.json` must become portable provenance ledger.

`proof_stats.json` is raw + derived proof metrics.

`verify_live_steel.py` is static source gate.

`verify_artifact.py` must become independent proof verifier.

## M1 - Evidence table from current archive

Archive file count: `33`.

Archive bytes: `7080091`.

Archive SHA256: `5d45cc9031c7c0079a47d960ee62059f2204d0875f8398746a2d5967568ddd35`.

Standalone HTML SHA256: `05155aed2bac8fe2239ed3f7fad79cb8b8c709793459384718c92b9c59a2cf54`.

Embedded WASM SHA256: `5f5344c2fe647210c20a9ce3dfbacd6344a98a429d15ad56465a09275a69f46f`.

Embedded WASM bytes: `78625`.

Embedded WASM magic: `0061736d`.

Font SHA256: `e8f4e3baf6cc35fed6fcce3a540e8b39e8f6cda1d22a28f2ec8f526fef7a43f5`.

Font bytes: `989780`.

Source SHA256: `6c2988137572efbe869a83e42c0c27bf7cba6d2cb6f19a61b60325600cc7bd63`.

Template SHA256: `170e3ee7dd37ba8ee5829bc8a5306f0136b9383a637d69d717aa15601d538af3`.

Build script SHA256: `069a4a2748ff2f28ba9a69d03383d610204f3b0e92eadda85b78482449f89aa6`.

Capture script SHA256: `4b7ea041bd3e1b2ad94bf0bb1fab4de1c6d00e39d87540c7342cf01b10a72ddd`.

Verify script SHA256: `89bb8a81b47a1515901e0afbdfb759379b881468a11d5b07e7cc511507b4f6c4`.

Proof stats SHA256: `ae28594d2046f30724f1c4b34a41743dc63ecbcdc99b2934375a386a8dc8c630`.

Current gates deterministic: `true`.

Current gates noPageErrors: `true`.

Current gates denseStrokes: `true`.

Current gates noGlowTypography: `true`.

Current gates temporalLife: `true`.

Current gates morphContinuity: `true`.

Current gates simOnlyPerfWithin8ms: `true`.

Current simOnly max: `7.4645 ms/step`.

Current dwell coverage: `0.8024`.

Current dwell hole preservation: `0.9798`.

Current dwell avg chain: `0.6143`.

Current dwell p90 chain: `1.0`.

Current dwell glint ratio: `0.1574`.

Current dwell over128 ratio: `0.000222`.

Current dwell blueDominant ratio: `0.000354`.

## M2 - Gate hierarchy

Gate A: static source architecture.

Gate B: Rust fmt/test/clippy.

Gate C: Rust toolchain pin.

Gate D: source-to-WASM rebuild.

Gate E: manifest schema and hashes.

Gate F: JS syntax and lint.

Gate G: Playwright dependency lock.

Gate H: browser recapture.

Gate I: independent proof_stats recomputation.

Gate J: cross-platform alpha and metric replay.

Gate K: render performance and integration.

Release candidate requires A through J.

Production component requires A through K.

## M3 - Variant comparison

Variant A: rewrite physics.

Variant A verdict: reject.

Reason: V16 has working proof-bearing physics and fixed old P0s.

Variant B: trust REVIEW_BRIEF and ship.

Variant B verdict: reject.

Reason: portability claim is not independently complete.

Variant C: keep V16 and add reproducibility shell.

Variant C verdict: accept but incomplete.

Reason: this was V16 final plan.

Variant D: keep V16, add reproducibility shell, plus fix browser pin, toolchain pin, font fail-closed, endpoint reach, threshold config, independent verifier, lint.

Variant D verdict: best.

Final variant: D.

---

# РЕАЛИЗАЦИЯ

## R0 - Freeze scope

Action: create tag `live-steel-v16-physics-baseline`.

Action: create branch `live-steel-v17-release-hardening`.

Action: allow only release-shell patches before first V17 proof.

Allowed patch class: packaging.

Allowed patch class: manifest.

Allowed patch class: verifier.

Allowed patch class: CI.

Allowed patch class: endpoint reach raw-distance fix.

Forbidden patch class: visual tuning.

Forbidden patch class: particle count change.

Forbidden patch class: phrase/font/style change.

Forbidden patch class: physics constants change except endpoint reach bugfix.

Acceptance: `git diff live-steel-v16-physics-baseline...HEAD` contains no aesthetic drift.

## R1 - Add Rust toolchain lock

Action: add `rust-toolchain.toml`.

Required content:

```toml
[toolchain]
channel = "1.95.0"
targets = ["wasm32-unknown-unknown"]
components = ["rustfmt", "clippy"]
profile = "minimal"
```

Acceptance: `rustc --version` equals manifest toolchain before build.

Acceptance: `cargo --version` equals manifest toolchain before build.

Acceptance: build fails if toolchain differs unless manifest baseline is intentionally updated.

Why: recorded toolchain is evidence, enforced toolchain is a contract.

## R2 - Add JS dependency lock

Action: add `package.json`.

Action: add `package-lock.json`.

Action: pin Playwright exact version.

Action: pin lint dependency exact version if ESLint is used.

Action: commit lockfile.

Action: never ship with placeholder dependency version.

Required `package.json` shape:

```json
{
  "private": true,
  "type": "commonjs",
  "scripts": {
    "check:js": "node --check capture_live_steel_proof.js && node tools/check_template_js.cjs",
    "lint:js": "eslint capture_live_steel_proof.js tools/*.cjs",
    "capture": "node capture_live_steel_proof.js --html=mf_proto_live_steel.html --out=mf_live_steel_review_pack --manifest=review_manifest.json --release-browser",
    "verify": "python3 verify_artifact.py --root . --manifest review_manifest.json"
  },
  "devDependencies": {
    "playwright": "EXACT_VERIFIED_VERSION",
    "eslint": "EXACT_VERIFIED_VERSION"
  }
}
```

Acceptance: committed file has real versions, not `EXACT_VERIFIED_VERSION`.

Acceptance: `npm ci` succeeds from clean checkout.

Acceptance: `npm run check:js` succeeds.

Acceptance: `npm run lint:js` succeeds.

Why: a capture script without dependency lock is a spell, not a machine.

## R3 - Make release capture use Playwright-managed Chromium by default

Current defect: system Chrome auto-detection is used before Playwright bundled browser.

Action: change release mode to forbid implicit system Chrome.

Action: keep `--chrome` only for local exploratory capture.

Action: add `--release-browser` flag that refuses system Chrome.

Action: manifest records `browser_executable_path: playwright-managed` for release capture.

Action: manifest records exact browser version from `browser.version()`.

Required behavior:

```js
function browserExecutable({ releaseBrowser }) {
  const explicit = argValue("--chrome", process.env.CHROME_PATH || "");
  if (releaseBrowser && explicit) {
    throw new Error("release capture forbids --chrome/CHROME_PATH");
  }
  if (releaseBrowser) {
    return null;
  }
  return explicit ? existingPath(explicit) : null;
}
```

Acceptance: CI capture uses Playwright-managed Chromium.

Acceptance: non-release capture with `--chrome` marks manifest as `release_eligible: false`.

Fail behavior: system Chrome capture cannot produce release manifest.

## R4 - Make font handling fail-closed

Current defect: `build.py` can fallback to host font.

Action: remove release fallback to `C:\Windows\Fonts\arialbd.ttf`.

Action: require `assets/LiveSteelProof.ttf` unless `--allow-host-font` is explicitly passed.

Action: host font mode marks manifest as `release_eligible: false`.

Action: add `FONT_LICENSE.md`.

Action: manifest records `font_license_status`.

Required release rule:

```text
No bundled release font.
No release build.
```

Acceptance: deleting `assets/LiveSteelProof.ttf` makes release build fail.

Acceptance: changing font bytes changes alpha hashes and blocks reuse of old proof_stats.

Acceptance: missing font license blocks public release package.

## R5 - Normalize manifest v2

Action: create `review_manifest.schema.json`.

Action: change manifest schema to `live-steel-review-manifest-v2`.

Action: paths block must use relative paths only.

Action: origin block may store absolute paths.

Action: commands must be arrays, not shell strings.

Action: hashes must include `cargo_lock_sha256`.

Action: hashes must include `rust_toolchain_sha256`.

Action: hashes must include `package_lock_sha256`.

Action: hashes must include `gate_config_sha256`.

Action: hashes must include `embedded_font_sha256`.

Action: hashes must include `embedded_wasm_sha256`.

Action: capture block must include browser source: `playwright-managed` or `system-chrome-nonrelease`.

Action: manifest must include `release_eligible` boolean.

Required v2 skeleton:

```json
{
  "schema": "live-steel-review-manifest-v2",
  "release_eligible": true,
  "paths": {
    "src_lib": "src/lib.rs",
    "cargo_toml": "Cargo.toml",
    "cargo_lock": "Cargo.lock",
    "rust_toolchain": "rust-toolchain.toml",
    "template": "template.html",
    "font": "assets/LiveSteelProof.ttf",
    "standalone_html": "mf_proto_live_steel.html",
    "proof_stats": "proof_stats.json",
    "gate_config": "gate_config.json"
  },
  "hashes": {
    "src_lib_sha256": "...",
    "embedded_wasm_sha256": "...",
    "standalone_html_sha256": "...",
    "proof_stats_sha256": "...",
    "package_lock_sha256": "..."
  },
  "toolchain": {
    "rustc_version": "rustc 1.95.0 (59807616e 2026-04-14)",
    "cargo_version": "cargo 1.95.0 (f2d3ce0bd 2026-03-21)",
    "target": "wasm32-unknown-unknown",
    "node_version": "...",
    "playwright_version": "...",
    "browser_source": "playwright-managed",
    "browser_version": "..."
  },
  "origin": {
    "os": "...",
    "cpu": "...",
    "paths": {}
  }
}
```

Acceptance: v2 verifier fails any absolute path under `paths`.

Acceptance: v2 verifier permits absolute paths only under `origin.paths`.

## R6 - Patch endpoint reach math

Current defect: softened distance shrinks geometric endpoint reach.

Action: split raw endpoint distance from softened force distance.

Required patch shape:

```rust
let r2_raw = r_x * r_x + r_y * r_y;
if r2_raw > chain_far2 {
    continue;
}
let d_raw = r2_raw.sqrt();
let d2_force = r2_raw + SOFTEN;
let range_gate = smooth_down(CHAIN_NEAR, CHAIN_FAR, d_raw);
let f_mag = endpoint_force_scalar(q_i, q_j, d2_force, endpoint_gate);
```

Action: use `d_raw` for range gate.

Action: use `d2_force` for force denominator.

Action: do not change `CHAIN_FAR`.

Action: do not retune `K_DIPOLE`.

Action: do not alter center broadphase.

New test:

```rust
#[test]
fn endpoint_chain_reach_is_not_shrunk_by_softening() {
    let raw_gap = CHAIN_FAR - 0.01;
    let r2_raw = raw_gap * raw_gap;
    let softened = r2_raw + SOFTEN;
    assert!(r2_raw <= CHAIN_FAR * CHAIN_FAR);
    assert!(softened > CHAIN_FAR * CHAIN_FAR);
}
```

Better test:

```rust
#[test]
fn endpoint_gate_uses_raw_distance_for_chain_far() {
    let raw_gap = CHAIN_FAR - 0.01;
    let gate = smooth_down(CHAIN_NEAR, CHAIN_FAR, raw_gap);
    assert!(gate > 0.0);
}
```

Acceptance: old six tests pass.

Acceptance: new endpoint reach tests pass.

Acceptance: proof pack is regenerated because WASM hash changes.

## R7 - Refactor proof gates away from array positions

Current defect: `stats.proof[3]` assumes order.

Action: select dwell evidence by frame and phase.

Required helper:

```js
function exactlyOne(items, predicate, label) {
  const matches = items.filter(predicate);
  if (matches.length !== 1) throw new Error(`expected exactly one ${label}, got ${matches.length}`);
  return matches[0];
}
```

Required usage:

```js
const dwell = exactlyOne(
  stats.proof,
  item => item.frame === 330 && item.report.phase === "dwell",
  "dwell proof frame 330"
);
```

Acceptance: shuffling `stats.proof` does not change gates.

Acceptance: deleting frame 330 fails.

Acceptance: duplicating frame 330 fails.

## R8 - Externalize threshold contract

Action: add `gate_config.json`.

Action: capture script reads thresholds from config.

Action: verifier reads same config.

Action: manifest hashes config.

Required config skeleton:

```json
{
  "schema": "live-steel-gate-config-v1",
  "proofFrames": [8, 48, 150, 330],
  "morphFrames": [390, 450, 540, 620],
  "thresholds": {
    "dwellCoveredRatioMin": 0.78,
    "dwellHolePreservationMin": 0.90,
    "dwellOver128RatioMax": 0.0025,
    "dwellBlueDominantRatioMax": 0.002,
    "dwellJsGlintRatioMin": 0.08,
    "dwellJsGlintRatioMax": 0.18,
    "temporalDwell1ActiveAvgMin": 0.015,
    "temporalDwell1ActiveP95Max": 6.0,
    "temporalMorph1ActiveP95Max": 10.0,
    "morphLastAvgActiveMin": 0.30,
    "simOnlyMaxMsPerStepMax": 8.0
  }
}
```

Acceptance: changing thresholds changes `gate_config_sha256`.

Acceptance: proof_stats records `gate_config_sha256`.

Acceptance: verifier recomputes gates using config, not captured booleans.

## R9 - Add independent artifact verifier

Action: create `verify_artifact.py`.

Verifier must recompute archive file hashes.

Verifier must extract embedded WASM from HTML.

Verifier must extract embedded font from HTML.

Verifier must compare embedded hashes to manifest.

Verifier must compare source/template/script hashes to manifest.

Verifier must compare PNG hashes to `proof_stats.json`.

Verifier must compare proof vs repeat PNG hash equality.

Verifier must recompute gate booleans from raw metrics.

Verifier must fail on absolute paths inside manifest `paths`.

Verifier must fail on missing `package-lock.json` for release mode.

Verifier must fail on missing `rust-toolchain.toml` for release mode.

Verifier must fail on missing `FONT_LICENSE.md` for public release mode.

Verifier must fail on `release_eligible=false` in release mode.

Acceptance command:

```bash
python3 verify_artifact.py --root . --manifest review_manifest.json --mode release
```

Acceptance output must be a compact gate table.

Acceptance output must not hide failures inside warnings.

## R10 - Add JS lint gate

Action: add ESLint or equivalent pinned linter.

Minimum rule: `no-dupe-keys`.

Minimum rule: `no-undef`.

Minimum rule: `no-unused-vars` except explicit `_` conventions.

Minimum rule: no implicit globals.

Minimum rule: no hardcoded release system Chrome path.

Minimum rule: no duplicate schema keys in generated stats object.

Acceptance: duplicate `buildManifest` key is removed.

Acceptance: `npm run lint:js` passes.

Acceptance: deliberately adding duplicate key fails.

## R11 - Clean rebuild gate

Action: clean checkout.

Action: install pinned Rust toolchain from `rust-toolchain.toml`.

Action: run `cargo fmt --check`.

Action: run `cargo test`.

Action: run `cargo clippy --target wasm32-unknown-unknown --release -- -D warnings`.

Action: run build script.

Action: extract generated embedded WASM.

Action: compare generated WASM hash to manifest expected hash.

Action: compare generated HTML hash to manifest expected hash when deterministic.

Acceptance: rebuilt WASM hash equals manifest hash.

Acceptance: if endpoint patch lands, baseline hash is intentionally updated and old V16 hash is retired.

Fail behavior: hash mismatch blocks release.

Fail behavior: cargo unavailable blocks local claim but not CI claim.

## R12 - Clean capture gate

Action: clean checkout or clean unzip.

Action: run `npm ci`.

Action: run `npx playwright install chromium`.

Action: run `npm run capture`.

Action: run `python3 verify_artifact.py --root mf_live_steel_review_pack --manifest mf_live_steel_review_pack/review_manifest.json --mode release`.

Acceptance: release capture uses Playwright-managed Chromium.

Acceptance: page errors zero.

Acceptance: proof repeat byte-identical within same environment.

Acceptance: alpha hashes match expected for same browser/font config.

Acceptance: proof gates recomputed true.

Fail behavior: any system Chrome capture becomes non-release evidence.

## R13 - Cross-platform replay gate

Matrix minimum: Windows and Linux.

Matrix preferred: Windows, Linux, macOS.

Browser source: Playwright-managed Chromium.

Viewport: `1280x720`.

Device scale factor: `1`.

Seed: fixed.

Font: bundled release font.

Required cross-platform exact match: WASM ABI values.

Required cross-platform exact match: alpha hashes, unless browser version differs by design.

Required within-environment exact match: proof repeat PNG hashes.

Required cross-platform threshold pass: coverage.

Required cross-platform threshold pass: hole preservation.

Required cross-platform threshold pass: no-glow metrics.

Required cross-platform threshold pass: temporal life.

Required cross-platform threshold pass: morph continuity.

Not required at first: PNG byte identity across OS.

Fail behavior: alpha mismatch blocks release until investigated.

Fail behavior: metrics pass but PNG bytes differ across OS is renderer variance, not physics failure.

## R14 - Performance gate redesign

Action: split sim-only and render performance.

Action: measure warmup separately.

Action: measure at least 15 sim-only runs.

Action: record p50, p90, p95, max.

Action: record target hardware class.

Action: record browser version.

Action: add render-only measure.

Action: add step+draw measure.

Sim-only prototype threshold: p95 <= `8.0 ms/step` on baseline host class.

Render prototype threshold: p95 <= `33.3 ms/frame`.

Production UI threshold: p95 <= `16.67 ms/frame`.

Current allowed claim: provided sim-only smoke pass.

Current forbidden claim: production 60 FPS.

## R15 - Release package contract

Package must include `src/lib.rs`.

Package must include `Cargo.toml`.

Package must include `Cargo.lock`.

Package must include `rust-toolchain.toml`.

Package must include `assets/LiveSteelProof.ttf` only if font license is proven.

Package must include `FONT_LICENSE.md`.

Package must include `template.html`.

Package must include `build.py`.

Package must include `capture_live_steel_proof.js`.

Package must include `verify_live_steel.py`.

Package must include `verify_artifact.py`.

Package must include `review_manifest.schema.json`.

Package must include `gate_config.json`.

Package must include `package.json`.

Package must include `package-lock.json`.

Package must include `review_manifest.json`.

Package must include `mf_proto_live_steel.html`.

Package must include `proof_stats.json`.

Package must include proof PNGs.

Package must include repeat PNGs.

Package must include morph PNGs.

Package must include debug PNGs.

Package must include generated contact sheets if they are declared in manifest.

Package must exclude `target/`.

Package must exclude `node_modules/`.

Package must exclude browser cache.

Package must exclude host absolute scratch directories.

Package must exclude secrets.

Package must exclude undistributable fonts.

Acceptance: unzip into arbitrary directory.

Acceptance: one verifier command passes.

Acceptance: one capture command regenerates pack.

Acceptance: one rebuild command regenerates HTML/WASM.

## R16 - CI workflow

Job static: run `python3 verify_live_steel.py`.

Job JS: run `npm ci`.

Job JS: run `npm run check:js`.

Job JS: run `npm run lint:js`.

Job Rust: run `cargo fmt --check`.

Job Rust: run `cargo test`.

Job Rust: run `cargo clippy --target wasm32-unknown-unknown --release -- -D warnings`.

Job build: run `python3 build.py --out mf_proto_live_steel.html --manifest review_manifest.json --font assets/LiveSteelProof.ttf --release`.

Job build: run `python3 verify_artifact.py --root . --manifest review_manifest.json --mode pre-capture`.

Job capture: run `npx playwright install chromium`.

Job capture: run `npm run capture`.

Job capture: run `python3 verify_artifact.py --root mf_live_steel_review_pack --manifest mf_live_steel_review_pack/review_manifest.json --mode release`.

Job package: create release zip.

Job package: compute zip SHA256.

Job package: write zip hash into release manifest or sidecar.

Job package: upload artifact.

Acceptance: no manual release when CI red.

## R17 - Claim ledger

Doc section `VERIFIED_HERE` lists only local facts.

Doc section `VERIFIED_IN_CI` lists CI facts with run id.

Doc section `ACCEPTED_FROM_AUTHOR_RUN` lists facts not reproduced by reviewer.

Doc section `NOT_CLAIMED` lists production FPS, cross-platform PNG identity, public release readiness.

Doc section `KNOWN_LIMITS` lists browser variance and font license boundary.

Acceptance: release notes cannot say production-ready unless render/integration gates pass.

Acceptance: release notes cannot say source-to-WASM proven unless clean rebuild hash passes.

Acceptance: release notes cannot say portable replay unless `npm ci` + Playwright-managed capture passes.

## R18 - Order of execution

Step 1: freeze V16 physics baseline.

Step 2: add `rust-toolchain.toml`.

Step 3: add `package.json` and lockfile.

Step 4: patch capture browser selection.

Step 5: patch font fail-closed behavior.

Step 6: patch endpoint raw-distance reach.

Step 7: add `gate_config.json`.

Step 8: refactor gate summary by frame/phase lookup.

Step 9: remove duplicate JS key.

Step 10: add lint gate.

Step 11: add manifest v2 schema.

Step 12: add independent verifier.

Step 13: run clean rebuild.

Step 14: run clean capture.

Step 15: run cross-platform replay.

Step 16: package V17-clean.

Step 17: write claim ledger.

## R19 - One-command release target

Target command:

```bash
python3 tools/release_live_steel.py --mode v17-clean --out live_steel_v17_clean.zip
```

Command must run static gates.

Command must run Rust gates.

Command must run build gate.

Command must run JS gates.

Command must run browser capture.

Command must run artifact verifier.

Command must create zip.

Command must print release SHA256.

Command must fail on first hard gate failure.

---

# VERDICT

## V0 - Да, лучше можно

Да.

Лучший план не переписывает V16.

Лучший план не добавляет новых visual tricks.

Лучший план превращает V16 из strong evidence bundle в reproducible release candidate.

## V1 - Главное отличие V17 от V16-плана

V16-план закрывал очевидную упаковку.

V17-план закрывает proof boundary.

V17 добавляет Rust toolchain enforcement.

V17 добавляет Playwright-managed release browser.

V17 запрещает silent host-font fallback.

V17 исправляет endpoint reach math.

V17 переносит thresholds в versioned config.

V17 требует independent verifier.

V17 требует JS lint gate.

V17 запрещает overclaim на production FPS.

## V2 - Единственный правильный next move

Implement R0 through R12 first.

Regenerate proof pack after endpoint patch.

Then run R13 cross-platform replay.

Then package V17-clean.

Only after V17-clean start visual tuning or product integration.

## V3 - Final ruling

V16 is alive steel.

V17 is the locked kiln.

Ship the kiln before forging more sparks. 🔩
