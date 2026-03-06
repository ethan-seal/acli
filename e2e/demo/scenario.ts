/**
 * Demo scenario definitions for acli.
 *
 * Defines the phases of the demo:
 * 1. Initial sync - Create cards using basic (->) and bidirectional (<->) syntax
 * 2. Add cards    - Add sequence (=>) and attribute (heading + ->) cards
 * 3. Update cards - Add pipe-table cards; modify existing content
 * 4. Delete cards - Remove some cards
 */

export interface Phase {
  name: string;
  description: string;
  markdownContent: string;
  changeSummary: string;
  expectedCardCount: number;
  /** Optional files to create in the content directory before syncing. */
  mediaFiles?: Array<{ name: string; content: string }>;
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
      changeSummary: "Starting fresh with 4 vocabulary cards (basic and bidirectional)",
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
      description: "Adding new card types: sequence and attribute cards",
      changeSummary:
        "Added 3 sequence cards for a process and 3 attribute cards under a heading",
      expectedCardCount: 10,
      markdownContent: `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios
    - Numbers
        - One -> Uno
        - Two -> Dos

# Photosynthesis
- inputs -> CO2 + H2O + sunlight
- outputs -> glucose + oxygen
- location -> chloroplasts

Boot sequence
=> power on
=> BIOS/UEFI loads
=> bootloader runs
`,
    },
    {
      name: "update",
      description: "Updating existing cards and adding a pipe-table",
      changeSummary:
        "Updated 'Goodbye' to include 'Hasta luego'; added a verb-conjugation pipe table (6 cards)",
      expectedCardCount: 16,
      markdownContent: `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios / Hasta luego
    - Numbers
        - One -> Uno
        - Two -> Dos

# Photosynthesis
- inputs -> CO2 + H2O + sunlight
- outputs -> glucose + oxygen
- location -> chloroplasts

Boot sequence
=> power on
=> BIOS/UEFI loads
=> bootloader runs

subject | conjugation <->
yo | soy
tú | eres
él/ella | es
`,
    },
    {
      name: "delete",
      description: "Removing some cards",
      changeSummary:
        "Removed the boot-sequence block and the 'location' attribute card",
      expectedCardCount: 12,
      markdownContent: `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios / Hasta luego
    - Numbers
        - One -> Uno
        - Two -> Dos

# Photosynthesis
- inputs -> CO2 + H2O + sunlight
- outputs -> glucose + oxygen

subject | conjugation <->
yo | soy
tú | eres
él/ella | es
`,
    },
    {
      name: "images",
      description: "Adding cards with local images",
      changeSummary:
        "Added 2 country-flag cards with SVG images; exercises the media copy pipeline",
      expectedCardCount: 14,
      mediaFiles: [
        {
          name: "spain.svg",
          content: `<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80">
  <rect width="120" height="80" fill="#c60b1e"/>
  <rect y="20" width="120" height="40" fill="#ffc400"/>
</svg>`,
        },
        {
          name: "france.svg",
          content: `<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80">
  <rect width="40" height="80" fill="#002395"/>
  <rect x="40" width="40" height="80" fill="#fff"/>
  <rect x="80" width="40" height="80" fill="#ed2939"/>
</svg>`,
        },
      ],
      markdownContent: `- Spanish Vocabulary
    - Greetings
        - Hello <-> Hola
        - Goodbye <-> Adios / Hasta luego
    - Numbers
        - One -> Uno
        - Two -> Dos

# Photosynthesis
- inputs -> CO2 + H2O + sunlight
- outputs -> glucose + oxygen

subject | conjugation <->
yo | soy
tú | eres
él/ella | es

# Country Flags
- ![Spain](spain.svg) -> España
- ![France](france.svg) -> Francia
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
