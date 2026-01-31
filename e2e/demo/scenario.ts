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
 * Parsed card from markdown.
 */
export interface ParsedCard {
  front: string;
  back: string;
  type: "one-way" | "reversible";
  path: string;
}

/**
 * Parse flashcards from markdown content.
 * Recognizes patterns like:
 *   - Question -> Answer  (one-way)
 *   - Question <-> Answer (reversible)
 */
export function parseCardsFromMarkdown(markdown: string): ParsedCard[] {
  if (!markdown) return [];

  const cards: ParsedCard[] = [];
  const lines = markdown.split("\n");

  // Track hierarchy path
  const pathStack: string[] = [];

  for (const line of lines) {
    // Count indentation (assuming 4 spaces per level)
    const stripped = line.replace(/^\s*-\s*/, "");
    const indentMatch = line.match(/^(\s*)/);
    const indent = indentMatch ? Math.floor(indentMatch[1].length / 4) : 0;

    // Check for card patterns
    const reversibleMatch = stripped.match(/^(.+?)\s*<->\s*(.+)$/);
    const oneWayMatch = stripped.match(/^(.+?)\s*->\s*(.+)$/);

    if (reversibleMatch) {
      const path = pathStack.slice(0, indent).join("::");
      cards.push({
        front: reversibleMatch[1].trim(),
        back: reversibleMatch[2].trim(),
        type: "reversible",
        path,
      });
    } else if (oneWayMatch) {
      const path = pathStack.slice(0, indent).join("::");
      cards.push({
        front: oneWayMatch[1].trim(),
        back: oneWayMatch[2].trim(),
        type: "one-way",
        path,
      });
    } else if (stripped.trim()) {
      // It's a hierarchy node, update path stack
      while (pathStack.length > indent) {
        pathStack.pop();
      }
      pathStack[indent] = stripped.trim();
    }
  }

  return cards;
}

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
