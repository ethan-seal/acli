#!/usr/bin/env bun

/**
 * Convert Art.xls to Markdown card format.
 *
 * Setup: bun add xlsx
 * Usage: bun convert_art.ts Art.xls > art-cards.md
 *
 * Also runnable with Node.js:
 *   npx ts-node convert_art.ts Art.xls > art-cards.md
 *   node -e "$(cat convert_art.ts | sed 's/Bun\.argv/process.argv/g')" Art.xls
 */

import { readFileSync } from "node:fs";
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
const sheet = workbook.Sheets[workbook.SheetNames[0]];
const rows: string[][] = XLSX.utils.sheet_to_json(sheet, { header: 1 });

for (const row of rows) {
  const imgPath = String(row[0] ?? "").trim();
  const info = String(row[1] ?? "").trim();

  // Skip rows with no image reference
  if (!imgPath) continue;

  // Skip rows where both columns are empty
  if (!imgPath && !info) continue;

  const parts = info.split("\n", 2);
  const title = parts[0]?.trim() || "Unknown";
  const artist = parts[1]?.trim() || "Unknown artist";

  console.log(`- ![](${imgPath}) -> ${title} by ${artist}`);
}
