const crypto = require("crypto");
const path = require("path");
const { chromium } = require("playwright");

function argValue(name, fallback) {
  const prefix = `${name}=`;
  const hit = process.argv.find((arg) => arg.startsWith(prefix));
  return hit ? hit.slice(prefix.length) : fallback;
}

function fileUrl(filePath) {
  return `file:///${path.resolve(filePath).replace(/\\/g, "/")}`;
}

function requireCondition(condition, message) {
  if (!condition) throw new Error(message);
}

async function panelSnapshot(page) {
  return page.evaluate(() => window.__liveSteelPanel.snapshot());
}

async function fieldAlphaHash(page) {
  // FNV-1a over phrase-0 alpha bytes straight from wasm memory: the physics INPUT,
  // upstream of render noise. Same host + same params must reproduce it exactly.
  /* eslint-disable no-undef -- views/FIELD_MAX are page globals inside evaluate */
  return page.evaluate(() => {
    const v = views();
    const a = v.alpha.slice(0, FIELD_MAX);
    const bytes = new Uint8Array(a.buffer, a.byteOffset, a.byteLength);
    let h = 0x811c9dc5;
    for (let i = 0; i < bytes.length; i++) {
      h ^= bytes[i];
      h = Math.imul(h, 0x01000193) >>> 0;
    }
    return h.toString(16).padStart(8, "0") + ":" + bytes.length;
  });
  /* eslint-enable no-undef */
}

async function morphPhaseWalk(page, seed) {
  // Deterministic fast-forward through the template audit hooks — no wall-clock sleeps.
  /* eslint-disable no-undef, no-implicit-globals -- running/views are page-global lexical bindings inside evaluate */
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

async function waitForPanel(page) {
  await page.waitForFunction(
    () =>
      window.__liveSteelPanel &&
      window.__liveSteelPanel.snapshot &&
      window.__liveSteelPanel.snapshot().ready,
    null,
    { timeout: 30000 },
  );
}

async function applySinglePhraseProbe(page) {
  await page.evaluate(() => {
    window.__liveSteelPanel.applyPreset({ text: "codex trace", mode: "locked", autoLock: false });
  });
  const neutralHash = await fieldAlphaHash(page);
  const snapshot = await page.evaluate(() => {
    return window.__liveSteelPanel.setLetter(1, {
      rotation: -18,
      scale: 1.36,
      x: -10,
      y: 4,
      density: 1.32,
    });
  });
  requireCondition(snapshot.phrases.length === 1, "single phrase should stay single");
  requireCondition(snapshot.phrases[0] === "CODEX TRACE", `unexpected phrase: ${snapshot.phrases[0]}`);
  const letter = snapshot.state.letters[1];
  requireCondition(letter.rotation === -18, "per-letter rotation did not persist");
  requireCondition(letter.scale === 1.36, "per-letter scale did not persist");
  requireCondition(letter.density === 1.32, "per-letter density did not persist");
  const patchedHash = await fieldAlphaHash(page);
  requireCondition(
    patchedHash !== neutralHash,
    "per-letter patch had NO effect on the field alpha — state echoes but physics input unchanged",
  );
  const repatched = await page.evaluate(() =>
    window.__liveSteelPanel.setLetter(1, { rotation: -18, scale: 1.36, x: -10, y: 4, density: 1.32 }),
  );
  requireCondition(Boolean(repatched), "re-apply snapshot missing");
  const repatchedHash = await fieldAlphaHash(page);
  requireCondition(
    repatchedHash === patchedHash,
    `raster path is nondeterministic on same host: ${repatchedHash} != ${patchedHash}`,
  );
  return { snapshot, neutralHash, patchedHash };
}

async function applyMorphProbe(page) {
  const snapshot = await page.evaluate(() => {
    window.__liveSteelPanel.applyPreset({
      text: "iron memory|soft machine",
      mode: "morph",
      autoLock: false,
      seed: 7331,
      render: { trails: true, causal: true, glints: true },
    });
    return window.__liveSteelPanel.snapshot();
  });
  requireCondition(snapshot.phrases.length === 2, "morph probe should expose two phrases");
  requireCondition(snapshot.phrases[0] === "IRON MEMORY", `bad morph phrase 1: ${snapshot.phrases[0]}`);
  requireCondition(snapshot.phrases[1] === "SOFT MACHINE", `bad morph phrase 2: ${snapshot.phrases[1]}`);
  const disabled = await page.$eval("#ls-letter-section", (node) => node.classList.contains("is-disabled"));
  requireCondition(disabled, "per-letter section should disable for multi-phrase morph");
  const walk = await morphPhaseWalk(page, 7331);
  requireCondition(walk.seen.includes(0) && walk.seen.includes(1),
    `morph never transitioned between phrases: visited ${JSON.stringify(walk.seen)}`);
  requireCondition(walk.litRatio > 0.005, `stepped canvas too dark: litRatio=${walk.litRatio}`);
  return { snapshot, walk };
}

async function assertCanvasAlive(page) {
  await page.waitForTimeout(500);
  const stats = await page.evaluate(() => window.__pixelStats());
  requireCondition(stats.litRatio > 0.005, `canvas too dark: litRatio=${stats.litRatio}`);
  requireCondition(stats.maxRGB > 64, `canvas has weak signal: maxRGB=${stats.maxRGB}`);
  return stats;
}

async function main() {
  const html = argValue("--html", "live_steel_panel_built.html");
  const out = argValue("--out", "");
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
    requireCondition(/^live-steel-panel-v\d+$/.test(initial.version), `bad panel version: ${initial.version}`);

    const single = await applySinglePhraseProbe(page);
    const singleStats = await assertCanvasAlive(page);
    const morph = await applyMorphProbe(page);
    const morphStats = await assertCanvasAlive(page);
    const morphWalk = morph.walk;
    const dataUrl = await page.evaluate(() => window.__liveSteelPanel.exportDataUrl());
    requireCondition(dataUrl.startsWith("data:image/png;base64,"), "panel export did not return a PNG data URL");
    requireCondition(dataUrl.length > 100000, `panel export is suspiciously small: ${dataUrl.length}`);

    let screenshotHash = "";
    if (out) {
      const screenshot = await page.screenshot({ path: out });
      screenshotHash = crypto.createHash("sha256").update(screenshot).digest("hex");
    }

    requireCondition(pageErrors.length === 0, `page errors:\n${pageErrors.join("\n")}`);
    const result = {
      panel: initial.version,
      singlePhrase: single.snapshot.phrases[0],
      perLetterRotation: single.snapshot.state.letters[1].rotation,
      alphaNeutral: single.neutralHash,
      alphaPatched: single.patchedHash,
      morphPhrases: morph.snapshot.phrases,
      morphPhraseWalk: morphWalk.seen,
      steppedLitRatio: morphWalk.litRatio,
      singleLitRatio: singleStats.litRatio,
      morphLitRatio: morphStats.litRatio,
      exportBytesApprox: Math.floor((dataUrl.length * 3) / 4),
      screenshotHash,
    };
    console.log(JSON.stringify(result, null, 2));
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(`FAIL: ${error.message}`);
  process.exit(1);
});
