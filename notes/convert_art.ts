#!/usr/bin/env bun

/**
 * Convert Art.xls to Markdown card format.
 *
 * Setup: bun add xlsx
 * Usage: bun convert_art.ts Art.xls
 *
 * Creates one output file per sheet, named after the sheet:
 *   <SheetName>.md
 *
 * Also runnable with Node.js:
 *   npx ts-node convert_art.ts Art.xls
 */

import { readFileSync, writeFileSync } from "node:fs";
import * as XLSX from "xlsx";

// Support both Bun and Node runtimes
const argv: string[] =
  typeof Bun !== "undefined" ? Bun.argv : process.argv;

// argv[0] = runtime, argv[1] = script, argv[2] = xls path
const xlsPath = argv[2];
if (!xlsPath) {
  console.error("Usage: bun convert_art.ts <xls_file>");
  process.exit(1);
}

const data = readFileSync(xlsPath);
const workbook = XLSX.read(data);

for (const sheetName of workbook.SheetNames) {
  const sheet = workbook.Sheets[sheetName];
  const rows: string[][] = XLSX.utils.sheet_to_json(sheet, { header: 1 });

  const lines: string[] = [];

  for (const row of rows) {
    const imgPath = String(row[0] ?? "").trim();
    const info = String(row[1] ?? "").trim();

    // Skip rows with no image reference
    if (!imgPath) continue;

    const parts = info.split("\n", 2);
    const title = parts[0]?.trim() || "Unknown";
    const artist = parts[1]?.trim() || "Unknown artist";

    lines.push(`- ![](${imgPath}) -> ${title} by ${artist}`);
  }

  // Sanitize sheet name for use as a filename (replace path-unsafe chars)
  const safeName = sheetName.replace(/[/\\?%*:|"<>]/g, "_");
  const outPath = `${safeName}.md`;

  writeFileSync(outPath, lines.join("\n") + "\n");
  console.log(`Wrote ${lines.length} card(s) to ${outPath}`);
}
