#!/usr/bin/env node
/**
 * Simple test for the card-diff module.
 */
import { parseCards, computeCardDiff, getCardDiffHtml } from "./card-diff";

// Test 1: Parse cards from markdown
console.log("Test 1: Parse cards from markdown");
const markdown1 = `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios
    - Numbers
        - One -> Uno
        - Two -> Dos
`;

const cards1 = parseCards(markdown1);
console.log(`  Found ${cards1.length} cards`);
console.log(`  Card 1: ${JSON.stringify(cards1[0])}`);
console.log(`  Card 2: ${JSON.stringify(cards1[1])}`);

// Test 2: Compute diff between two versions
console.log("\nTest 2: Compute diff (add cards)");
const markdown2 = `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios
        - Good morning <-> Buenos dias
    - Numbers
        - One -> Uno
        - Two -> Dos
        - Three -> Tres
`;

const diff = computeCardDiff(markdown1, markdown2);
console.log(`  Added: ${diff.added.length} cards`);
console.log(`  Deleted: ${diff.deleted.length} cards`);
console.log(`  Updated: ${diff.updated.length} cards`);

if (diff.added.length > 0) {
  console.log(`  First added card: ${JSON.stringify(diff.added[0])}`);
}

// Test 3: Generate HTML diff
console.log("\nTest 3: Generate HTML diff");
const html = getCardDiffHtml(markdown1, markdown2);
console.log(`  Generated ${html.length} bytes of HTML`);
console.log(`  Contains "Added": ${html.includes("Added")}`);
console.log(`  Contains "diff-add": ${html.includes("diff-add")}`);

// Test 4: Update scenario (content change)
console.log("\nTest 4: Compute diff (update card - content change)");
const markdown3 = `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios / Hasta luego
    - Numbers
        - One -> Uno
        - Two -> Dos
`;

const diff2 = computeCardDiff(markdown1, markdown3);
console.log(`  Added: ${diff2.added.length} cards (should be 1 - new version)`);
console.log(`  Deleted: ${diff2.deleted.length} cards (should be 1 - old version)`);
console.log(`  Updated: ${diff2.updated.length} cards (should be 0 - content-based IDs)`);

// Test 5: Delete scenario
console.log("\nTest 5: Compute diff (delete cards)");
const markdown4 = `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
    - Numbers
        - One -> Uno
`;

const diff3 = computeCardDiff(markdown1, markdown4);
console.log(`  Added: ${diff3.added.length} cards`);
console.log(`  Deleted: ${diff3.deleted.length} cards (should be 2)`);

console.log("\nAll tests completed!");
