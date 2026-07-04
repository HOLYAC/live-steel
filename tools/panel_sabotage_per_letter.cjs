const fs = require("fs");
const path = require("path");

function argValue(name, fallback = "") {
  const prefix = `${name}=`;
  const hit = process.argv.find((arg) => arg.startsWith(prefix));
  return hit ? hit.slice(prefix.length) : fallback;
}

function replaceOnce(html, pattern, replacement, label) {
  const next = html.replace(pattern, replacement);
  if (next === html) throw new Error(`sabotage anchor not found: ${label}`);
  return next;
}

function main() {
  const input = argValue("--in");
  const output = argValue("--out");
  if (!input || !output) throw new Error("usage: node tools/panel_sabotage_per_letter.cjs --in=panel.html --out=panel_sabotaged.html");
  let html = fs.readFileSync(input, "utf8");

  // Kill the visible glyph override path while leaving state setters intact.
  // This is the exact corpse the old smoke missed: state says rotation=-18,
  // but physics input receives default letters.
  html = replaceOnce(
    html,
    /const\s+override\s*=\s*isMultiPhrase\(\)\s*\?\s*defaultLetter\(letterIndex,\s*ch\)\s*:\s*\(state\.letters\[letterIndex\]\s*\|\|\s*defaultLetter\(letterIndex,\s*ch\)\);|const\s+override\s*=\s*state\.letters\[letterIndex\]\s*\|\|\s*defaultLetter\(letterIndex,\s*ch\);/,
    "const override = defaultLetter(letterIndex, ch);",
    "glyph override lookup (v2 ternary or v1 direct)",
  );

  // Kill spacing separately because spacing can affect line layout before glyph paint.
  html = replaceOnce(
    html,
    /if\s*\(letter\)\s*cursor\s*\+=\s*clamp\(letter\.spacing,\s*-18,\s*24\);/,
    "if (false && letter) cursor += clamp(letter.spacing, -18, 24);",
    "per-letter spacing layout",
  );

  fs.mkdirSync(path.dirname(path.resolve(output)), { recursive: true });
  fs.writeFileSync(output, html, "utf8");
  console.log(`wrote sabotaged panel: ${output}`);
}

try {
  main();
} catch (error) {
  console.error(`FAIL: ${error.message}`);
  process.exit(1);
}
