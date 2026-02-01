/**
 * HTML report generator for the acli demo.
 */
import { readFileSync, existsSync } from "fs";
import type { Phase } from "./scenario";
import { getDiffHtml } from "./scenario";

export interface CardData {
  front: string;
  back: string;
  cardType: "one-way" | "reversible";
  path: string;
  status: "added" | "updated" | "unchanged" | "deleted";
}

export interface PhaseResult {
  phase: Phase;
  prevMarkdown: string;
  cliCommand: string;
  cliOutput: string;
  cliExitCode: number;
  browseScreenshotPath: string | null;
  cardScreenshotPaths: string[];
  cards: CardData[];
  error: string | null;
}

export interface DemoReport {
  title: string;
  deckName: string;
  generatedAt: Date;
  phases: PhaseResult[];
  acliVersion: string;
}

function embedImage(path: string | null): string {
  if (!path || !existsSync(path)) {
    return "";
  }

  const data = readFileSync(path);
  const base64 = data.toString("base64");

  const ext = path.split(".").pop()?.toLowerCase() || "png";
  const mimeTypes: Record<string, string> = {
    png: "image/png",
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    gif: "image/gif",
  };
  const mimeType = mimeTypes[ext] || "image/png";

  return `data:${mimeType};base64,${base64}`;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function formatDate(date: Date): string {
  return date.toISOString().replace("T", " ").substring(0, 19);
}

export function generateReport(report: DemoReport): string {
  const phasesHtml = report.phases
    .map((result, index) => {
      const phaseNum = index + 1;
      const statusClass =
        result.error || result.cliExitCode !== 0
          ? "status-error"
          : "status-success";
      const statusText =
        result.error || result.cliExitCode !== 0 ? "Error" : "Success";

      const diffHtml =
        phaseNum > 1
          ? `<div class="section">
              <h3 class="section-title">Markdown Changes</h3>
              ${getDiffHtml(result.prevMarkdown, result.phase.markdownContent)}
            </div>`
          : `<div class="section">
              <h3 class="section-title">Initial Markdown Content</h3>
              <pre class="markdown-content">${escapeHtml(result.phase.markdownContent)}</pre>
            </div>`;

      // Generate card table HTML
      const cardTableHtml = result.cards.length > 0
        ? `<table class="card-table">
            <thead>
              <tr>
                <th>Front</th>
                <th>Back</th>
                <th>Path</th>
              </tr>
            </thead>
            <tbody>
              ${result.cards
                .map(
                  (card) => `<tr class="card-row card-status-${card.status}">
                  <td class="card-front"><div class="card-front-content">${card.front}</div></td>
                  <td class="card-back"><div class="card-back-content">${card.back}</div></td>
                  <td class="card-path-cell">
                    <div class="card-path">${escapeHtml(card.path)}</div>
                    <span class="type-badge type-${card.cardType}">${card.cardType === "reversible" ? "Reversible" : "One-way"}</span>
                  </td>
                </tr>`
                )
                .join("\n")}
            </tbody>
          </table>`
        : `<div class="no-cards">No cards in collection</div>`;

      const hasScreenshots = result.browseScreenshotPath || result.cardScreenshotPaths.length > 0;
      const hasCards = result.cards.length > 0;
      const showTabs = hasScreenshots || hasCards;

      const screenshotsContent = hasScreenshots
        ? `<div class="screenshots">
            ${
              result.browseScreenshotPath
                ? `<div class="screenshot">
                <img src="${embedImage(result.browseScreenshotPath)}" alt="Browse Window">
                <div class="screenshot-caption">Browse Window - Card List</div>
              </div>`
                : ""
            }
            ${result.cardScreenshotPaths
              .map(
                (path, i) => `<div class="screenshot">
                <img src="${embedImage(path)}" alt="Card Preview ${i + 1}">
                <div class="screenshot-caption">Card Preview ${i + 1}</div>
              </div>`
              )
              .join("\n")}
          </div>`
        : `<div class="no-screenshots">No screenshots available</div>`;

      const tabbedContentHtml = showTabs
        ? `<div class="section">
            <div class="tabs" data-phase="${phaseNum}">
              <button class="tab-btn active" data-tab="screenshots-${phaseNum}">Screenshots</button>
              <button class="tab-btn" data-tab="cards-${phaseNum}">Card Table</button>
            </div>
            <div class="tab-content active" id="screenshots-${phaseNum}">
              ${screenshotsContent}
            </div>
            <div class="tab-content" id="cards-${phaseNum}">
              ${cardTableHtml}
            </div>
          </div>`
        : "";

      const errorHtml = result.error
        ? `<div class="section">
            <h3 class="section-title">Error</h3>
            <div class="cli-output" style="border-left: 3px solid var(--error);">
              ${escapeHtml(result.error)}
            </div>
          </div>`
        : "";

      return `
        <div class="phase">
          <div class="phase-header">
            <div class="phase-number">${phaseNum}</div>
            <div class="phase-title">
              <h2>${escapeHtml(result.phase.description)}</h2>
              <p>${escapeHtml(result.phase.changeSummary)}</p>
            </div>
            <div style="margin-left: auto;">
              <span class="status-badge ${statusClass}">${statusText}</span>
            </div>
          </div>
          
          <div class="phase-content">
            <div class="summary-grid">
              <div class="summary-item">
                <div class="label">Expected Cards</div>
                <div class="value">${result.phase.expectedCardCount}</div>
              </div>
              <div class="summary-item">
                <div class="label">Exit Code</div>
                <div class="value">${result.cliExitCode}</div>
              </div>
            </div>
            
            <div class="content-columns">
              <div class="content-left">
                ${diffHtml}
                
                <div class="section">
                  <h3 class="section-title">CLI Execution</h3>
                  <div class="cli-output">
                    <div class="cli-command">${escapeHtml(result.cliCommand)}</div>
                    <div class="cli-result">${escapeHtml(result.cliOutput)}</div>
                  </div>
                </div>
              </div>
              
              ${tabbedContentHtml ? `<div class="content-right">${tabbedContentHtml}</div>` : ""}
            </div>
            
            ${errorHtml}
          </div>
        </div>
      `;
    })
    .join("\n");

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(report.title)}</title>
  <style>
    :root {
      --bg-primary: #1a1a2e;
      --bg-secondary: #16213e;
      --bg-card: #0f3460;
      --text-primary: #eaeaea;
      --text-secondary: #b8b8b8;
      --accent: #e94560;
      --success: #00d26a;
      --warning: #ffc107;
      --error: #ff4757;
      --border: #2a2a4a;
    }
    
    * { box-sizing: border-box; margin: 0; padding: 0; }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: var(--bg-primary);
      color: var(--text-primary);
      line-height: 1.6;
      padding: 2rem;
    }
    
    .container { max-width: 1400px; margin: 0 auto; }
    
    header {
      text-align: center;
      margin-bottom: 3rem;
      padding-bottom: 2rem;
      border-bottom: 1px solid var(--border);
    }
    
    header h1 {
      font-size: 2.5rem;
      margin-bottom: 0.5rem;
      background: linear-gradient(135deg, var(--accent), #ff6b6b);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    header .meta { color: var(--text-secondary); font-size: 0.9rem; }
    
    .phase {
      background: var(--bg-secondary);
      border-radius: 12px;
      margin-bottom: 2rem;
      overflow: hidden;
      border: 1px solid var(--border);
    }
    
    .phase-header {
      background: var(--bg-card);
      padding: 1.5rem;
      display: flex;
      align-items: center;
      gap: 1rem;
    }
    
    .phase-number {
      background: var(--accent);
      color: white;
      width: 40px;
      height: 40px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: bold;
      font-size: 1.2rem;
    }
    
    .phase-title h2 { font-size: 1.3rem; margin-bottom: 0.25rem; }
    .phase-title p { color: var(--text-secondary); font-size: 0.9rem; }
    
    .phase-content { padding: 1.5rem; }
    
    .content-columns {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 1.5rem;
      align-items: start;
    }
    
    .content-left {
      min-width: 0;
    }
    
    .content-right .section { margin-bottom: 0; }
    .content-right .screenshots {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }
    
    .section { margin-bottom: 1.5rem; }
    .section:last-child { margin-bottom: 0; }
    
    .section-title {
      font-size: 1rem;
      font-weight: 600;
      margin-bottom: 0.75rem;
      color: var(--accent);
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
    
    .cli-output {
      background: #0d1117;
      border-radius: 8px;
      padding: 1rem;
      font-family: 'Monaco', 'Menlo', monospace;
      font-size: 0.85rem;
      overflow-x: auto;
    }
    
    .cli-command {
      color: var(--success);
      margin-bottom: 0.5rem;
      padding-bottom: 0.5rem;
      border-bottom: 1px solid var(--border);
    }
    .cli-command::before { content: "$ "; color: var(--text-secondary); }
    
    .cli-result { color: var(--text-secondary); white-space: pre-wrap; }
    
    .diff, .markdown-content {
      background: #0d1117;
      border-radius: 8px;
      padding: 1rem;
      font-family: 'Monaco', 'Menlo', monospace;
      font-size: 0.85rem;
      overflow-x: auto;
      white-space: pre;
    }
    
    .diff-header { color: var(--text-secondary); font-weight: bold; display: block; }
    .diff-add { color: var(--success); background: rgba(0, 210, 106, 0.1); display: block; }
    .diff-del { color: var(--error); background: rgba(255, 71, 87, 0.1); display: block; }
    .diff-ctx { display: block; }
    
    .screenshots {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }
    
    .screenshot {
      background: var(--bg-card);
      border-radius: 8px;
      overflow: hidden;
    }
    
    .screenshot img { width: 100%; height: auto; display: block; }
    
    .screenshot-caption {
      padding: 0.75rem;
      text-align: center;
      color: var(--text-secondary);
      font-size: 0.85rem;
    }
    
    .status-badge {
      display: inline-block;
      padding: 0.25rem 0.75rem;
      border-radius: 9999px;
      font-size: 0.75rem;
      font-weight: 600;
      text-transform: uppercase;
    }
    
    .status-success { background: rgba(0, 210, 106, 0.2); color: var(--success); }
    .status-error { background: rgba(255, 71, 87, 0.2); color: var(--error); }
    
    .summary-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
      gap: 1rem;
      margin-bottom: 1rem;
    }
    
    .summary-item {
      background: var(--bg-card);
      padding: 1rem;
      border-radius: 8px;
      text-align: center;
    }
    
    .summary-item .label {
      color: var(--text-secondary);
      font-size: 0.8rem;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
    
    .summary-item .value {
      font-size: 1.5rem;
      font-weight: bold;
      margin-top: 0.25rem;
    }
    
    footer {
      text-align: center;
      margin-top: 3rem;
      padding-top: 2rem;
      border-top: 1px solid var(--border);
      color: var(--text-secondary);
      font-size: 0.85rem;
    }
    
    /* Tabs */
    .tabs {
      display: flex;
      gap: 0;
      margin-bottom: 0;
      border-bottom: 1px solid var(--border);
    }
    
    .tab-btn {
      background: transparent;
      border: none;
      color: var(--text-secondary);
      padding: 0.75rem 1.5rem;
      cursor: pointer;
      font-size: 0.9rem;
      font-weight: 500;
      border-bottom: 2px solid transparent;
      transition: all 0.2s ease;
    }
    
    .tab-btn:hover {
      color: var(--text-primary);
      background: rgba(255, 255, 255, 0.05);
    }
    
    .tab-btn.active {
      color: var(--accent);
      border-bottom-color: var(--accent);
    }
    
    .tab-content {
      display: none;
      padding-top: 1rem;
    }
    
    .tab-content.active {
      display: block;
    }
    
    /* Card Table */
    .card-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.85rem;
    }
    
    .card-table th,
    .card-table td {
      padding: 0.75rem;
      text-align: left;
      border-bottom: 1px solid var(--border);
    }
    
    .card-table th {
      background: var(--bg-card);
      color: var(--text-secondary);
      font-weight: 600;
      text-transform: uppercase;
      font-size: 0.75rem;
      letter-spacing: 0.5px;
    }
    
    .card-table td {
      color: var(--text-primary);
    }
    
    .card-row:hover {
      background: rgba(255, 255, 255, 0.03);
    }
    
    .card-front {
      font-weight: 500;
    }
    
    .card-front-content {
      font-size: 0.9rem;
      line-height: 1.4;
    }
    
    .card-front-content ul {
      margin: 0;
      padding-left: 1.5rem;
      list-style-type: disc;
    }
    
    .card-front-content li {
      margin: 0.25rem 0;
    }
    
    .card-back-content {
      font-size: 0.9rem;
      line-height: 1.4;
    }
    
    .card-path-cell {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
    
    .card-path {
      color: var(--text-secondary);
      font-size: 0.8rem;
    }
    
    .type-badge {
      display: inline-block;
      padding: 0.2rem 0.5rem;
      border-radius: 4px;
      font-size: 0.7rem;
      font-weight: 600;
      text-transform: uppercase;
      width: fit-content;
    }
    
    .type-badge.type-reversible {
      background: rgba(138, 138, 255, 0.2);
      color: #8a8aff;
    }
    
    .type-badge.type-one-way {
      background: rgba(138, 180, 248, 0.2);
      color: #8ab4f8;
    }
    
    .status-pill {
      display: inline-block;
      padding: 0.2rem 0.5rem;
      border-radius: 4px;
      font-size: 0.7rem;
      font-weight: 600;
      text-transform: uppercase;
    }
    
    .status-added {
      background: rgba(0, 210, 106, 0.2);
      color: var(--success);
    }
    
    .status-updated {
      background: rgba(255, 193, 7, 0.2);
      color: var(--warning);
    }
    
    .status-unchanged {
      background: rgba(184, 184, 184, 0.15);
      color: var(--text-secondary);
    }
    
    .status-deleted {
      background: rgba(255, 71, 87, 0.2);
      color: var(--error);
    }
    
    .card-status-added {
      background: rgba(0, 210, 106, 0.05);
    }
    
    .card-status-updated {
      background: rgba(255, 193, 7, 0.05);
    }
    
    .no-cards, .no-screenshots {
      padding: 2rem;
      text-align: center;
      color: var(--text-secondary);
      font-style: italic;
    }
  </style>
</head>
<body>
  <script>
    document.addEventListener('DOMContentLoaded', function() {
      // Tab switching functionality
      document.querySelectorAll('.tab-btn').forEach(function(btn) {
        btn.addEventListener('click', function() {
          var tabId = this.getAttribute('data-tab');
          var tabsContainer = this.closest('.tabs');
          var contentContainer = tabsContainer.parentElement;
          
          // Deactivate all tabs in this group
          tabsContainer.querySelectorAll('.tab-btn').forEach(function(b) {
            b.classList.remove('active');
          });
          
          // Hide all content in this group
          contentContainer.querySelectorAll('.tab-content').forEach(function(c) {
            c.classList.remove('active');
          });
          
          // Activate clicked tab and show its content
          this.classList.add('active');
          document.getElementById(tabId).classList.add('active');
        });
      });
    });
  </script>
  <div class="container">
    <header>
      <h1>${escapeHtml(report.title)}</h1>
      <div class="meta">
        <p>Deck: <strong>${escapeHtml(report.deckName)}</strong></p>
        <p>Generated: ${formatDate(report.generatedAt)}</p>
        <p>acli version: ${escapeHtml(report.acliVersion)}</p>
      </div>
    </header>
    
    ${phasesHtml}
    
    <footer>
      <p>Generated by acli e2e demo runner</p>
    </footer>
  </div>
</body>
</html>`;
}

export async function saveReport(
  report: DemoReport,
  outputPath: string
): Promise<void> {
  const html = generateReport(report);
  await Bun.write(outputPath, html);
  console.log(`Report generated: ${outputPath}`);
}
