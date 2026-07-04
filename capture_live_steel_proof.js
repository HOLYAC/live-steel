const fs = require("fs");
const os = require("os");
const path = require("path");
const crypto = require("crypto");
const { pathToFileURL } = require("url");
const { chromium } = require("playwright");

const ROOT = __dirname;
const DEFAULT_HTML = path.resolve(ROOT, "mf_proto_live_steel.html");
const DEFAULT_OUT = path.resolve(ROOT, "mf_live_steel_review_pack");
const DEFAULT_MANIFEST = path.resolve(ROOT, "review_manifest.json");
const DEFAULT_GATE_CONFIG = path.resolve(ROOT, "gate_config.json");

const RELEASE_FILES = [
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  "package.json",
  "package-lock.json",
  "eslint.config.cjs",
  "gate_config.json",
  "review_manifest.schema.json",
  "FONT_LICENSE.md",
  "build.py",
  "capture_live_steel_proof.js",
  "verify_live_steel.py",
  "verify_artifact.py",
  "template.html",
  "mf_proto_live_steel.html",
  "src/lib.rs",
  "assets/LiveSteelProof.woff2",
  "tools/audit_causal_letters.cjs",
  "tools/check_template_js.cjs",
  "tools/release_live_steel.py",
];

function argValue(name, fallback) {
  const prefix = `${name}=`;
  const arg = process.argv.find((item) => item.startsWith(prefix));
  return arg ? arg.slice(prefix.length) : fallback;
}

function hasFlag(name) {
  return process.argv.includes(name);
}

function existingPath(candidate) {
  return candidate && fs.existsSync(candidate) ? candidate : null;
}

function browserExecutable({ releaseBrowser }) {
  const explicit = argValue("--chrome", process.env.CHROME_PATH || "");
  if (releaseBrowser && explicit) {
    throw new Error("release capture forbids --chrome/CHROME_PATH");
  }
  if (!explicit) {
    return null;
  }
  const found = existingPath(explicit);
  if (!found) {
    throw new Error(`explicit Chrome path does not exist: ${explicit}`);
  }
  return found;
}

function imageBuffer(dataUrl) {
  const marker = "base64,";
  const idx = dataUrl.indexOf(marker);
  if (idx < 0) {
    throw new Error("capture returned non-base64 data URL");
  }
  return Buffer.from(dataUrl.slice(idx + marker.length), "base64");
}

function sha256(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function sha256File(file) {
  return sha256(fs.readFileSync(file));
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function savePng(outDir, name, dataUrl) {
  const buffer = imageBuffer(dataUrl);
  fs.writeFileSync(path.join(outDir, name), buffer);
  return { file: name, sha256: sha256(buffer), bytes: buffer.length };
}

function hashFloat32(values) {
  const buffer = Buffer.alloc(values.length * 4);
  values.forEach((value, idx) => buffer.writeFloatLE(value, idx * 4));
  return sha256(buffer);
}

function readManifest(manifestPath) {
  if (!fs.existsSync(manifestPath)) {
    return null;
  }
  return readJson(manifestPath);
}

function exactlyOne(items, predicate, label) {
  const matches = items.filter(predicate);
  if (matches.length !== 1) {
    throw new Error(`expected exactly one ${label}, got ${matches.length}`);
  }
  return matches[0];
}

function copyReleaseFiles(outDir) {
  for (const relative of RELEASE_FILES) {
    const src = path.join(ROOT, relative);
    if (!fs.existsSync(src)) {
      continue;
    }
    const dest = path.join(outDir, relative);
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    fs.copyFileSync(src, dest);
  }
}

async function captureProof(page, outDir, prefix) {
  const frames = await page.evaluate(() => window.__captureProofSet());
  return frames.map((frame) => {
    const name = `${prefix}_${String(frame.frame).padStart(4, "0")}.png`;
    const image = savePng(outDir, name, frame.dataUrl);
    return {
      frame: frame.frame,
      image,
      report: frame.report,
      pixel: frame.pixel,
      coverage: frame.coverage,
    };
  });
}

async function captureDebug(page, outDir, dwellFrame) {
  const labels = {
    1: "alpha",
    2: "sdf",
    3: "tangent",
    4: "activation",
    5: "target_density",
    6: "current_density",
    7: "density_void",
    8: "chain",
    9: "density_pressure",
  };
  const frames = await page.evaluate((frame) => window.__captureDebugSet(frame), dwellFrame);
  return frames.map((frame) => {
    const name = `debug_${String(frame.mode).padStart(2, "0")}_${labels[frame.mode]}.png`;
    return { mode: frame.mode, label: labels[frame.mode], image: savePng(outDir, name, frame.dataUrl) };
  });
}

async function captureMorph(page, outDir, targets) {
  let done = 0;
  await page.evaluate(() => {
    window.__debugField = 0;
    window.__reset();
  });
  const frames = [];
  for (const frame of targets) {
    await page.evaluate((steps) => window.__steps(steps), frame - done);
    done = frame;
    const payload = await page.evaluate(() => ({
      dataUrl: window.__shot(),
      report: window.__perf(),
      pixel: window.__pixelStats(),
      coverage: window.__coverage(),
      activation: window.__activationSums(),
    }));
    const name = `morph_${String(frame).padStart(4, "0")}.png`;
    frames.push({
      frame,
      image: savePng(outDir, name, payload.dataUrl),
      report: payload.report,
      pixel: payload.pixel,
      coverage: payload.coverage,
      activation: payload.activation,
    });
  }
  return frames;
}

async function motionAt(page, frame, steps) {
  return page.evaluate(
    ({ frame, steps }) => {
      window.__debugField = 0;
      window.__reset();
      window.__steps(frame);
      return window.__motionProbe(steps);
    },
    { frame, steps }
  );
}

async function alphaHashes(page) {
  return page.evaluate(() => {
    return window.__abi().phrases.map((phrase, phraseIdx) => ({
      phrase,
      values: Array.from(window.__field(0, phraseIdx)),
    }));
  }).then((entries) =>
    entries.map((entry) => ({
      phrase: entry.phrase,
      sha256: hashFloat32(entry.values),
      cells: entry.values.length,
    }))
  );
}

function deterministicReport(firstRun, secondRun) {
  return firstRun.map((frame) => {
    const repeat = exactlyOne(secondRun, (item) => item.frame === frame.frame, `repeat frame ${frame.frame}`);
    return {
      frame: frame.frame,
      first: frame.image.sha256,
      second: repeat.image.sha256,
      equal: frame.image.sha256 === repeat.image.sha256,
    };
  });
}

function percentile(values, p) {
  const sorted = values.slice().sort((a, b) => a - b);
  const pos = (p / 100) * (sorted.length - 1);
  const lo = Math.floor(pos);
  const hi = Math.ceil(pos);
  const t = pos - lo;
  return sorted[lo] + (sorted[hi] - sorted[lo]) * t;
}

function gateSummary(stats, config) {
  const thresholds = config.thresholds;
  const dwellFrame = config.proofFrames[config.proofFrames.length - 1];
  const dwell = exactlyOne(
    stats.proof,
    (item) => item.frame === dwellFrame && item.report.phase === "dwell",
    `dwell proof frame ${dwellFrame}`
  );
  const morphFirstFrame = config.morphFrames[0];
  const morphLastFrame = config.morphFrames[config.morphFrames.length - 1];
  const morphFirst = exactlyOne(stats.morph, (item) => item.frame === morphFirstFrame, `morph frame ${morphFirstFrame}`);
  const morphLast = exactlyOne(stats.morph, (item) => item.frame === morphLastFrame, `morph frame ${morphLastFrame}`);
  const activation0Falls = morphLast.activation[0].sum < morphFirst.activation[0].sum;
  const activation1Rises = morphLast.activation[1].sum > morphFirst.activation[1].sum;
  const perfValues = stats.simOnly.map((run) => run.msPerStep);
  const perfMax = Math.max(...perfValues);
  const perfP95 = percentile(perfValues, 95);
  return {
    deterministic: stats.determinism.every((item) => item.equal),
    noPageErrors: stats.pageErrors.length === 0,
    denseStrokes:
      dwell.coverage.all.coveredRatio >= thresholds.dwellCoveredRatioMin &&
      dwell.coverage.all.holePreservation >= thresholds.dwellHolePreservationMin,
    readableMetalTypography:
      dwell.pixel.over128Ratio <= thresholds.dwellOver128RatioMax &&
      dwell.pixel.over180Ratio <= thresholds.dwellOver180RatioMax &&
      dwell.pixel.blueDominantRatio <= thresholds.dwellBlueDominantRatioMax &&
      dwell.report.jsGlintRatio >= thresholds.dwellJsGlintRatioMin &&
      dwell.report.jsGlintRatio <= thresholds.dwellJsGlintRatioMax,
    temporalLife:
      stats.temporal.dwell1.active.avg > thresholds.temporalDwell1ActiveAvgMin &&
      stats.temporal.dwell1.active.p95 < thresholds.temporalDwell1ActiveP95Max &&
      stats.temporal.morph1.active.p95 < thresholds.temporalMorph1ActiveP95Max,
    morphContinuity: activation0Falls && activation1Rises && morphLast.report.avgActive > thresholds.morphLastAvgActiveMin,
    simOnlyPerfMaxMs: +perfMax.toFixed(4),
    simOnlyPerfP95Ms: +perfP95.toFixed(4),
    simOnlyPerfWithin8ms: perfP95 <= thresholds.simOnlyP95MsPerStepMax,
  };
}

async function main() {
  const html = path.resolve(argValue("--html", DEFAULT_HTML));
  const outDir = path.resolve(argValue("--out", DEFAULT_OUT));
  const manifestPath = path.resolve(argValue("--manifest", DEFAULT_MANIFEST));
  const gateConfigPath = path.resolve(argValue("--gate-config", DEFAULT_GATE_CONFIG));
  const releaseBrowser = hasFlag("--release-browser");
  const gateConfig = readJson(gateConfigPath);
  const gateConfigSha256 = sha256File(gateConfigPath);
  const dwellFrame = gateConfig.proofFrames[gateConfig.proofFrames.length - 1];

  fs.rmSync(outDir, { recursive: true, force: true });
  fs.mkdirSync(outDir, { recursive: true });
  copyReleaseFiles(outDir);

  const executablePath = browserExecutable({ releaseBrowser });
  const browserSource = executablePath ? "system-chrome-nonrelease" : "playwright-managed";
  const launchOptions = {
    headless: true,
    args: ["--allow-file-access-from-files", "--disable-gpu-vsync"],
  };
  if (executablePath) {
    launchOptions.executablePath = executablePath;
  }

  const pageErrors = [];
  const browser = await chromium.launch(launchOptions);
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 1 });
    page.on("pageerror", (err) => pageErrors.push(err.stack || err.message));
    page.on("console", (msg) => {
      if (msg.type() === "error") pageErrors.push(msg.text());
    });

    await page.goto(pathToFileURL(html).href, { waitUntil: "load" });
    await page.waitForFunction(
      () =>
        typeof window.__captureProofSet === "function" &&
        typeof window.__captureDebugSet === "function" &&
        typeof window.__shot === "function" &&
        typeof window.__motionProbe === "function" &&
        typeof window.__simOnly === "function" &&
        typeof window.__abi === "function",
      { timeout: 15000 }
    );
    await page.evaluate(() => window.__pause());
    const abi = await page.evaluate(() => window.__abi());
    const alpha = await alphaHashes(page);
    await page.evaluate(() => window.__captureProofSet());

    const proof = await captureProof(page, outDir, "proof");
    const proofRepeat = await captureProof(page, outDir, "proof_repeat");
    const debug = await captureDebug(page, outDir, dwellFrame);
    const morph = await captureMorph(page, outDir, gateConfig.morphFrames);
    const temporal = {
      dwell1: await motionAt(page, dwellFrame, 1),
      dwell6: await motionAt(page, dwellFrame, 6),
      morph1: await motionAt(page, gateConfig.morphFrames[2], 1),
      morph6: await motionAt(page, gateConfig.morphFrames[2], 6),
    };
    const simOnlyWarmup = [];
    for (let idx = 0; idx < (gateConfig.simOnlyWarmupRuns || 0); idx++) {
      simOnlyWarmup.push(await page.evaluate((frames) => window.__simOnly(frames), gateConfig.simOnlyFramesPerRun));
    }
    const simOnly = [];
    for (let idx = 0; idx < gateConfig.simOnlyMeasuredRuns; idx++) {
      simOnly.push(await page.evaluate((frames) => window.__simOnly(frames), gateConfig.simOnlyFramesPerRun));
    }

    const buildManifest = readManifest(manifestPath);
    const releaseEligible = Boolean(buildManifest?.release_eligible) && releaseBrowser && browserSource === "playwright-managed";
    const stats = {
      html,
      htmlSha256: sha256File(html),
      manifestPath,
      buildManifest,
      gateConfig,
      gateConfigSha256,
      captureCommand: process.argv.slice(),
      releaseEligible,
      browser: {
        name: "chromium",
        version: browser.version(),
        source: browserSource,
        executablePath: executablePath || "playwright-managed",
      },
      os: `${os.type()} ${os.release()} ${os.arch()}`,
      cpu: os.cpus()[0]?.model || "unknown",
      viewport: { width: 1280, height: 720, deviceScaleFactor: 1 },
      abi,
      alphaHashes: alpha,
      pageErrors,
      proof,
      proofRepeat,
      determinism: deterministicReport(proof, proofRepeat),
      debug,
      morph,
      temporal,
      simOnlyWarmup,
      simOnly,
    };
    stats.gates = gateSummary(stats, gateConfig);

    const statsPath = path.join(outDir, "proof_stats.json");
    fs.writeFileSync(statsPath, JSON.stringify(stats, null, 2));
    const finalManifest = {
      ...(buildManifest || {}),
      release_eligible: releaseEligible,
      alpha_hashes: alpha,
      capture: {
        command: stats.captureCommand,
        release_browser: releaseBrowser,
        release_eligible: releaseEligible,
        os: stats.os,
        cpu: stats.cpu,
      },
    };
    finalManifest.hashes = {
      ...(buildManifest?.hashes || {}),
      proof_stats_sha256: sha256File(statsPath),
      gate_config_sha256: gateConfigSha256,
    };
    finalManifest.toolchain = {
      ...(buildManifest?.toolchain || {}),
      browser_source: browserSource,
      browser_version: browser.version(),
    };
    fs.writeFileSync(path.join(outDir, "review_manifest.json"), JSON.stringify(finalManifest, null, 2));
    console.log(JSON.stringify(stats.gates, null, 2));
  } finally {
    await browser.close();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
