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
import { saveReport, type DemoReport, type PhaseResult, type CardData, type ReviewTestResult } from "./report";

// Get the directory where this script is located
const SCRIPT_DIR = import.meta.dir;

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

async function queryCards(
  config: Config,
  deckName: string
): Promise<CardData[]> {
  // Find the Python query script
  const scriptLocations = [
    `${SCRIPT_DIR}/anki-query-cards.py`,
    "/home/anki/demo/anki-query-cards.py",
    `${process.cwd()}/e2e/demo/anki-query-cards.py`,
  ];

  let scriptPath: string | null = null;
  for (const loc of scriptLocations) {
    if (existsSync(loc)) {
      scriptPath = loc;
      break;
    }
  }

  if (!scriptPath) {
    console.log("  WARNING: Could not find anki-query-cards.py");
    return [];
  }

  // Get the collection directory
  const collectionDir = config.collectionPath.substring(
    0,
    config.collectionPath.lastIndexOf("/")
  );

  try {
    console.log(`  Querying cards from collection...`);
    const result = await $`python3 ${scriptPath} \
      --collection ${collectionDir} \
      --deck ${deckName}`.quiet().nothrow();

    if (result.exitCode !== 0) {
      console.log(`  Card query failed: ${result.stderr.toString()}`);
      return [];
    }

    const stdout = result.stdout.toString().trim();
    if (!stdout) {
      console.log("  No cards returned from query");
      return [];
    }

    // Parse JSON output
    const rawCards = JSON.parse(stdout) as Array<{
      front: string;
      back: string;
      cardType: string;
      deckPath: string;
      tags: string;
      cardId: number;
      ordinal: number;
      question: string;
    }>;

    // Map query results to CardData
    const cards: CardData[] = rawCards.map((raw) => ({
      front: raw.front,
      back: raw.back,
      cardType: raw.cardType === "reversible" ? "reversible" : "one-way",
      path: raw.deckPath,
      cardId: raw.cardId,
    }));

    console.log(`  Found ${cards.length} cards in collection`);
    return cards;
  } catch (e) {
    console.log(`  Error querying cards: ${e}`);
    return [];
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

  // Find the Python screenshot script - check multiple locations
  const scriptLocations = [
    `${SCRIPT_DIR}/anki-screenshot.py`,           // Same directory as this script
    "/home/anki/demo/anki-screenshot.py",          // Container path
    `${process.cwd()}/e2e/demo/anki-screenshot.py`, // Project root
  ];

  let scriptPath: string | null = null;
  for (const loc of scriptLocations) {
    if (existsSync(loc)) {
      scriptPath = loc;
      break;
    }
  }

  if (!scriptPath) {
    console.log("  ERROR: Could not find anki-screenshot.py");
    console.log(`  Searched: ${scriptLocations.join(", ")}`);
    return { browsePath: null, cardPaths };
  }

  // Retry logic for screenshot capture
  const maxRetries = 3;
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    console.log(`  Taking screenshot using Python API (attempt ${attempt}/${maxRetries})...`);

    try {
      // Use the Python script that controls Anki internally
      console.log(`  Running: python3 ${scriptPath} --collection ${collectionDir} --deck "${deckName}" --output ${browsePath} --timeout 30`);
      const result = await $`python3 -u ${scriptPath} \
        --collection ${collectionDir} \
        --deck ${deckName} \
        --output ${browsePath} \
        --timeout 30`.nothrow();

      const output = result.stdout.toString() + result.stderr.toString();

      if (result.exitCode === 0 && existsSync(browsePath)) {
        console.log(`  Saved: ${browsePath}`);
        return { browsePath, cardPaths };
      } else {
        console.log(`  Attempt ${attempt} failed (exit code: ${result.exitCode})`);
        if (output.trim()) {
          console.log(`  Output: ${output.substring(0, 500)}`);
        }

        if (attempt < maxRetries) {
          console.log(`  Retrying in 2 seconds...`);
          await sleep(2000);
        }
      }
    } catch (e) {
      console.log(`  Attempt ${attempt} error: ${e}`);
      if (attempt < maxRetries) {
        await sleep(2000);
      }
    }
  }

  console.log("  ERROR: All screenshot attempts failed");
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

  // Query card data from collection
  console.log("\nQuerying card data...");
  const cards = await queryCards(
    config,
    DEMO_SCENARIO.deckName
  );

  return {
    phase,
    prevMarkdown,
    cliCommand: command,
    cliOutput: output,
    cliExitCode: exitCode,
    browseScreenshotPath: browsePath,
    cardScreenshotPaths: cardPaths,
    cards,
    error: exitCode !== 0 ? output : null,
  };
}

// === Review preservation test ===

async function runReviewCard(
  config: Config,
  cardId: number,
  rating: string
): Promise<{ output: string; exitCode: number }> {
  const cmd = [
    config.acliBinary,
    "review-card",
    "--card-id",
    String(cardId),
    "--rating",
    rating,
    "--collection",
    config.collectionPath,
  ];

  console.log(`  Running: ${cmd.join(" ")}`);
  try {
    const result = await $`${cmd}`.quiet().nothrow();
    const output = result.stdout.toString() + result.stderr.toString();
    return { output: output.trim(), exitCode: result.exitCode };
  } catch (e) {
    return { output: `ERROR: ${e}`, exitCode: 1 };
  }
}

async function runGetReviews(
  config: Config,
  cardId: number
): Promise<{ reviews: any[]; output: string; exitCode: number }> {
  const cmd = [
    config.acliBinary,
    "get-reviews",
    "--card-id",
    String(cardId),
    "--collection",
    config.collectionPath,
  ];

  console.log(`  Running: ${cmd.join(" ")}`);
  try {
    const result = await $`${cmd}`.quiet().nothrow();
    const stdout = result.stdout.toString().trim();
    const stderr = result.stderr.toString().trim();

    if (result.exitCode !== 0) {
      return { reviews: [], output: stderr || stdout, exitCode: result.exitCode };
    }

    try {
      const reviews = JSON.parse(stdout);
      return { reviews, output: stdout, exitCode: 0 };
    } catch {
      return { reviews: [], output: stdout, exitCode: 0 };
    }
  } catch (e) {
    return { reviews: [], output: `ERROR: ${e}`, exitCode: 1 };
  }
}

async function testReviewPreservation(
  config: Config,
  deckName: string,
  markdownContent: string
): Promise<ReviewTestResult> {
  console.log(`\n${"=".repeat(60)}`);
  console.log("Review Preservation Test");
  console.log(`${"=".repeat(60)}`);

  // 1. Query cards to get a card ID
  console.log("\nStep 1: Finding a card to review...");
  const cards = await queryCards(config, deckName);
  if (cards.length === 0) {
    return {
      cardId: 0,
      reviewsBefore: 0,
      reviewsAfter: 0,
      passed: false,
      details: "No cards found in deck to test review preservation",
    };
  }

  const testCard = cards[0];
  const cardId = testCard.cardId || 0;
  if (!cardId) {
    return {
      cardId: 0,
      reviewsBefore: 0,
      reviewsAfter: 0,
      passed: false,
      details: "Card has no cardId — cannot test review preservation",
    };
  }

  console.log(`  Selected card ${cardId} (front: ${testCard.front.substring(0, 50)}...)`);

  // 2. Record a review
  console.log("\nStep 2: Recording a review...");
  const reviewResult = await runReviewCard(config, cardId, "good");
  if (reviewResult.exitCode !== 0) {
    return {
      cardId,
      reviewsBefore: 0,
      reviewsAfter: 0,
      passed: false,
      details: `Failed to record review: ${reviewResult.output}`,
    };
  }

  // 3. Verify the review exists
  console.log("\nStep 3: Verifying review was recorded...");
  const beforeResult = await runGetReviews(config, cardId);
  const reviewsBefore = beforeResult.reviews.length;
  console.log(`  Found ${reviewsBefore} review(s) before re-sync`);

  if (reviewsBefore === 0) {
    return {
      cardId,
      reviewsBefore: 0,
      reviewsAfter: 0,
      passed: false,
      details: "Review was not recorded (get-reviews returned 0 reviews)",
    };
  }

  // 4. Re-sync the same content
  console.log("\nStep 4: Re-syncing same content...");
  await writeMarkdown(config, markdownContent);
  const syncResult = await runAcliSync(config, deckName);
  console.log(`  Sync exit code: ${syncResult.exitCode}`);
  if (syncResult.output) {
    console.log(`  Sync output: ${syncResult.output.substring(0, 200)}`);
  }

  // 5. Check reviews survived
  console.log("\nStep 5: Checking reviews survived re-sync...");
  const afterResult = await runGetReviews(config, cardId);
  const reviewsAfter = afterResult.reviews.length;
  console.log(`  Found ${reviewsAfter} review(s) after re-sync`);

  const passed = reviewsAfter >= reviewsBefore;
  const details = passed
    ? `Review preserved: ${reviewsBefore} review(s) before, ${reviewsAfter} after re-sync`
    : `REVIEW LOST: ${reviewsBefore} review(s) before, ${reviewsAfter} after re-sync`;

  console.log(`\n  Result: ${passed ? "PASSED" : "FAILED"} — ${details}`);

  return { cardId, reviewsBefore, reviewsAfter, passed, details };
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
  let reviewTestResult: ReviewTestResult | null = null;

  for (let i = 0; i < DEMO_SCENARIO.phases.length; i++) {
    const phase = DEMO_SCENARIO.phases[i];
    const result = await runPhase(config, phase, i, prevMarkdown);
    phaseResults.push(result);
    prevMarkdown = phase.markdownContent;

    // Run review preservation test after the initial sync phase
    if (phase.name === "initial") {
      reviewTestResult = await testReviewPreservation(
        config,
        DEMO_SCENARIO.deckName,
        phase.markdownContent
      );
    }
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
    reviewTest: reviewTestResult,
  };

  const reportPath = `${config.outputDir}/demo_report.html`;
  await saveReport(report, reportPath);

  console.log("\nDemo complete!");
  console.log(`Report: ${reportPath}`);

  // Fail the demo if the review preservation test failed
  if (reviewTestResult && !reviewTestResult.passed) {
    console.error("\nERROR: Review preservation test FAILED!");
    console.error(`  ${reviewTestResult.details}`);
    process.exit(1);
  }
}

main().catch((e) => {
  console.error("ERROR:", e);
  process.exit(1);
});
