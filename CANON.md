# CANON — канонический хост Live Steel

Дата: 2026-07-04. Статус: ратифицировано первым зелёным `capture` в CI
(см. §4).

## 1. Определение

Канон — не машина, а рецепт. Identity артефакта (все хэши в
`review_manifest.json`) имеет право производить только рецепт ниже. Любая
конкретная коробка — включая ту, где проект родился — даёт origin evidence,
не identity.

Рецепт (все слои pinned файлами репо, не памятью оператора):

| слой | версия | enforce |
|---|---|---|
| ОС | Linux x86_64, класс `ubuntu-24.04` (GitHub Actions runner) | `.github/workflows/gates.yml` |
| Rust | rustc 1.95.0, target `wasm32-unknown-unknown` | `rust-toolchain.toml` |
| Node | 22.22.2 | `.nvmrc` |
| Playwright | 1.61.0 → Chromium headless shell v1228 | `package-lock.json` |
| Шрифт | bundled `assets/LiveSteelProof.woff2`, fail-closed | `build.py` |
| Пороги | `gate_config.json` (schema v1) | `verify_artifact.py` |

## 2. Почему Linux, а не Windows-бокс генезиса

1. **Git уже проголосовал.** `.gitattributes` (`* text=auto eol=lf`) переписал
   CRLF-байты виндового артефакта при первом же коммите. Байты, которые хэшировал
   старый манифест (`9d3f6c…`), в репо не заезжали никогда — канон-как-машина
   умер в момент публикации. Восстановить его из репо невозможно; воспроизвести
   можно только рецепт.
2. **Публичная воспроизводимость.** Любой читатель может перезапустить
   `gates.yml` и пересчитать identity. Виндовый бокс не может перезапустить никто.
3. **Направление remap.** `.cargo/config.toml` сворачивает `src\lib.rs` →
   `src/lib.rs` в panic-location строках: Windows-сборки сходятся к
   Linux-канонической идентичности, не наоборот.

## 3. Лестница идентичности (что чем является)

- **Host-invariant (identity):** source → WASM. MEASURED: Linux-сборка pinned
  1.95.0 отличалась от виндовой ровно на 1 байт из 125 283 (позиция 107982,
  `\` vs `/`); после remap-фикса расхождений быть не должно.
  Windows-подтверждение — MEASURED 2026-07-04: `cargo build` на боксе дал sha,
  равный `embedded_wasm_sha256` (коммит 7e0ad84).
- **Cross-host (пороговые гейты):** все 6 proof-гейтов зелёные и на Windows
  Chrome 149, и на Linux Chromium 149 (MEASURED оба). Perf p95: 5.26ms (win) /
  5.42ms (linux) при пороге 10.
- **Host-scoped (пиксельная):** `alphaHashes`, PNG-хэши. Растеризация глифов
  платформенная; пиксельная идентичность действительна только внутри канона.
  Внутри рецепта пиксельная идентичность — MEASURED 2026-07-04: два
  независимых Linux x64 инстанса (контейнер-кандидат ↔ runner, run
  `28714240201`) дали byte-equal HTML и 21/21 PNG; ручная сверка артефакта.
  Кросс-платформенное равенство пикселей НЕ заявляется.

## 4. Ратификация

Ратифицировано 2026-07-04: первый зелёный `workflow_dispatch` прогон job
`capture` в GitHub Actions на `ubuntu-24.04` — run `28714010254`, commit
`35bb03e`. Runner заново выполнил `build.py`, `capture_live_steel_proof.js`,
`verify_artifact.py --root review_pack --manifest review_manifest.json --mode
release`; `standalone_html_sha256`, `embedded_wasm_sha256` и все PNG байты
совпали с кандидатом, `png_hashes:21` и `recomputed_gates` прошли.

`proof_stats.json` не является byte-identity артефактом: он несёт путь checkout,
kernel/CPU и perf timing конкретного runner'а. В ратификационном прогоне он
отличался только этими host-local полями; пиксельного drift не было.

Начиная со следующего `capture`, расхождение уже не ручной ритуал, а tripwire:
`tools/verify_capture_candidate.py` сверяет свежий `review_pack` против
закоммиченного `proof_stats.json` по `htmlSha256` и 21 PNG sha256. Если будущий
runner даст иные пиксельные хэши, job падает; канон пересобирается из
артефактов того прогона (identity принадлежит рецепту, см. §1), а расхождение
фиксируется здесь, не замалчивается.

## 5. Открытые пункты (честный паркинг)

- **Causal audit красный на Linux: 9 из ~60 проверок.** Все девять — одно
  семейство shape-IoU против глифовой маски (static lattice 100/80/60% +
  mid-assembly); все механизменные проверки причинности (chaos
  non-preformation, provenance traces, teleport continuity, glyph-field poison,
  particle dependency, identity shuffle, render variants) — зелёные. Ключевой
  пример: mid primary shape4 iou = 0.8745 (win) / 0.7957 (linux) при пороге
  0.82 — виндовая маржа 0.054 съедена растром. Пороги НЕ перетюнены под
  зелёный. Решение политики за оператором; предложение: вынести THRESHOLDS из
  `tools/audit_causal_letters.cjs` в versioned config (закрыть D9 для аудита,
  как уже сделано для proof-гейтов) и деривировать пороги как
  min(два хоста) − объявленная маржа.
- **LICENSE кода не объявлена.** Шрифт — OFL 1.1; код по умолчанию
  all rights reserved, для публичного репо это должно быть сознательным выбором.
