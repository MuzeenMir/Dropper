// Renders templates/blockpage.html with fixture values for the a11y CI gate.
// Mirrors src/blockpage/mod.rs render(): plain string replacement of the 8
// documented {{tokens}}. Fixture values are realistic-length so contrast and
// layout are evaluated on representative content, not empty strings.
//
// Usage: node render-blockpage-fixture.mjs <template-path> <output-path>

import { readFileSync, writeFileSync } from "node:fs";

const [templatePath, outputPath] = process.argv.slice(2);
if (!templatePath || !outputPath) {
  console.error("usage: render-blockpage-fixture.mjs <template-path> <output-path>");
  process.exit(2);
}

const fixtures = {
  domain: "malware-delivery.example-threat.com",
  feed: "URLhaus",
  listed_date: "2026-06-01",
  listed_relative: "(11 days ago)",
  threat_type: "malware host / credential harvest",
  block_id: "a1b2c3d4",
  ts_iso: "2026-06-12T15:00:00Z",
  version: "0.1.4",
};

let html = readFileSync(templatePath, "utf8");
for (const [token, value] of Object.entries(fixtures)) {
  html = html.replaceAll(`{{${token}}}`, value);
}

// Same guarantee as the Rust test render_leaves_no_unsubstituted_tokens:
// a new template token without a fixture value must fail loudly, not ship
// "{{foo}}" literals into the page axe evaluates.
const leftover = html.match(/\{\{[a-z_]+\}\}/g);
if (leftover) {
  console.error(`unsubstituted template tokens: ${[...new Set(leftover)].join(", ")}`);
  console.error("add fixture values for them in render-blockpage-fixture.mjs");
  process.exit(1);
}

writeFileSync(outputPath, html);
console.log(`rendered ${templatePath} -> ${outputPath} (${html.length} bytes)`);
