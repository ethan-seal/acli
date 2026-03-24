use crate::error::ParseError;
use crate::types::Warning;

use super::pipe_table::split_pipe_cells;

/// Expand template blocks in the markdown.
///
/// A template block is a code fence with `template` language followed by a
/// pipe table.  Each data row is expanded through the template using simple
/// `{{ column }}` substitution, and the template+table is replaced with the
/// expanded text.
///
/// Returns the markdown with all template blocks expanded.  Non-template
/// content passes through unchanged.
pub(super) fn expand_templates(markdown: &str) -> Result<(String, Vec<Warning>), ParseError> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut warnings: Vec<Warning> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        // Look for a template code fence opening.
        let trimmed = lines[i].trim();
        if trimmed == "```template" || trimmed == "~~~template" {
            let fence_char = if trimmed.starts_with('`') { '`' } else { '~' };
            let fence_open = i;
            i += 1;

            // Collect template body until closing fence.
            let mut template_lines: Vec<&str> = Vec::new();
            let mut fence_close = None;
            while i < lines.len() {
                let t = lines[i].trim();
                if (fence_char == '`' && t.starts_with("```"))
                    || (fence_char == '~' && t.starts_with("~~~"))
                {
                    fence_close = Some(i);
                    i += 1;
                    break;
                }
                template_lines.push(lines[i]);
                i += 1;
            }

            let fence_close = match fence_close {
                Some(fc) => fc,
                None => {
                    // Unclosed fence — pass through as-is.
                    for line in &lines[fence_open..] {
                        result_lines.push(line.to_string());
                    }
                    break;
                }
            };

            let template = template_lines.join("\n");

            // Collect the pipe table (header + data rows).
            // The table must start on the very next line — no blank lines
            // between the closing fence and the header row.
            if i >= lines.len() || !lines[i].contains('|') {
                // No pipe table immediately after template — warn and pass
                // through as-is.
                warnings.push(Warning::new(
                    fence_open + 1,
                    "template block has no pipe table immediately after it",
                ));
                for line in &lines[fence_open..=fence_close] {
                    result_lines.push(line.to_string());
                }
                continue;
            }

            // Parse header row.
            let header_cells: Vec<&str> = split_pipe_cells(lines[i])
                .iter()
                .map(|s| s.trim())
                .collect::<Vec<_>>()
                .into_iter()
                .collect();
            let header_names: Vec<String> = header_cells.iter().map(|s| s.to_string()).collect();
            i += 1;

            // Parse data rows.
            let mut data_rows: Vec<Vec<String>> = Vec::new();
            while i < lines.len() && lines[i].contains('|') {
                let cells: Vec<String> = split_pipe_cells(lines[i])
                    .iter()
                    .map(|s| s.trim().to_string())
                    .collect();
                data_rows.push(cells);
                i += 1;
            }

            // Expand template for each row.
            for (row_idx, row) in data_rows.iter().enumerate() {
                // Check for empty or missing cells — every placeholder must
                // have a value.
                for (col_idx, col_name) in header_names.iter().enumerate() {
                    let cell = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                    if cell.is_empty() {
                        return Err(ParseError::TemplateError(format!(
                            "empty cell for column '{}' in row {} of template at line {}",
                            col_name,
                            row_idx + 1,
                            fence_open + 1,
                        )));
                    }
                }

                let expanded = expand_template_row(&template, &header_names, row, fence_open)?;

                // Add blank line between expanded rows for block card separation.
                if row_idx > 0 {
                    result_lines.push(String::new());
                }
                for line in expanded.lines() {
                    result_lines.push(line.to_string());
                }
            }
        } else {
            result_lines.push(lines[i].to_string());
            i += 1;
        }
    }

    Ok((result_lines.join("\n"), warnings))
}

/// Expand a single template row by replacing `{{ column }}` placeholders.
///
/// Handles optional whitespace inside the braces: `{{name}}`, `{{ name }}`,
/// `{{ name}}`, etc.  Reports an error for unknown placeholders.
fn expand_template_row(
    template: &str,
    header_names: &[String],
    row: &[String],
    fence_open: usize,
) -> Result<String, ParseError> {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        // Copy everything before the placeholder.
        result.push_str(&rest[..start]);

        let after_open = &rest[start + 2..];
        match after_open.find("}}") {
            Some(end) => {
                let name = after_open[..end].trim();
                // Look up the column name.
                let col_idx = header_names.iter().position(|h| h == name);
                match col_idx {
                    Some(idx) => {
                        let value = row.get(idx).map(|s| s.as_str()).unwrap_or("");
                        result.push_str(value);
                    }
                    None => {
                        return Err(ParseError::TemplateError(format!(
                            "unknown placeholder '{}' in template at line {}",
                            name,
                            fence_open + 1,
                        )));
                    }
                }
                rest = &after_open[end + 2..];
            }
            None => {
                // No closing `}}` — copy literally and stop.
                result.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    // Copy any remaining text after the last placeholder.
    result.push_str(rest);

    Ok(result)
}
