# CANON — канонический хост Live Steel

Дата: 2026-07-04. Статус: кандидат до первой ратификации в CI (см. §4).

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
  Windows-подтверждение — PENDING (прогнать `cargo build` на боксе, сверить
  `embedded_wasm_sha256`).
- **Cross-host (пороговые гейты):** все 6 proof-гейтов зелёные и на Windows
  Chrome 149, и на Linux Chromium 149 (MEASURED оба). Perf p95: 5.26ms (win) /
  5.42ms (linux) при пороге 10.
- **Host-scoped (пиксельная):** `alphaHashes`, PNG-хэши. Растеризация глифов
  платформенная; пиксельная идентичность действительна только внутри канона.
  Кросс-платформенное равенство пикселей НЕ заявляется.

## 4. Ратификация

Текущий `review_manifest.json` собран на Linux x86_64 с точно pinned стеком §1
и является **кандидатом**. Первый зелёный прогон job `capture` в GitHub Actions
на `ubuntu-24.04` — ратификация. Если runner даст иные пиксельные хэши, чем
кандидат, канон пересобирается из артефактов того прогона (identity принадлежит
рецепту, см. §1) — расхождение фиксируется здесь, не замалчивается.

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
- **Windows-сторона remap-фикса не измерена** (нет виндовой машины в этом
  прогоне). Один `cargo build` на боксе закрывает.
- **LICENSE кода не объявлена.** Шрифт — OFL 1.1; код по умолчанию
  all rights reserved, для публичного репо это должно быть сознательным выбором.
