/**
 * Demo scenario definitions for acli.
 *
 * Defines the phases of the demo:
 * 1. Initial sync - Create some cards from markdown
 * 2. Add cards - Add new cards to the markdown
 * 3. Update cards - Modify existing cards
 * 4. Delete cards - Remove some cards
 */

export interface Phase {
  name: string;
  description: string;
  markdownContent: string;
  changeSummary: string;
  expectedCardCount: number;
}

export interface Scenario {
  name: string;
  deckName: string;
  phases: Phase[];
}

export const DEMO_SCENARIO: Scenario = {
  name: "acli Demo: Markdown to Anki Sync",
  deckName: "Demo::Languages",
  phases: [
    {
      name: "initial",
      description: "Initial sync - Creating cards from markdown",
      changeSummary: "Starting fresh with 4 vocabulary cards",
      expectedCardCount: 4,
      markdownContent: `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios
    - Numbers
        - One -> Uno
        - Two -> Dos
`,
    },
    {
      name: "add",
      description: "Adding new cards",
      changeSummary: "Added 3 new cards: 'Three', 'Good morning', 'Good night'",
      expectedCardCount: 7,
      markdownContent: `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios
        - Good morning <-> Buenos dias
        - Good night <-> Buenas noches
    - Numbers
        - One -> Uno
        - Two -> Dos
        - Three -> Tres
`,
    },
    {
      name: "update",
      description: "Updating existing cards",
      changeSummary: "Updated 'Goodbye' answer to include 'Hasta luego'",
      expectedCardCount: 7,
      markdownContent: `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios / Hasta luego
        - Good morning <-> Buenos dias
        - Good night <-> Buenas noches
    - Numbers
        - One -> Uno
        - Two -> Dos
        - Three -> Tres
`,
    },
    {
      name: "delete",
      description: "Removing cards",
      changeSummary: "Removed 'Good night' and 'Three' cards",
      expectedCardCount: 5,
      markdownContent: `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios / Hasta luego
        - Good morning <-> Buenos dias
    - Numbers
        - One -> Uno
        - Two -> Dos
`,
    },
  ],
};

/**
 * Generate a unified diff between two strings.
 */
export function generateDiff(prev: string, current: string): string[] {
  const prevLines = prev ? prev.trim().split("\n") : [];
  const currLines = current.trim().split("\n");

  const diff: string[] = [];
  diff.push("--- previous");
  diff.push("+++ current");

  // Simple line-by-line diff (not a true unified diff, but good enough for demo)
  const maxLen = Math.max(prevLines.length, currLines.length);

  for (let i = 0; i < maxLen; i++) {
    const prevLine = prevLines[i];
    const currLine = currLines[i];

    if (prevLine === currLine) {
      diff.push(` ${currLine}`);
    } else if (prevLine === undefined) {
      diff.push(`+${currLine}`);
    } else if (currLine === undefined) {
      diff.push(`-${prevLine}`);
    } else {
      diff.push(`-${prevLine}`);
      diff.push(`+${currLine}`);
    }
  }

  return diff;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Generate HTML-formatted diff display.
 */
export function getDiffHtml(prev: string, current: string): string {
  if (!prev) {
    return `<pre class="markdown-content">${escapeHtml(current)}</pre>`;
  }

  const diffLines = generateDiff(prev, current);
  const htmlParts = ['<pre class="diff">'];

  for (const line of diffLines) {
    if (line.startsWith("+++") || line.startsWith("---")) {
      htmlParts.push(`<span class="diff-header">${escapeHtml(line)}</span>`);
    } else if (line.startsWith("+")) {
      htmlParts.push(`<span class="diff-add">${escapeHtml(line)}</span>`);
    } else if (line.startsWith("-")) {
      htmlParts.push(`<span class="diff-del">${escapeHtml(line)}</span>`);
    } else {
      htmlParts.push(`<span class="diff-ctx">${escapeHtml(line)}</span>`);
    }
  }

  htmlParts.push("</pre>");
  return htmlParts.join("");
}

if (import.meta.main) {
  console.log(`Scenario: ${DEMO_SCENARIO.name}`);
  console.log(`Deck: ${DEMO_SCENARIO.deckName}`);
  console.log(`Phases: ${DEMO_SCENARIO.phases.length}`);
  console.log();

  for (let i = 0; i < DEMO_SCENARIO.phases.length; i++) {
    const phase = DEMO_SCENARIO.phases[i];
    console.log(`Phase ${i + 1}: ${phase.name}`);
    console.log(`  Description: ${phase.description}`);
    console.log(`  Changes: ${phase.changeSummary}`);
    console.log(`  Expected cards: ${phase.expectedCardCount}`);
    console.log();
  }
}
