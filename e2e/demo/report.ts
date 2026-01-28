/**
 * HTML report generator for the acli demo.
 */
import { readFileSync, existsSync } from "fs";
import type { Phase } from "./scenario";
import { getDiffHtml } from "./scenario";

export interface PhaseResult {
  phase: Phase;
  prevMarkdown: string;
  cliCommand: string;
  cliOutput: string;
  cliExitCode: number;
  browseScreenshotPath: string | null;
  cardScreenshotPaths: string[];
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

      const screenshotsHtml =
        result.browseScreenshotPath || result.cardScreenshotPaths.length > 0
          ? `<div class="section">
              <h3 class="section-title">Anki Screenshots</h3>
              <div class="screenshots">
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
            
            ${diffHtml}
            
            <div class="section">
              <h3 class="section-title">CLI Execution</h3>
              <div class="cli-output">
                <div class="cli-command">${escapeHtml(result.cliCommand)}</div>
                <div class="cli-result">${escapeHtml(result.cliOutput)}</div>
              </div>
            </div>
            
            ${screenshotsHtml}
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
    
    .container { max-width: 1200px; margin: 0 auto; }
    
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
    
    .diff-header { color: var(--text-secondary); font-weight: bold; }
    .diff-add { color: var(--success); background: rgba(0, 210, 106, 0.1); display: block; }
    .diff-del { color: var(--error); background: rgba(255, 71, 87, 0.1); display: block; }
    
    .screenshots {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
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
  </style>
</head>
<body>
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
