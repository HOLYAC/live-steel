const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

const ROOT = path.resolve(__dirname, "..");
const THRESHOLDS = {
  slotToSupportRatioMin: 0.98,
  materialRatioMin: 0.74,
  weightedMaterialRatioMin: 0.78,
  weightedLockedRatioMin: 0.58,
  weightedCapturedRatioMin: 0.96,
  weightedTracedRatioMin: 0.94,
  weightedRotatedRatioMin: 0.94,
  weightedStateLockedRatioMin: 0.90,
  distanceP90Max: 30,
  activeDistanceP90Max: 24,
  angleP90Max: 1.05,
  birthToSlotP50Min: 250,
  traceToBirthP50Min: 0.95,
  rotationP50Min: 1.60,
  rotationP90Min: 2.80,
  captureTimeP95Max: 3.9,
  coverageMin: 0.74,
  holePreservationMin: 0.975,
  chaosCoverageMax: 0.12,
  chaosStrongCoverageMax: 0.04,
  chaosPrimaryShape4IouMax: 0.10,
  chaosAllShape4IouMax: 0.10,
  chaosLockedRatioMax: 0.01,
  chaosCapturedRatioMax: 0.02,
  chaosTracedRatioMax: 0.02,
  chaosPixelLitRatioMin: 0.012,
  chaosPixelLitRatioMax: 0.024,
  chaosBirthVisiblePrimaryRatioMin: 0.96,
  chaosBirthOnlyPrimaryRatioMin: 0.72,
  chaosBirthLetterMaterialRatioMax: 0.16,
  earlyCoverageMin: 0.18,
  earlyCapturedMin: 0.12,
  earlyCausalTraceCountMin: 90,
  earlyCausalTraceP95Min: 150,
  midCoverageMin: 0.60,
  midWeightedLockedMin: 0.55,
  midWeightedCapturedMin: 0.58,
  midWeightedTracedMin: 0.65,
  midWeightedRotatedMin: 0.92,
  midPrimaryCoverageMin: 0.90,
  midPrimaryShape4IouMin: 0.82,
  midPrimaryMinLetterCoverageMin: 0.80,
  midPrimaryMinLetterStrongMin: 0.75,
  midPrimaryMinLetterShape2IouMin: 0.70,
  midSecondaryCoverageMax: 0.30,
  midSecondaryShape4IouMax: 0.12,
  midMotionTrailCountMin: 380,
  midCausalTraceCountMin: 350,
  midCausalTraceP95Min: 300,
  finalMotionTrailCountMax: 80,
  finalCausalTraceCountMax: 12,
  finalPrimaryCoverageMin: 0.80,
  finalPrimaryStrongMin: 0.75,
  finalPrimaryShape4IouMin: 0.84,
  finalPrimaryHolePreservationMin: 0.97,
  finalPrimaryMinLetterCoverageMin: 0.80,
  finalPrimaryMinLetterStrongMin: 0.74,
  finalPrimaryMinLetterShape2IouMin: 0.82,
  finalPrimaryMaxSpaceIntrusionMax: 0.02,
  finalSecondaryCoverageMax: 0.35,
  finalSecondaryStrongMax: 0.30,
  finalSecondaryShape4IouMax: 0.12,
  finalSecondaryMaxLetterCoverageMax: 0.08,
  finalSecondaryMaxLetterShape2IouMax: 0.12,
  finalSecondaryMaterialCountMin: 44,
  finalSecondaryHaloCountMin: 40,
  finalSecondaryInsideCountMax: 8,
  finalSecondaryFarRatioMax: 0.58,
  particleKilledLitRatioMax: 0.08,
  identityShuffleLockedRatioMax: 0.20,
  identityShuffleWeightedLockedRatioMax: 0.20,
  identityShuffleLockedDropMin: 0.70,
  midBareRenderShape4IouMin: 0.45,
  midBareRenderHolePreservationMin: 0.96,
  finalBareRenderShape4IouMin: 0.82,
  finalNoGlintsShape4IouMin: 0.82,
  finalNoTrailsShape4IouMin: 0.82,
  finalBareRenderHolePreservationMin: 0.99,
  finalBareMinLetterCoverageMin: 0.72,
  finalBareMinLetterStrongMin: 0.70,
  finalBareMinLetterShape2IouMin: 0.79,
  finalBareMaxSpaceIntrusionMax: 0.02,
  finalVisibleMaterialCountMin: 2400,
  finalVisibleTraceP50Min: 300,
  finalVisibleTraceRatioP50Min: 1.00,
  finalVisibleLowTraceRatioCountMax: 0,
  finalVisibleShortTraceCountMax: 0,
  finalVisibleLowRotationCountMax: 0,
  finalVisibleRotationP50Min: 1.20,
  finalSecondaryVisibleCountMin: 40,
  finalSecondaryVisibleTraceP50Min: 200,
  finalSecondaryVisibleRotationP50Min: 0.80,
  continuityTeleportCountMax: 0,
  continuityMaterialTeleportCountMax: 0,
  continuityMaxDisplacementMax: 14,
  continuityMaterialP99Max: 11.5,
  continuityFrameMaxP95Max: 12,
  staticFullCoverageMin: 0.92,
  staticFullSmall2CoverageMin: 0.92,
  staticFullSmall4CoverageMin: 0.88,
  staticFullShape4IouMin: 0.88,
  staticFullMinLetterCoverageMin: 0.99,
  staticFullMinLetterStrongMin: 0.99,
  staticFullMinLetterShape2IouMin: 0.74,
  staticFullMaxSpaceIntrusionMax: 0.10,
  staticPartial80CoverageMin: 0.78,
  staticPartial80Small2CoverageMin: 0.78,
  staticPartial80Shape4IouMin: 0.92,
  staticPartial80MinLetterCoverageMin: 0.90,
  staticPartial80MinLetterStrongMin: 0.83,
  staticPartial80MinLetterShape2IouMin: 0.76,
  staticPartial60CoverageMin: 0.58,
  staticPartial60Small2CoverageMin: 0.58,
  staticPartial60Shape4IouMin: 0.82,
  staticPartial60MinLetterCoverageMin: 0.74,
  staticPartial60MinLetterStrongMin: 0.68,
  staticPartial60MinLetterShape2IouMin: 0.68,
  staticPrimaryPartial60CoverageMin: 0.94,
  staticPrimaryPartial60Small2CoverageMin: 0.94,
  staticPrimaryPartial60Shape4IouMin: 0.95,
  staticPrimaryPartial60MinLetterCoverageMin: 0.90,
  staticPrimaryPartial60MinLetterStrongMin: 0.86,
  staticPrimaryPartial60MinLetterShape2IouMin: 0.76,
  staticPartial60MaxSpaceIntrusionMax: 0.025,
  staticHolePreservationMin: 0.98,
  slotFlowNonHorizontalRatioMin: 0.40,
  slotFlowAvgAbsDegMin: 24.0,
  slotFlowP90AbsDegMin: 64.0,
  smallViewportWidth: 640,
  smallViewportHeight: 360,
  smallFinalLockedRatioMin: 0.90,
  smallFinalWeightedLockedRatioMin: 0.91,
  smallFinalWeightedTracedRatioMin: 0.82,
  smallFinalWeightedRotatedRatioMin: 0.98,
  smallFinalDistanceP90Max: 8,
  smallFinalAngleP90Max: 0.75,
  smallFinalPrimaryCoverageMin: 0.90,
  smallFinalPrimaryStrongMin: 0.86,
  smallFinalPrimaryShape4IouMin: 0.84,
  smallFinalPrimaryHolePreservationMin: 0.985,
  smallFinalPrimaryMinLetterCoverageMin: 0.84,
  smallFinalPrimaryMinLetterStrongMin: 0.78,
  smallFinalPrimaryMinLetterShape2IouMin: 0.82,
  smallFinalSecondaryCoverageMax: 0.05,
  smallFinalSecondaryStrongMax: 0.03,
  smallFinalSecondaryShape4IouMax: 0.08,
  smallFinalSecondaryMaxLetterCoverageMax: 0.08,
  smallFinalSecondaryMaxLetterShape2IouMax: 0.12,
  smallFinalSecondaryMaterialCountMin: 40,
  smallFinalVisibleMaterialCountMin: 2400,
  smallFinalSecondaryVisibleCountMin: 40,
  smallFinalVisibleLowTraceRatioCountMax: 0,
  smallFinalVisibleShortTraceCountMax: 0,
  smallFinalVisibleLowRotationCountMax: 0,
  smallBareRenderShape4IouMin: 0.78,
  smallNoGlintsShape4IouMin: 0.78,
  smallNoTrailsShape4IouMin: 0.78,
  smallBareRenderHolePreservationMin: 0.98,
  smallBareMinLetterCoverageMin: 0.74,
  smallBareMinLetterStrongMin: 0.72,
  smallBareMinLetterShape2IouMin: 0.78,
  smallBareMaxSpaceIntrusionMax: 0.02,
  smallPixelLitRatioMin: 0.04,
  smallPixelLitRatioMax: 0.08,
};

function argValue(name, fallback) {
  const hit = process.argv.find((arg) => arg.startsWith(`${name}=`));
  return hit ? hit.slice(name.length + 1) : fallback;
}

function fileUrl(filePath) {
  return `file:///${path.resolve(filePath).replace(/\\/g, "/")}`;
}

function fail(message) {
  throw new Error(message);
}

function assertNoRenderGlyphLayer(source) {
  const forbidden = [
    "buildLockedGlyphLayer",
    "drawLockedFilings",
    "lockedGlyphCache",
    "glyphLayouts",
    "destination-in",
  ];
  for (const token of forbidden) {
    if (source.includes(token)) {
      fail(`render-only glyph layer token present: ${token}`);
    }
  }
}

function assertMetric(condition, message, detail) {
  if (!condition) {
    fail(`${message}: ${JSON.stringify(detail)}`);
  }
}

async function smallViewportAudit(browser, htmlPath) {
  const page = await browser.newPage({
    viewport: { width: THRESHOLDS.smallViewportWidth, height: THRESHOLDS.smallViewportHeight },
    deviceScaleFactor: 1,
  });
  try {
    await page.goto(fileUrl(htmlPath));
    await page.waitForFunction(
      () =>
        typeof window.__slotAudit === "function" &&
        typeof window.__primaryCoverage === "function" &&
        typeof window.__secondaryCoverage === "function" &&
        typeof window.__secondaryHalo === "function" &&
        typeof window.__renderVariantAudit === "function" &&
        typeof window.__visibleProvenanceAudit === "function" &&
        typeof window.__birthMaterialAudit === "function" &&
        typeof window.__letterAudit === "function",
    );
    return await page.evaluate(() => {
      window.__pause();
      window.__reset(2246);
      window.__steps(330);
      const renderVariants = window.__renderVariantAudit();
      return {
        viewport: { width: window.innerWidth, height: window.innerHeight },
        audit: window.__slotAudit(),
        primaryCoverage: window.__primaryCoverage(),
        secondaryCoverage: window.__secondaryCoverage(),
        secondaryHalo: window.__secondaryHalo(),
        visibleProvenance: window.__visibleProvenanceAudit(),
        birthMaterial: window.__birthMaterialAudit(),
        renderVariants: {
          production: renderVariants.production,
          bareRods: renderVariants.bareRods,
          noGlints: renderVariants.noGlints,
          noTrails: renderVariants.noTrails,
        },
        pixel: window.__pixelStats(),
      };
    });
  } finally {
    await page.close();
  }
}

async function main() {
  const htmlPath = path.resolve(ROOT, argValue("--html", "mf_proto_live_steel.html"));
  const sourcePath = path.resolve(ROOT, argValue("--source", "template.html"));
  const outPath = path.resolve(ROOT, argValue("--out", "causal_audit.json"));
  const frames = argValue("--frames", "8,48,150,330")
    .split(",")
    .map((value) => Number.parseInt(value, 10))
    .filter((value) => Number.isFinite(value) && value >= 0);

  if (!fs.existsSync(htmlPath)) fail(`html missing: ${htmlPath}`);
  if (!fs.existsSync(sourcePath)) fail(`source missing: ${sourcePath}`);
  assertNoRenderGlyphLayer(fs.readFileSync(sourcePath, "utf8"));

  const browser = await chromium.launch();
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 1 });
    await page.goto(fileUrl(htmlPath));
    await page.waitForFunction(
      () =>
        typeof window.__slotAudit === "function" &&
        typeof window.__staticLatticeAudit === "function" &&
        typeof window.__slotFlowAudit === "function" &&
        typeof window.__primaryCoverage === "function" &&
        typeof window.__secondaryCoverage === "function" &&
        typeof window.__secondaryHalo === "function" &&
        typeof window.__particleDependencyAudit === "function" &&
        typeof window.__debugTargetIsolationAudit === "function" &&
        typeof window.__identityShuffleAudit === "function" &&
        typeof window.__renderVariantAudit === "function" &&
        typeof window.__visibleProvenanceAudit === "function" &&
        typeof window.__birthMaterialAudit === "function" &&
        typeof window.__continuityAudit === "function" &&
        typeof window.__glyphFieldPoisonAudit === "function" &&
        typeof window.__letterAudit === "function",
    );

    const result = await page.evaluate((params) => {
      const auditFrames = params.frames;
      window.__pause();
      window.__reset(2246);
      const abi = window.__abi();
      const staticLattice = abi.phrases.map((_phrase, idx) => window.__staticLatticeAudit(idx));
      const slotFlow = abi.phrases.map((_phrase, idx) => window.__slotFlowAudit(idx));
      let done = 0;
      const samples = [];
      for (const frame of auditFrames) {
        window.__steps(frame - done);
        done = frame;
        samples.push({
          frame,
          abi: window.__abi(),
          audit: window.__slotAudit(),
          coverage: window.__coverage(),
          primaryCoverage: window.__primaryCoverage(),
          secondaryCoverage: window.__secondaryCoverage(),
          secondaryHalo: window.__secondaryHalo(),
          birthMaterial: window.__birthMaterialAudit(),
          perf: window.__perf(),
          pixel: window.__pixelStats(),
        });
      }
      const renderVariants = {};
      for (const frame of auditFrames) {
        if (frame !== 150 && frame !== 330) continue;
        window.__reset(2246);
        window.__steps(frame);
        renderVariants[String(frame)] = window.__renderVariantAudit();
      }
      const continuity = window.__continuityAudit(330, params.teleportLimit);
      const visibleProvenance = window.__visibleProvenanceAudit();
      const glyphFieldPoison = window.__glyphFieldPoisonAudit();
      const particleDependency = window.__particleDependencyAudit();
      const debugTargetIsolation = window.__debugTargetIsolationAudit();
      const identityShuffle = window.__identityShuffleAudit();
      return { staticLattice, slotFlow, samples, renderVariants, continuity, visibleProvenance, glyphFieldPoison, particleDependency, debugTargetIsolation, identityShuffle };
    }, { frames, teleportLimit: THRESHOLDS.continuityMaxDisplacementMax });

    result.smallViewport = await smallViewportAudit(browser, htmlPath);
    result.createdAt = new Date().toISOString();
    result.thresholds = THRESHOLDS;
    fs.writeFileSync(outPath, JSON.stringify(result, null, 2));

    const dwell = result.samples.find((sample) => sample.frame === 330) || result.samples[result.samples.length - 1];
    const chaos = result.samples.find((sample) => sample.frame === 8);
    const early = result.samples.find((sample) => sample.frame === 48);
    const mid = result.samples.find((sample) => sample.frame === 150);
    if (chaos) {
      assertMetric(chaos.coverage.all.coveredRatio <= THRESHOLDS.chaosCoverageMax, "chaos frame already contains too much letter coverage", chaos.coverage.all);
      assertMetric(chaos.coverage.all.strongRatio <= THRESHOLDS.chaosStrongCoverageMax, "chaos frame already contains too much strong letter material", chaos.coverage.all);
      assertMetric(chaos.primaryCoverage.shape4.iou <= THRESHOLDS.chaosPrimaryShape4IouMax, "chaos frame primary rods already form a word silhouette", chaos.primaryCoverage.shape4);
      assertMetric(chaos.coverage.shape4.iou <= THRESHOLDS.chaosAllShape4IouMax, "chaos frame all material already forms a word silhouette", chaos.coverage.shape4);
      assertMetric(chaos.audit.lockedRatio <= THRESHOLDS.chaosLockedRatioMax, "chaos frame already has locked primary rods", chaos.audit);
      assertMetric(chaos.audit.capturedRatio <= THRESHOLDS.chaosCapturedRatioMax, "chaos frame already has captured primary rods", chaos.audit);
      assertMetric(chaos.audit.tracedRatio <= THRESHOLDS.chaosTracedRatioMax, "chaos frame already has traced/claimed primary rods", chaos.audit);
      assertMetric(chaos.birthMaterial.primaryVisibleRatio >= THRESHOLDS.chaosBirthVisiblePrimaryRatioMin, "chaos frame primary rods are not visibly born as metal", chaos.birthMaterial);
      assertMetric(chaos.birthMaterial.birthOnlyRatio >= THRESHOLDS.chaosBirthOnlyPrimaryRatioMin, "chaos frame visible rods are already too captured, not birth material", chaos.birthMaterial);
      assertMetric(chaos.birthMaterial.letterMaterialRatio <= THRESHOLDS.chaosBirthLetterMaterialRatioMax, "chaos frame has too much full letter-strength material", chaos.birthMaterial);
      assertMetric(chaos.pixel.litRatio >= THRESHOLDS.chaosPixelLitRatioMin, "chaos frame particles are not visible enough", chaos.pixel);
      assertMetric(chaos.pixel.litRatio <= THRESHOLDS.chaosPixelLitRatioMax, "chaos frame is too bright/dense before field assembly", chaos.pixel);
    }
    if (early) {
      assertMetric(early.coverage.all.coveredRatio >= THRESHOLDS.earlyCoverageMin, "early field direction is not visible enough", early.coverage.all);
      assertMetric(early.audit.weightedCapturedRatio >= THRESHOLDS.earlyCapturedMin, "early capture response too weak", early.audit);
      assertMetric(early.audit.frame.causalTraceCount >= THRESHOLDS.earlyCausalTraceCountMin, "early chaos-to-field provenance traces are not visible enough", early.audit.frame);
      assertMetric(early.audit.frame.causalTraceP95 >= THRESHOLDS.earlyCausalTraceP95Min, "early provenance traces are too short to show travel from chaos", early.audit.frame);
    }
    if (mid) {
      assertMetric(mid.coverage.all.coveredRatio >= THRESHOLDS.midCoverageMin, "mid-assembly pause legibility too weak", mid.coverage.all);
      assertMetric(mid.primaryCoverage.all.coveredRatio >= THRESHOLDS.midPrimaryCoverageMin, "mid-assembly primary rods do not carry enough letter coverage", mid.primaryCoverage.all);
      assertMetric(mid.primaryCoverage.shape4.iou >= THRESHOLDS.midPrimaryShape4IouMin, "mid-assembly primary rods do not form a coarse readable silhouette", mid.primaryCoverage.shape4);
      assertMetric(mid.primaryCoverage.letters.minCoverage >= THRESHOLDS.midPrimaryMinLetterCoverageMin, "mid-assembly leaves at least one letter mostly absent", mid.primaryCoverage.letters);
      assertMetric(mid.primaryCoverage.letters.minStrong >= THRESHOLDS.midPrimaryMinLetterStrongMin, "mid-assembly leaves at least one letter too weak to read", mid.primaryCoverage.letters);
      assertMetric(mid.primaryCoverage.letters.minShape2Iou >= THRESHOLDS.midPrimaryMinLetterShape2IouMin, "mid-assembly per-letter shape is too weak", mid.primaryCoverage.letters);
      assertMetric(mid.secondaryCoverage.all.coveredRatio <= THRESHOLDS.midSecondaryCoverageMax, "mid-assembly secondary halo carries too much letter coverage", mid.secondaryCoverage.all);
      assertMetric(mid.secondaryCoverage.shape4.iou <= THRESHOLDS.midSecondaryShape4IouMax, "mid-assembly secondary halo forms too much of the word silhouette", mid.secondaryCoverage.shape4);
      assertMetric(mid.audit.frame.motionTrailCount >= THRESHOLDS.midMotionTrailCountMin, "mid-assembly motion trails do not make movement visible", mid.audit.frame);
      assertMetric(mid.audit.frame.causalTraceCount >= THRESHOLDS.midCausalTraceCountMin, "mid-assembly provenance traces are not visible enough", mid.audit.frame);
      assertMetric(mid.audit.frame.causalTraceP95 >= THRESHOLDS.midCausalTraceP95Min, "mid-assembly provenance traces are too short to prove travel from chaos", mid.audit.frame);
      assertMetric(mid.audit.weightedLockedRatio >= THRESHOLDS.midWeightedLockedMin, "mid-assembly slot lock too weak", mid.audit);
      assertMetric(mid.audit.weightedCapturedRatio >= THRESHOLDS.midWeightedCapturedMin, "mid-assembly capture trace too weak", mid.audit);
      assertMetric(mid.audit.weightedTracedRatio >= THRESHOLDS.midWeightedTracedMin, "mid-assembly chaos-to-slot trace too weak", mid.audit);
      assertMetric(mid.audit.weightedRotatedRatio >= THRESHOLDS.midWeightedRotatedMin, "mid-assembly rods have not visibly rotated into the field", mid.audit);
      const midBare = result.renderVariants?.["150"]?.bareRods;
      assertMetric(Boolean(midBare), "mid bare-rods render variant audit missing", result.renderVariants);
      assertMetric(midBare.shape4.iou >= THRESHOLDS.midBareRenderShape4IouMin, "mid bare rods do not hold a readable canvas silhouette without effects", midBare);
      assertMetric(midBare.all.holePreservation >= THRESHOLDS.midBareRenderHolePreservationMin, "mid bare rods intrude into counters without effects", midBare.all);
    }
    for (const lattice of result.staticLattice) {
      const full = lattice.fills.find((item) => item.fillRatio === 1);
      const partial80 = lattice.fills.find((item) => item.fillRatio === 0.8);
      const partial60 = lattice.fills.find((item) => item.fillRatio === 0.6);
      assertMetric(Boolean(full && partial80 && partial60), "static lattice fill levels missing", lattice);
      assertMetric(full.all.coveredRatio >= THRESHOLDS.staticFullCoverageMin, "static 100% lattice coverage too low", { phrase: lattice.phrase, full });
      assertMetric(full.small2.coveredRatio >= THRESHOLDS.staticFullSmall2CoverageMin, "static 100% small-2 coverage too low", { phrase: lattice.phrase, full });
      assertMetric(full.small4.coveredRatio >= THRESHOLDS.staticFullSmall4CoverageMin, "static 100% small-4 coverage too low", { phrase: lattice.phrase, full });
      assertMetric(full.shape4.iou >= THRESHOLDS.staticFullShape4IouMin, "static 100% coarse silhouette match too low", { phrase: lattice.phrase, full });
      assertMetric(full.all.holePreservation >= THRESHOLDS.staticHolePreservationMin, "static 100% counter intrusion too high", { phrase: lattice.phrase, full });
      assertMetric(full.letters.minCoverage >= THRESHOLDS.staticFullMinLetterCoverageMin, "static 100% has an unreadable weak letter", { phrase: lattice.phrase, letters: full.letters });
      assertMetric(full.letters.minStrong >= THRESHOLDS.staticFullMinLetterStrongMin, "static 100% has a structurally weak letter", { phrase: lattice.phrase, letters: full.letters });
      assertMetric(full.letters.minShape2Iou >= THRESHOLDS.staticFullMinLetterShape2IouMin, "static 100% per-letter shape match too low", { phrase: lattice.phrase, letters: full.letters });
      assertMetric(full.letters.maxSpaceIntrusion <= THRESHOLDS.staticFullMaxSpaceIntrusionMax, "static 100% rods overrun an inter-letter space", { phrase: lattice.phrase, letters: full.letters });
      assertMetric(partial80.all.coveredRatio >= THRESHOLDS.staticPartial80CoverageMin, "static 80% lattice coverage too low", { phrase: lattice.phrase, partial80 });
      assertMetric(partial80.small2.coveredRatio >= THRESHOLDS.staticPartial80Small2CoverageMin, "static 80% small-2 coverage too low", { phrase: lattice.phrase, partial80 });
      assertMetric(partial80.shape4.iou >= THRESHOLDS.staticPartial80Shape4IouMin, "static 80% coarse silhouette match too low", { phrase: lattice.phrase, partial80 });
      assertMetric(partial80.all.holePreservation >= THRESHOLDS.staticHolePreservationMin, "static 80% counter intrusion too high", { phrase: lattice.phrase, partial80 });
      assertMetric(partial80.letters.minCoverage >= THRESHOLDS.staticPartial80MinLetterCoverageMin, "static 80% has an unreadable weak letter", { phrase: lattice.phrase, letters: partial80.letters });
      assertMetric(partial80.letters.minStrong >= THRESHOLDS.staticPartial80MinLetterStrongMin, "static 80% has a structurally weak letter", { phrase: lattice.phrase, letters: partial80.letters });
      assertMetric(partial80.letters.minShape2Iou >= THRESHOLDS.staticPartial80MinLetterShape2IouMin, "static 80% per-letter shape match too low", { phrase: lattice.phrase, letters: partial80.letters });
      assertMetric(partial60.all.coveredRatio >= THRESHOLDS.staticPartial60CoverageMin, "static 60% lattice coverage too low", { phrase: lattice.phrase, partial60 });
      assertMetric(partial60.small2.coveredRatio >= THRESHOLDS.staticPartial60Small2CoverageMin, "static 60% small-2 coverage too low", { phrase: lattice.phrase, partial60 });
      assertMetric(partial60.shape4.iou >= THRESHOLDS.staticPartial60Shape4IouMin, "static 60% coarse silhouette match too low", { phrase: lattice.phrase, partial60 });
      assertMetric(partial60.all.holePreservation >= THRESHOLDS.staticHolePreservationMin, "static 60% counter intrusion too high", { phrase: lattice.phrase, partial60 });
      assertMetric(partial60.letters.minCoverage >= THRESHOLDS.staticPartial60MinLetterCoverageMin, "static 60% has an unreadable weak letter", { phrase: lattice.phrase, letters: partial60.letters });
      assertMetric(partial60.letters.minStrong >= THRESHOLDS.staticPartial60MinLetterStrongMin, "static 60% has a structurally weak letter", { phrase: lattice.phrase, letters: partial60.letters });
      assertMetric(partial60.letters.minShape2Iou >= THRESHOLDS.staticPartial60MinLetterShape2IouMin, "static 60% per-letter shape match too low", { phrase: lattice.phrase, letters: partial60.letters });
      assertMetric(partial60.letters.maxSpaceIntrusion <= THRESHOLDS.staticPartial60MaxSpaceIntrusionMax, "static 60% rods overrun an inter-letter space", { phrase: lattice.phrase, letters: partial60.letters });
      if (lattice.phraseIndex === 0) {
        assertMetric(partial60.all.coveredRatio >= THRESHOLDS.staticPrimaryPartial60CoverageMin, "primary static 60% lattice coverage does not meet the v41 rail-readability bar", { phrase: lattice.phrase, partial60 });
        assertMetric(partial60.small2.coveredRatio >= THRESHOLDS.staticPrimaryPartial60Small2CoverageMin, "primary static 60% small-2 coverage does not meet the v41 rail-readability bar", { phrase: lattice.phrase, partial60 });
        assertMetric(partial60.shape4.iou >= THRESHOLDS.staticPrimaryPartial60Shape4IouMin, "primary static 60% coarse silhouette does not meet the v41 rail-readability bar", { phrase: lattice.phrase, partial60 });
        assertMetric(partial60.letters.minCoverage >= THRESHOLDS.staticPrimaryPartial60MinLetterCoverageMin, "primary static 60% leaves a weak letter below the v41 rail-readability bar", { phrase: lattice.phrase, letters: partial60.letters });
        assertMetric(partial60.letters.minStrong >= THRESHOLDS.staticPrimaryPartial60MinLetterStrongMin, "primary static 60% leaves a structurally weak letter below the v41 rail-readability bar", { phrase: lattice.phrase, letters: partial60.letters });
        assertMetric(partial60.letters.minShape2Iou >= THRESHOLDS.staticPrimaryPartial60MinLetterShape2IouMin, "primary static 60% per-letter shape is below the v41 rail-readability bar", { phrase: lattice.phrase, letters: partial60.letters });
      }
    }
    for (const flow of result.slotFlow.slice(0, 1)) {
      assertMetric(flow.nonHorizontalRatio >= THRESHOLDS.slotFlowNonHorizontalRatioMin, "slot lattice is still too horizontally filled; stroke-flow tangent diversity is too low", flow);
      assertMetric(flow.avgAbsDeg >= THRESHOLDS.slotFlowAvgAbsDegMin, "slot lattice average stroke-flow angle is too flat", flow);
      assertMetric(flow.p90AbsDeg >= THRESHOLDS.slotFlowP90AbsDegMin, "slot lattice has too few visibly angled stroke-flow rods", flow);
    }
    assertMetric(
      dwell.audit.slotCount >= dwell.coverage.all.support * THRESHOLDS.slotToSupportRatioMin,
      "slot graph too sparse for glyph support",
      { slotCount: dwell.audit.slotCount, support: dwell.coverage.all.support },
    );
    assertMetric(dwell.audit.identityRatio === 1, "particle identity assignment incomplete", dwell.audit);
    assertMetric(dwell.audit.materialRatio >= THRESHOLDS.materialRatioMin, "too many assigned slots have no material particle", dwell.audit);
    assertMetric(dwell.audit.weightedMaterialRatio >= THRESHOLDS.weightedMaterialRatioMin, "weighted material fill too low", dwell.audit);
    assertMetric(dwell.audit.weightedLockedRatio >= THRESHOLDS.weightedLockedRatioMin, "weighted slot lock too low", dwell.audit);
    assertMetric(dwell.audit.weightedCapturedRatio >= THRESHOLDS.weightedCapturedRatioMin, "captured particle ratio too low", dwell.audit);
    assertMetric(dwell.audit.weightedTracedRatio >= THRESHOLDS.weightedTracedRatioMin, "chaos-to-slot trace ratio too low", dwell.audit);
    assertMetric(dwell.audit.weightedRotatedRatio >= THRESHOLDS.weightedRotatedRatioMin, "primary rods did not accumulate enough rotation", dwell.audit);
    assertMetric(dwell.audit.weightedStateLockedRatio >= THRESHOLDS.weightedStateLockedRatioMin, "locked/settled state ratio too low", dwell.audit);
    assertMetric(dwell.audit.distancePx.p90 <= THRESHOLDS.distanceP90Max, "slot distance p90 too high", dwell.audit.distancePx);
    assertMetric(dwell.audit.activeDistancePx.p90 <= THRESHOLDS.activeDistanceP90Max, "active slot distance p90 too high", dwell.audit.activeDistancePx);
    assertMetric(dwell.audit.angleRad.p90 <= THRESHOLDS.angleP90Max, "slot tangent angle p90 too high", dwell.audit.angleRad);
    assertMetric(dwell.audit.birthToSlotPx.p50 >= THRESHOLDS.birthToSlotP50Min, "particles appear too close to final glyph", dwell.audit.birthToSlotPx);
    assertMetric(dwell.audit.traceToBirthRatio.p50 >= THRESHOLDS.traceToBirthP50Min, "particle traces do not prove travel from chaos", dwell.audit.traceToBirthRatio);
    assertMetric(dwell.audit.rotationRad.p50 >= THRESHOLDS.rotationP50Min, "primary rods did not rotate enough before final lock", dwell.audit.rotationRad);
    assertMetric(dwell.audit.rotationRad.p90 >= THRESHOLDS.rotationP90Min, "too few primary rods show substantial rotation before final lock", dwell.audit.rotationRad);
    assertMetric(dwell.audit.captureTimeS.p95 <= THRESHOLDS.captureTimeP95Max, "too many primary rods capture after the final dwell proof point", dwell.audit.captureTimeS);
    assertMetric(dwell.coverage.all.coveredRatio >= THRESHOLDS.coverageMin, "stroke coverage too low", dwell.coverage.all);
    assertMetric(dwell.coverage.all.holePreservation >= THRESHOLDS.holePreservationMin, "counter/hole intrusion too high", dwell.coverage.all);
    assertMetric(dwell.primaryCoverage.all.coveredRatio >= THRESHOLDS.finalPrimaryCoverageMin, "final primary rods do not carry the word", dwell.primaryCoverage.all);
    assertMetric(dwell.primaryCoverage.all.strongRatio >= THRESHOLDS.finalPrimaryStrongMin, "final primary rods are too weak to carry strokes", dwell.primaryCoverage.all);
    assertMetric(dwell.primaryCoverage.shape4.iou >= THRESHOLDS.finalPrimaryShape4IouMin, "final primary rods do not form the word silhouette", dwell.primaryCoverage.shape4);
    assertMetric(dwell.primaryCoverage.all.holePreservation >= THRESHOLDS.finalPrimaryHolePreservationMin, "final primary rods intrude into counters", dwell.primaryCoverage.all);
    assertMetric(dwell.primaryCoverage.letters.minCoverage >= THRESHOLDS.finalPrimaryMinLetterCoverageMin, "final primary rods leave at least one letter unreadable", dwell.primaryCoverage.letters);
    assertMetric(dwell.primaryCoverage.letters.minStrong >= THRESHOLDS.finalPrimaryMinLetterStrongMin, "final primary rods leave at least one letter structurally weak", dwell.primaryCoverage.letters);
    assertMetric(dwell.primaryCoverage.letters.minShape2Iou >= THRESHOLDS.finalPrimaryMinLetterShape2IouMin, "final primary rods fail per-letter shape", dwell.primaryCoverage.letters);
    assertMetric(dwell.primaryCoverage.letters.maxSpaceIntrusion <= THRESHOLDS.finalPrimaryMaxSpaceIntrusionMax, "final primary rods overrun an inter-letter space", dwell.primaryCoverage.letters);
    assertMetric(dwell.secondaryCoverage.all.coveredRatio <= THRESHOLDS.finalSecondaryCoverageMax, "final secondary halo carries too much word coverage", dwell.secondaryCoverage.all);
    assertMetric(dwell.secondaryCoverage.all.strongRatio <= THRESHOLDS.finalSecondaryStrongMax, "final secondary halo carries too much strong word coverage", dwell.secondaryCoverage.all);
    assertMetric(dwell.secondaryCoverage.shape4.iou <= THRESHOLDS.finalSecondaryShape4IouMax, "final secondary halo forms too much of the word silhouette", dwell.secondaryCoverage.shape4);
    assertMetric(dwell.secondaryCoverage.letters.maxCoverage <= THRESHOLDS.finalSecondaryMaxLetterCoverageMax, "final secondary halo carries a readable letter region", dwell.secondaryCoverage.letters);
    assertMetric(dwell.secondaryCoverage.letters.maxShape2Iou <= THRESHOLDS.finalSecondaryMaxLetterShape2IouMax, "final secondary halo forms a letter shape", dwell.secondaryCoverage.letters);
    assertMetric(dwell.secondaryHalo.materialCount >= THRESHOLDS.finalSecondaryMaterialCountMin, "final secondary material is too sparse to prove surplus metal", dwell.secondaryHalo);
    assertMetric(dwell.secondaryHalo.haloCount >= THRESHOLDS.finalSecondaryHaloCountMin, "final secondary material is not visible in the halo band", dwell.secondaryHalo);
    assertMetric(dwell.secondaryHalo.insideCount <= THRESHOLDS.finalSecondaryInsideCountMax, "final secondary material intrudes into the letter support", dwell.secondaryHalo);
    assertMetric(dwell.secondaryHalo.farRatio <= THRESHOLDS.finalSecondaryFarRatioMax, "final secondary material drifts too far from the word to read as controlled halo", dwell.secondaryHalo);
    assertMetric(dwell.audit.frame.motionTrailCount <= THRESHOLDS.finalMotionTrailCountMax, "final settled word still has too many motion trails", dwell.audit.frame);
    assertMetric(dwell.audit.frame.causalTraceCount <= THRESHOLDS.finalCausalTraceCountMax, "final word is still propped up by provenance traces", dwell.audit.frame);
    const finalVariants = result.renderVariants?.["330"];
    assertMetric(Boolean(finalVariants?.bareRods && finalVariants?.noGlints && finalVariants?.noTrails), "final render variant audit missing", result.renderVariants);
    assertMetric(finalVariants.bareRods.shape4.iou >= THRESHOLDS.finalBareRenderShape4IouMin, "final bare rods do not hold the rendered word without effects", finalVariants.bareRods);
    assertMetric(finalVariants.noGlints.shape4.iou >= THRESHOLDS.finalNoGlintsShape4IouMin, "final word depends too much on glints", finalVariants.noGlints);
    assertMetric(finalVariants.noTrails.shape4.iou >= THRESHOLDS.finalNoTrailsShape4IouMin, "final word depends too much on motion/provenance trails", finalVariants.noTrails);
    assertMetric(finalVariants.bareRods.all.holePreservation >= THRESHOLDS.finalBareRenderHolePreservationMin, "final bare rods intrude into counters without effects", finalVariants.bareRods.all);
    for (const name of ["bareRods", "noGlints", "noTrails"]) {
      const letters = finalVariants[name].letters;
      assertMetric(letters.minCoverage >= THRESHOLDS.finalBareMinLetterCoverageMin, `final ${name} leaves at least one letter unreadable`, letters);
      assertMetric(letters.minStrong >= THRESHOLDS.finalBareMinLetterStrongMin, `final ${name} leaves at least one letter structurally weak`, letters);
      assertMetric(letters.minShape2Iou >= THRESHOLDS.finalBareMinLetterShape2IouMin, `final ${name} fails per-letter shape`, letters);
      assertMetric(letters.maxSpaceIntrusion <= THRESHOLDS.finalBareMaxSpaceIntrusionMax, `final ${name} overruns an inter-letter space`, letters);
    }
    assertMetric(result.visibleProvenance.all.count >= THRESHOLDS.finalVisibleMaterialCountMin, "too few final visible particles have material provenance", result.visibleProvenance.all);
    assertMetric(result.visibleProvenance.all.tracePx.p50 >= THRESHOLDS.finalVisibleTraceP50Min, "final visible particles did not travel far enough from chaos", result.visibleProvenance.all.tracePx);
    assertMetric(result.visibleProvenance.all.traceToBirthRatio.p50 >= THRESHOLDS.finalVisibleTraceRatioP50Min, "final visible particles do not trace their birth-to-final path", result.visibleProvenance.all.traceToBirthRatio);
    assertMetric(result.visibleProvenance.all.lowTraceRatioCount <= THRESHOLDS.finalVisibleLowTraceRatioCountMax, "some final visible particles lack birth-to-final trace provenance", result.visibleProvenance.all);
    assertMetric(result.visibleProvenance.all.shortTraceCount <= THRESHOLDS.finalVisibleShortTraceCountMax, "some final visible particles barely moved from their birth positions", result.visibleProvenance.all);
    assertMetric(result.visibleProvenance.all.lowRotationCount <= THRESHOLDS.finalVisibleLowRotationCountMax, "some final visible particles did not rotate enough to prove magnetic alignment", result.visibleProvenance.all);
    assertMetric(result.visibleProvenance.all.rotationRad.p50 >= THRESHOLDS.finalVisibleRotationP50Min, "final visible particles have weak rotation provenance", result.visibleProvenance.all.rotationRad);
    assertMetric(result.visibleProvenance.secondary.count >= THRESHOLDS.finalSecondaryVisibleCountMin, "too few final secondary halo particles remain visibly provenanced", result.visibleProvenance.secondary);
    assertMetric(result.visibleProvenance.secondary.tracePx.p50 >= THRESHOLDS.finalSecondaryVisibleTraceP50Min, "secondary halo particles did not visibly travel from chaos", result.visibleProvenance.secondary.tracePx);
    assertMetric(result.visibleProvenance.secondary.rotationRad.p50 >= THRESHOLDS.finalSecondaryVisibleRotationP50Min, "secondary halo particles did not visibly rotate with the field", result.visibleProvenance.secondary.rotationRad);
    const small = result.smallViewport;
    assertMetric(small.viewport.width === THRESHOLDS.smallViewportWidth && small.viewport.height === THRESHOLDS.smallViewportHeight, "small viewport audit ran at the wrong size", small.viewport);
    assertMetric(small.audit.lockedRatio >= THRESHOLDS.smallFinalLockedRatioMin, "small viewport final rods do not lock into readable slots", small.audit);
    assertMetric(small.audit.weightedLockedRatio >= THRESHOLDS.smallFinalWeightedLockedRatioMin, "small viewport weighted slot lock too weak", small.audit);
    assertMetric(small.audit.weightedTracedRatio >= THRESHOLDS.smallFinalWeightedTracedRatioMin, "small viewport chaos-to-slot trace too weak", small.audit);
    assertMetric(small.audit.weightedRotatedRatio >= THRESHOLDS.smallFinalWeightedRotatedRatioMin, "small viewport rods have not visibly rotated into the field", small.audit);
    assertMetric(small.audit.distancePx.p90 <= THRESHOLDS.smallFinalDistanceP90Max, "small viewport slot distance p90 too high", small.audit.distancePx);
    assertMetric(small.audit.angleRad.p90 <= THRESHOLDS.smallFinalAngleP90Max, "small viewport slot tangent angle p90 too high", small.audit.angleRad);
    assertMetric(small.primaryCoverage.all.coveredRatio >= THRESHOLDS.smallFinalPrimaryCoverageMin, "small viewport primary rods do not carry enough letter coverage", small.primaryCoverage.all);
    assertMetric(small.primaryCoverage.all.strongRatio >= THRESHOLDS.smallFinalPrimaryStrongMin, "small viewport primary rods are too weak to carry strokes", small.primaryCoverage.all);
    assertMetric(small.primaryCoverage.shape4.iou >= THRESHOLDS.smallFinalPrimaryShape4IouMin, "small viewport primary rods do not form the word silhouette", small.primaryCoverage.shape4);
    assertMetric(small.primaryCoverage.all.holePreservation >= THRESHOLDS.smallFinalPrimaryHolePreservationMin, "small viewport primary rods intrude into counters", small.primaryCoverage.all);
    assertMetric(small.primaryCoverage.letters.minCoverage >= THRESHOLDS.smallFinalPrimaryMinLetterCoverageMin, "small viewport primary rods leave at least one letter unreadable", small.primaryCoverage.letters);
    assertMetric(small.primaryCoverage.letters.minStrong >= THRESHOLDS.smallFinalPrimaryMinLetterStrongMin, "small viewport primary rods leave at least one letter structurally weak", small.primaryCoverage.letters);
    assertMetric(small.primaryCoverage.letters.minShape2Iou >= THRESHOLDS.smallFinalPrimaryMinLetterShape2IouMin, "small viewport primary rods fail per-letter shape", small.primaryCoverage.letters);
    assertMetric(small.secondaryCoverage.all.coveredRatio <= THRESHOLDS.smallFinalSecondaryCoverageMax, "small viewport secondary halo carries too much word coverage", small.secondaryCoverage.all);
    assertMetric(small.secondaryCoverage.all.strongRatio <= THRESHOLDS.smallFinalSecondaryStrongMax, "small viewport secondary halo carries too much strong word coverage", small.secondaryCoverage.all);
    assertMetric(small.secondaryCoverage.shape4.iou <= THRESHOLDS.smallFinalSecondaryShape4IouMax, "small viewport secondary halo forms too much of the word silhouette", small.secondaryCoverage.shape4);
    assertMetric(small.secondaryCoverage.letters.maxCoverage <= THRESHOLDS.smallFinalSecondaryMaxLetterCoverageMax, "small viewport secondary halo carries a readable letter region", small.secondaryCoverage.letters);
    assertMetric(small.secondaryCoverage.letters.maxShape2Iou <= THRESHOLDS.smallFinalSecondaryMaxLetterShape2IouMax, "small viewport secondary halo forms a letter shape", small.secondaryCoverage.letters);
    assertMetric(small.secondaryHalo.materialCount >= THRESHOLDS.smallFinalSecondaryMaterialCountMin, "small viewport secondary material is too sparse", small.secondaryHalo);
    assertMetric(small.visibleProvenance.all.count >= THRESHOLDS.smallFinalVisibleMaterialCountMin, "too few small viewport visible particles have material provenance", small.visibleProvenance.all);
    assertMetric(small.visibleProvenance.secondary.count >= THRESHOLDS.smallFinalSecondaryVisibleCountMin, "too few small viewport secondary halo particles remain visibly provenanced", small.visibleProvenance.secondary);
    assertMetric(small.visibleProvenance.all.lowTraceRatioCount <= THRESHOLDS.smallFinalVisibleLowTraceRatioCountMax, "some small viewport visible particles lack birth-to-final trace provenance", small.visibleProvenance.all);
    assertMetric(small.visibleProvenance.all.shortTraceCount <= THRESHOLDS.smallFinalVisibleShortTraceCountMax, "some small viewport visible particles barely moved from birth positions", small.visibleProvenance.all);
    assertMetric(small.visibleProvenance.all.lowRotationCount <= THRESHOLDS.smallFinalVisibleLowRotationCountMax, "some small viewport visible particles did not rotate enough", small.visibleProvenance.all);
    assertMetric(small.renderVariants.bareRods.shape4.iou >= THRESHOLDS.smallBareRenderShape4IouMin, "small viewport bare rods do not hold the rendered word without effects", small.renderVariants.bareRods);
    assertMetric(small.renderVariants.noGlints.shape4.iou >= THRESHOLDS.smallNoGlintsShape4IouMin, "small viewport word depends too much on glints", small.renderVariants.noGlints);
    assertMetric(small.renderVariants.noTrails.shape4.iou >= THRESHOLDS.smallNoTrailsShape4IouMin, "small viewport word depends too much on motion/provenance trails", small.renderVariants.noTrails);
    assertMetric(small.renderVariants.bareRods.all.holePreservation >= THRESHOLDS.smallBareRenderHolePreservationMin, "small viewport bare rods intrude into counters without effects", small.renderVariants.bareRods.all);
    for (const name of ["bareRods", "noGlints", "noTrails"]) {
      const letters = small.renderVariants[name].letters;
      assertMetric(letters.minCoverage >= THRESHOLDS.smallBareMinLetterCoverageMin, `small viewport ${name} leaves at least one letter unreadable`, letters);
      assertMetric(letters.minStrong >= THRESHOLDS.smallBareMinLetterStrongMin, `small viewport ${name} leaves at least one letter structurally weak`, letters);
      assertMetric(letters.minShape2Iou >= THRESHOLDS.smallBareMinLetterShape2IouMin, `small viewport ${name} fails per-letter shape`, letters);
      assertMetric(letters.maxSpaceIntrusion <= THRESHOLDS.smallBareMaxSpaceIntrusionMax, `small viewport ${name} overruns an inter-letter space`, letters);
    }
    assertMetric(small.pixel.litRatio >= THRESHOLDS.smallPixelLitRatioMin, "small viewport particle frame is too dark", small.pixel);
    assertMetric(small.pixel.litRatio <= THRESHOLDS.smallPixelLitRatioMax, "small viewport particle frame is too dense", small.pixel);
    assertMetric(result.continuity.teleportCount <= THRESHOLDS.continuityTeleportCountMax, "particle path contains teleport-sized jumps", result.continuity);
    assertMetric(result.continuity.materialTeleportCount <= THRESHOLDS.continuityMaterialTeleportCountMax, "visible material path contains teleport-sized jumps", result.continuity);
    assertMetric(result.continuity.all.max <= THRESHOLDS.continuityMaxDisplacementMax, "particle max per-frame displacement is too high", result.continuity.all);
    assertMetric(result.continuity.material.p99 <= THRESHOLDS.continuityMaterialP99Max, "visible material p99 per-frame displacement is too high", result.continuity.material);
    assertMetric(result.continuity.frameMax.p95 <= THRESHOLDS.continuityFrameMaxP95Max, "too many frames contain large particle jumps", result.continuity.frameMax);
    assertMetric(result.glyphFieldPoison.poisonUnchanged, "production render changes when hidden glyph/compiler fields are poisoned", result.glyphFieldPoison);
    assertMetric(result.glyphFieldPoison.restoredMatches, "glyph/compiler field poison audit does not restore production frame", result.glyphFieldPoison);
    assertMetric(result.particleDependency.production.pixel.litRatio > 0.01, "production particle frame is too dark for dependency audit", result.particleDependency.production.pixel);
    assertMetric(result.particleDependency.killedToProductionLitRatio <= THRESHOLDS.particleKilledLitRatioMax, "production frame survives after particle buffer is killed", result.particleDependency);
    assertMetric(result.particleDependency.restoredMatches, "particle dependency audit does not restore production frame deterministically", result.particleDependency);
    assertMetric(result.debugTargetIsolation.debugDiffers, "debug target view does not differ from production frame", result.debugTargetIsolation);
    assertMetric(result.debugTargetIsolation.restoredMatches, "debug target view leaks into production frame after being disabled", result.debugTargetIsolation);
    assertMetric(result.identityShuffle.actual.lockedRatio >= THRESHOLDS.weightedLockedRatioMin, "identity shuffle baseline is not locked enough to be meaningful", result.identityShuffle.actual);
    assertMetric(result.identityShuffle.shuffled.lockedRatio <= THRESHOLDS.identityShuffleLockedRatioMax, "shuffled particle-slot identity still appears locked", result.identityShuffle);
    assertMetric(result.identityShuffle.shuffled.weightedLockedRatio <= THRESHOLDS.identityShuffleWeightedLockedRatioMax, "shuffled weighted particle-slot identity still appears locked", result.identityShuffle);
    assertMetric(result.identityShuffle.lockedDrop >= THRESHOLDS.identityShuffleLockedDropMin, "particle-slot identity shuffle does not break enough locks", result.identityShuffle);

    console.log(`PASS: causal letter audit -> ${outPath}`);
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(`FAIL: ${error.message}`);
  process.exit(1);
});
