#!/usr/bin/env bun
/**
 * Main demo runner for acli.
 *
 * Orchestrates the complete demo:
 * 1. Sets up the Anki collection
 * 2. For each phase: write markdown, run acli, take screenshots
 * 3. Generates an HTML report
 */
import { parseArgs } from "util";
import { existsSync, mkdirSync } from "fs";
import { $ } from "bun";

import { sleep } from "./anki-controller";
import { DEMO_SCENARIO, type Phase } from "./scenario";
import { saveReport, type DemoReport, type PhaseResult } from "./report";

interface Config {
  acliBinary: string;
  outputDir: string;
  collectionPath: string;
  contentDir: string;
  screenshotsDir: string;
}

function ensureDir(path: string): void {
  if (!existsSync(path)) {
    mkdirSync(path, { recursive: true });
  }
}

async function writeMarkdown(config: Config, content: string): Promise<string> {
  const mdPath = `${config.contentDir}/flashcards.md`;
  await Bun.write(mdPath, content);
  return mdPath;
}

async function runAcliSync(
  config: Config,
  deckName: string
): Promise<{ command: string; output: string; exitCode: number }> {
  const cmd = [
    config.acliBinary,
    "sync",
    "--source",
    config.contentDir,
    "--deck",
    deckName,
    "--collection",
    config.collectionPath,
  ];

  const cmdStr = cmd.join(" ");
  console.log(`  Running: ${cmdStr}`);

  try {
    const result = await $`${cmd}`.quiet().nothrow();
    const output = result.stdout.toString() + result.stderr.toString();
    return { command: cmdStr, output: output.trim(), exitCode: result.exitCode };
  } catch (e) {
    return {
      command: cmdStr,
      output: `ERROR: ${e}`,
      exitCode: 1,
    };
  }
}

async function takeScreenshots(
  config: Config,
  phaseName: string,
  deckName: string,
  collectionPath: string
): Promise<{ browsePath: string | null; cardPaths: string[] }> {
  const browsePath = `${config.screenshotsDir}/${phaseName}_browse.png`;
  const cardPaths: string[] = [];

  // Get the collection directory (parent of collection.anki2)
  const collectionDir = collectionPath.substring(0, collectionPath.lastIndexOf("/"));

  console.log("  Taking screenshot using Python API...");
  
  try {
    // Use the Python script that controls Anki internally
    const result = await $`python3 /home/anki/demo/anki-screenshot.py \
      --collection ${collectionDir} \
      --deck ${deckName} \
      --output ${browsePath} \
      --timeout 12`.quiet().nothrow();
    
    if (result.exitCode === 0 && existsSync(browsePath)) {
      console.log(`  Saved: ${browsePath}`);
    } else {
      console.log("  WARNING: Screenshot script failed");
      console.log(`  Output: ${result.stdout.toString() + result.stderr.toString()}`);
    }
  } catch (e) {
    console.log(`  WARNING: Screenshot error: ${e}`);
  }

  return {
    browsePath: existsSync(browsePath) ? browsePath : null,
    cardPaths,
  };
}

async function runPhase(
  config: Config,
  phase: Phase,
  phaseIndex: number,
  prevMarkdown: string
): Promise<PhaseResult> {
  console.log(`\n${"=".repeat(60)}`);
  console.log(`Phase ${phaseIndex + 1}: ${phase.name}`);
  console.log(`Description: ${phase.description}`);
  console.log(`${"=".repeat(60)}`);

  // Write markdown content
  console.log("\nWriting markdown content...");
  await writeMarkdown(config, phase.markdownContent);

  // Run acli sync
  console.log("\nRunning acli sync...");
  const { command, output, exitCode } = await runAcliSync(
    config,
    DEMO_SCENARIO.deckName
  );
  console.log(`  Exit code: ${exitCode}`);
  if (output) {
    console.log(`  Output: ${output.substring(0, 200)}...`);
  }

  // Take screenshots
  console.log("\nTaking screenshots...");
  const { browsePath, cardPaths } = await takeScreenshots(
    config,
    phase.name,
    DEMO_SCENARIO.deckName,
    config.collectionPath
  );

  return {
    phase,
    prevMarkdown,
    cliCommand: command,
    cliOutput: output,
    cliExitCode: exitCode,
    browseScreenshotPath: browsePath,
    cardScreenshotPaths: cardPaths,
    error: exitCode !== 0 ? output : null,
  };
}

async function getAcliVersion(acliBinary: string): Promise<string> {
  try {
    const result = await $`${acliBinary} --version`.quiet().nothrow();
    return result.stdout.toString().trim() || "unknown";
  } catch {
    return "unknown";
  }
}

function findAcliBinary(): string | null {
  const locations = [
    "/project/target/release/acli",
    "/project/target/debug/acli",
    "/home/anki/acli",
    "/usr/local/bin/acli",
  ];

  for (const loc of locations) {
    if (existsSync(loc)) {
      return loc;
    }
  }

  return null;
}

async function main() {
  const { values } = parseArgs({
    args: Bun.argv.slice(2),
    options: {
      "acli-binary": { type: "string" },
      "output-dir": { type: "string", default: "/home/anki/output" },
      collection: { type: "string" },
      help: { type: "boolean", short: "h" },
    },
  });

  if (values.help) {
    console.log(`Usage: bun run-demo.ts [options]

Options:
  --acli-binary PATH   Path to acli binary (default: auto-detect)
  --output-dir PATH    Output directory (default: /home/anki/output)
  --collection PATH    Anki collection path (default: output-dir/collection.anki2)
  -h, --help           Show this help
`);
    process.exit(0);
  }

  // Find acli binary
  const acliBinary = values["acli-binary"] || findAcliBinary();
  if (!acliBinary) {
    console.error("ERROR: Could not find acli binary.");
    console.error("Please specify with --acli-binary or ensure it exists.");
    process.exit(1);
  }

  const outputDir = values["output-dir"]!;
  const collectionPath =
    values.collection || `${outputDir}/anki_collection/collection.anki2`;

  const config: Config = {
    acliBinary,
    outputDir,
    collectionPath,
    contentDir: `${outputDir}/content`,
    screenshotsDir: `${outputDir}/screenshots`,
  };

  // Create directories
  ensureDir(config.outputDir);
  ensureDir(config.contentDir);
  ensureDir(config.screenshotsDir);
  ensureDir(collectionPath.substring(0, collectionPath.lastIndexOf("/")));

  console.log("\n" + "=".repeat(60));
  console.log("ACLI DEMO RUNNER (Bun/TypeScript)");
  console.log("=".repeat(60));
  console.log(`Scenario: ${DEMO_SCENARIO.name}`);
  console.log(`Deck: ${DEMO_SCENARIO.deckName}`);
  console.log(`Phases: ${DEMO_SCENARIO.phases.length}`);
  console.log(`Output: ${config.outputDir}`);
  console.log(`Collection: ${config.collectionPath}`);
  console.log(`acli binary: ${config.acliBinary}`);

  // Run each phase
  const phaseResults: PhaseResult[] = [];
  let prevMarkdown = "";

  for (let i = 0; i < DEMO_SCENARIO.phases.length; i++) {
    const phase = DEMO_SCENARIO.phases[i];
    const result = await runPhase(config, phase, i, prevMarkdown);
    phaseResults.push(result);
    prevMarkdown = phase.markdownContent;
  }

  // Generate report
  console.log("\n" + "=".repeat(60));
  console.log("Generating HTML report...");
  console.log("=".repeat(60));

  const report: DemoReport = {
    title: DEMO_SCENARIO.name,
    deckName: DEMO_SCENARIO.deckName,
    generatedAt: new Date(),
    phases: phaseResults,
    acliVersion: await getAcliVersion(config.acliBinary),
  };

  const reportPath = `${config.outputDir}/demo_report.html`;
  await saveReport(report, reportPath);

  console.log("\nDemo complete!");
  console.log(`Report: ${reportPath}`);
}

main().catch((e) => {
  console.error("ERROR:", e);
  process.exit(1);
});
