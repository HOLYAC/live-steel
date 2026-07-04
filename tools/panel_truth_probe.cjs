const crypto = require("crypto");
const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

const PROBE_SCHEMA = "live-steel-panel-truth-probe-v1";
const SINGLE_TEXT = "codex trace";
const MORPH_TEXT = "iron memory|soft machine";
const MORPH_SEED = 7331;
const SIGNAL_MIN_LIT_RATIO = 0.005;
const SIGNAL_MIN_MAX_RGB = 64;

function argValue(name, fallback = "") {
  const prefix = `${name}=`;
  const hit = process.argv.find((arg) => arg.startsWith(prefix));
  return hit ? hit.slice(prefix.length) : fallback;
}

function fileUrl(filePath) {
  return `file:///${path.resolve(filePath).replace(/\\/g, "/")}`;
}

function fail(message) {
  throw new Error(message);
}

function requireCondition(condition, message) {
  if (!condition) fail(message);
}

function requireEqual(label, actual, expected) {
  requireCondition(
    actual === expected,
    `${label}: got ${JSON.stringify(actual)}, expected ${JSON.stringify(expected)}`,
  );
}

function requireArrayEqual(label, actual, expected) {
  requireCondition(Array.isArray(actual), `${label}: actual is not an array`);
  requireCondition(Array.isArray(expected), `${label}: expected is not an array`);
  requireEqual(`${label}.length`, actual.length, expected.length);
  for (let i = 0; i < expected.length; i++) requireEqual(`${label}[${i}]`, actual[i], expected[i]);
}

function readJson(filePath, label) {
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch (error) {
    fail(`${label} read failed (${filePath}): ${error.message}`);
  }
}

function verifyCandidate(result, candidatePath) {
  const candidate = readJson(candidatePath, "panel truth candidate");
  requireEqual("candidate schema", candidate.schema, "live-steel-panel-truth-candidate-v1");
  const expected = candidate.expected || {};
  if (expected.alphaNeutral) requireEqual("alphaNeutral", result.alpha.neutral, expected.alphaNeutral);
  if (expected.alphaAggregate) requireEqual("alphaAggregate", result.alpha.aggregate, expected.alphaAggregate);
  if (expected.renderInvariantAlpha) requireEqual("renderInvariantAlpha", result.alpha.renderInvariant, expected.renderInvariantAlpha);
  if (expected.textStyleScaleAlpha) requireEqual("textStyleScaleAlpha", result.alpha.textStyleScale, expected.textStyleScaleAlpha);
  if (expected.morphPhraseWalk) requireArrayEqual("morphPhraseWalk", result.morph.walk, expected.morphPhraseWalk);
  if (typeof expected.steppedLitRatio === "number") {
    const tolerance = Number((candidate.tolerance || {}).steppedLitRatio) || 0;
    requireCondition(
      Math.abs(result.morph.steppedLitRatio - expected.steppedLitRatio) <= tolerance,
      `steppedLitRatio: got ${result.morph.steppedLitRatio}, expected ${expected.steppedLitRatio} ± ${tolerance}`,
    );
  }
}

async function panelSnapshot(page) {
  return page.evaluate(() => window.__liveSteelPanel.snapshot());
}

async function waitForPanel(page) {
  await page.waitForFunction(
    () =>
      window.__liveSteelPanel &&
      typeof window.__liveSteelPanel.snapshot === "function" &&
      window.__liveSteelPanel.snapshot().ready,
    null,
    { timeout: 30000 },
  );
}

async function waitForCanvasSignal(page, label) {
  try {
    await page.waitForFunction(
      ([minLit, minRgb]) => {
        if (!window.__pixelStats) return false;
        const stats = window.__pixelStats();
        return stats.litRatio > minLit && stats.maxRGB > minRgb;
      },
      [SIGNAL_MIN_LIT_RATIO, SIGNAL_MIN_MAX_RGB],
      { timeout: 5000 },
    );
  } catch (error) {
    const stats = await page.evaluate(() => (window.__pixelStats ? window.__pixelStats() : null));
    fail(`${label} canvas never reached signal floor: ${JSON.stringify(stats)} (${error.message})`);
  }
}

async function fieldStats(page) {
  // Prefer a future first-class panel API, but keep this probe compatible with v2/v3.
  /* eslint-disable no-undef -- views/FIELD_* live inside the standalone page */
  return page.evaluate(() => {
    if (window.__liveSteelPanel && typeof window.__liveSteelPanel.fieldStats === "function") {
      return window.__liveSteelPanel.fieldStats();
    }
    const v = views();
    const alpha = v.alpha.slice(0, FIELD_MAX);
    const bytes = new Uint8Array(alpha.buffer, alpha.byteOffset, alpha.byteLength);
    let h = 0x811c9dc5;
    let mass = 0;
    let active = 0;
    let peak = 0;
    let minX = FIELD_W;
    let minY = FIELD_H;
    let maxX = -1;
    let maxY = -1;
    for (let i = 0; i < bytes.length; i++) {
      h ^= bytes[i];
      h = Math.imul(h, 0x01000193) >>> 0;
    }
    for (let i = 0; i < alpha.length; i++) {
      const value = alpha[i];
      mass += value;
      if (value > peak) peak = value;
      if (value > 0.12) {
        active += 1;
        const x = i % FIELD_W;
        const y = (i / FIELD_W) | 0;
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
    }
    return {
      alphaHash: h.toString(16).padStart(8, "0") + ":" + bytes.length,
      alphaBytes: bytes.length,
      alphaCells: alpha.length,
      activeCells: active,
      mass: Number(mass.toFixed(6)),
      peak: Number(peak.toFixed(6)),
      bounds: active ? { minX, minY, maxX, maxY } : null,
      phraseIdx: v.perf ? v.perf[2] | 0 : null,
      pixel: window.__pixelStats ? window.__pixelStats() : null,
    };
  });
  /* eslint-enable no-undef */
}

async function resetSingle(page, extra = {}) {
  const preset = Object.assign(
    {
      text: SINGLE_TEXT,
      mode: "locked",
      autoLock: false,
      seed: 2246,
      selectedLetter: 0,
      letters: [],
      textStyle: {
        uppercase: true,
        scale: 1,
        tracking: 1,
        wordGap: 1,
        lineGap: 1,
        x: 0,
        y: 0,
        lineBreak: "auto",
      },
    },
    extra,
  );
  await page.evaluate((next) => window.__liveSteelPanel.applyPreset(next), preset);
  const snapshot = await panelSnapshot(page);
  requireCondition(snapshot.phrases.length === 1, "single phrase should stay single");
  requireEqual("single phrase", snapshot.phrases[0], "CODEX TRACE");
  return snapshot;
}

async function setLetter(page, index, patch) {
  return page.evaluate(
    ({ i, p }) => window.__liveSteelPanel.setLetter(i, p),
    { i: index, p: patch },
  );
}

async function requireLetterEffect(page, neutralHash, probe) {
  await resetSingle(page);
  const snapshot = await setLetter(page, probe.index, probe.patch);
  for (const [key, value] of Object.entries(probe.patch)) {
    requireEqual(`${probe.name}.${key} persisted`, snapshot.state.letters[probe.index][key], value);
  }
  const stats = await fieldStats(page);
  requireCondition(
    stats.alphaHash !== neutralHash,
    `${probe.name} per-letter patch had NO effect on the field alpha`,
  );
  return stats;
}

async function singleTruth(page) {
  await resetSingle(page);
  const neutral = await fieldStats(page);
  requireCondition(neutral.alphaBytes > 0, "neutral alpha buffer is empty");

  const probes = [
    { name: "scale", index: 1, patch: { scale: 1.36 } },
    { name: "x", index: 1, patch: { x: -10 } },
    { name: "y", index: 1, patch: { y: 4 } },
    { name: "rotation", index: 3, patch: { rotation: -23 } },
    { name: "spacing", index: 1, patch: { spacing: 14 } },
    { name: "density", index: 1, patch: { density: 1.32 } },
    { name: "hidden", index: 3, patch: { hidden: true } },
  ];
  const perKnob = {};
  for (const probe of probes) perKnob[probe.name] = await requireLetterEffect(page, neutral.alphaHash, probe);

  await resetSingle(page);
  const aggregateSnapshot = await setLetter(page, 1, {
    rotation: -18,
    scale: 1.36,
    x: -10,
    y: 4,
    density: 1.32,
  });
  const aggregate = await fieldStats(page);
  requireCondition(
    aggregate.alphaHash !== neutral.alphaHash,
    "aggregate per-letter patch had NO effect on the field alpha — state echoes but physics input unchanged",
  );
  await setLetter(page, 1, { rotation: -18, scale: 1.36, x: -10, y: 4, density: 1.32 });
  const aggregateAgain = await fieldStats(page);
  requireEqual("same-host aggregate determinism", aggregateAgain.alphaHash, aggregate.alphaHash);

  await resetSingle(page, {
    render: {
      glints: false,
      shadows: false,
      dust: false,
      trails: false,
      causal: false,
      brightness: 1.21,
      contrast: 1.17,
      backgroundFade: 0.71,
    },
  });
  const renderInvariant = await fieldStats(page);
  requireEqual("render controls must not alter field alpha", renderInvariant.alphaHash, neutral.alphaHash);

  await resetSingle(page, { textStyle: { scale: 1.18 } });
  const textStyleScale = await fieldStats(page);
  requireCondition(
    textStyleScale.alphaHash !== neutral.alphaHash,
    "textStyle.scale had NO effect on the field alpha",
  );

  await waitForCanvasSignal(page, "single truth");
  return { neutral, aggregate, aggregateAgain, renderInvariant, textStyleScale, perKnob, snapshot: aggregateSnapshot };
}

async function morphPhaseWalk(page, seed) {
  // Deterministic fast-forward through template audit hooks. No blind sleeps.
  /* eslint-disable no-undef, no-implicit-globals -- page-global audit hooks */
  return page.evaluate((walkSeed) => {
    running = false;
    window.__reset(walkSeed | 0);
    const phraseIdx = () => views().perf[2] | 0;
    const seen = [phraseIdx()];
    for (let block = 0; block < 12; block++) {
      window.__steps(60);
      const idx = phraseIdx();
      if (seen[seen.length - 1] !== idx) seen.push(idx);
    }
    const pixel = window.__pixelStats();
    return { seen, litRatio: pixel.litRatio, maxRGB: pixel.maxRGB };
  }, seed);
  /* eslint-enable no-undef, no-implicit-globals */
}

async function morphTruth(page) {
  await page.evaluate((preset) => window.__liveSteelPanel.applyPreset(preset), {
    text: MORPH_TEXT,
    mode: "morph",
    autoLock: false,
    seed: MORPH_SEED,
    letters: [],
    render: { trails: true, causal: true, glints: true },
  });
  const snapshot = await panelSnapshot(page);
  requireEqual("morph phrase count", snapshot.phrases.length, 2);
  requireEqual("morph phrase 0", snapshot.phrases[0], "IRON MEMORY");
  requireEqual("morph phrase 1", snapshot.phrases[1], "SOFT MACHINE");

  const disabled = await page.$eval("#ls-letter-section", (node) => ({
    classDisabled: node.classList.contains("is-disabled"),
    ariaDisabled: node.getAttribute("aria-disabled"),
    controlsDisabled: Array.from(node.querySelectorAll("select,input,button")).every((el) => el.disabled),
  }));
  requireCondition(disabled.classDisabled, "per-letter section should visually disable for multi-phrase morph");
  requireEqual("per-letter aria-disabled", disabled.ariaDisabled, "true");
  requireCondition(disabled.controlsDisabled, "per-letter controls should be actually disabled under multi-phrase morph");

  const before = await fieldStats(page);
  const beforeLetter = JSON.stringify(snapshot.state.letters[1]);
  const blocked = await setLetter(page, 1, { rotation: 90, scale: 2.2, spacing: 18, density: 0.2 });
  const after = await fieldStats(page);
  requireEqual("blocked multi-phrase state", JSON.stringify(blocked.state.letters[1]), beforeLetter);
  requireEqual("blocked multi-phrase alpha", after.alphaHash, before.alphaHash);

  const walk = await morphPhaseWalk(page, MORPH_SEED);
  requireCondition(
    walk.seen.includes(0) && walk.seen.includes(1),
    `morph never transitioned between phrases: visited ${JSON.stringify(walk.seen)}`,
  );
  requireCondition(walk.litRatio > SIGNAL_MIN_LIT_RATIO, `stepped canvas too dark: litRatio=${walk.litRatio}`);
  requireCondition(walk.maxRGB > SIGNAL_MIN_MAX_RGB, `stepped canvas has weak signal: maxRGB=${walk.maxRGB}`);

  await waitForCanvasSignal(page, "morph truth");
  return { snapshot, disabled, before, after, walk };
}

async function exportTruth(page, out) {
  const dataUrl = await page.evaluate(() => window.__liveSteelPanel.exportDataUrl());
  requireCondition(dataUrl.startsWith("data:image/png;base64,"), "panel export did not return a PNG data URL");
  requireCondition(dataUrl.length > 100000, `panel export is suspiciously small: ${dataUrl.length}`);
  let screenshotHash = "";
  if (out) {
    const screenshot = await page.screenshot({ path: out });
    screenshotHash = crypto.createHash("sha256").update(screenshot).digest("hex");
  }
  return { exportBytesApprox: Math.floor((dataUrl.length * 3) / 4), screenshotSha256NonGating: screenshotHash };
}

async function main() {
  const html = argValue("--html", "live_steel_panel_built.html");
  const out = argValue("--out", "");
  const jsonOut = argValue("--json", "");
  const candidate = argValue("--candidate", "");
  const pageErrors = [];
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 1 });
    page.on("pageerror", (error) => pageErrors.push(String(error)));
    page.on("console", (message) => {
      if (message.type() === "error") pageErrors.push(message.text());
    });
    await page.goto(fileUrl(html));
    await waitForPanel(page);
    const initial = await panelSnapshot(page);
    requireCondition(/^live-steel-panel-v\d+/.test(initial.version), `bad panel version: ${initial.version}`);

    const single = await singleTruth(page);
    const morph = await morphTruth(page);
    const exported = await exportTruth(page, out);
    requireCondition(pageErrors.length === 0, `page errors:\n${pageErrors.join("\n")}`);

    const result = {
      schema: PROBE_SCHEMA,
      panel: initial.version,
      browser: browser.version(),
      singlePhrase: single.snapshot.phrases[0],
      perLetterRotation: single.snapshot.state.letters[1].rotation,
      alpha: {
        neutral: single.neutral.alphaHash,
        aggregate: single.aggregate.alphaHash,
        aggregateAgain: single.aggregateAgain.alphaHash,
        renderInvariant: single.renderInvariant.alphaHash,
        textStyleScale: single.textStyleScale.alphaHash,
        perKnob: Object.fromEntries(Object.entries(single.perKnob).map(([key, stats]) => [key, stats.alphaHash])),
      },
      alphaStats: {
        neutral: single.neutral,
        aggregate: single.aggregate,
        renderInvariant: single.renderInvariant,
        textStyleScale: single.textStyleScale,
      },
      morph: {
        phrases: morph.snapshot.phrases,
        disabled: morph.disabled,
        blockedAlphaBefore: morph.before.alphaHash,
        blockedAlphaAfter: morph.after.alphaHash,
        walk: morph.walk.seen,
        steppedLitRatio: morph.walk.litRatio,
      },
      export: exported,
    };
    if (candidate) verifyCandidate(result, candidate);
    if (jsonOut) fs.writeFileSync(jsonOut, JSON.stringify(result, null, 2) + "\n", "utf8");
    console.log(JSON.stringify(result, null, 2));
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(`FAIL: ${error.message}`);
  process.exit(1);
});
