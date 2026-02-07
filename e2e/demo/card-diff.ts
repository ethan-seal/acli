/**
 * Card-level diffing that matches the actual sync logic used by acli.
 * 
 * This implements the same diffing algorithm as the Rust code in
 * update-planner-parser/src/types.rs (DocumentDiff::compute).
 */

/**
 * Card type matching the Rust CardType enum.
 */
export type CardType = "Basic" | "Bidirectional";

/**
 * A card with its fields, matching the Rust Card struct.
 */
export interface Card {
  cardType: CardType;
  fields: string[];
}

/**
 * A stable ID for a card based on its content.
 * Matches the Rust CardId implementation.
 */
export type CardId = string;

/**
 * Parse markdown into cards, matching the behavior of doc-parser.
 * This extracts cards from bullet points with -> or <-> syntax.
 */
export function parseCards(markdown: string): Card[] {
  const cards: Card[] = [];
  const lines = markdown.split("\n");
  
  for (const line of lines) {
    const trimmed = line.trim();
    
    // Look for bidirectional cards: "Question <-> Answer"
    const bidirMatch = trimmed.match(/^-\s+(.+?)\s+<->\s+(.+)$/);
    if (bidirMatch) {
      cards.push({
        cardType: "Bidirectional",
        fields: [bidirMatch[1].trim(), bidirMatch[2].trim()],
      });
      continue;
    }
    
    // Look for basic cards: "Question -> Answer"
    const basicMatch = trimmed.match(/^-\s+(.+?)\s+->\s+(.+)$/);
    if (basicMatch) {
      cards.push({
        cardType: "Basic",
        fields: [basicMatch[1].trim(), basicMatch[2].trim()],
      });
    }
  }
  
  return cards;
}

/**
 * Simple hash function that produces the same hash for the same input.
 * This mimics the Rust DefaultHasher behavior for our purposes.
 */
function simpleHash(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32bit integer
  }
  // Convert to hex string, zero-padded to 16 chars like Rust's CardId Display impl
  return Math.abs(hash).toString(16).padStart(16, '0');
}

/**
 * Compute a stable CardId from a card's content.
 * Matches the Rust CardId::from_card implementation.
 */
export function computeCardId(card: Card): CardId {
  // Hash the card type and all fields
  const content = `${card.cardType}:${card.fields.join(":")}`;
  return simpleHash(content);
}

/**
 * The result of diffing two card sets.
 * Matches the Rust DocumentDiff struct.
 */
export interface CardDiff {
  added: Card[];
  deleted: Card[];
  updated: Array<{ old: Card; new: Card }>;
}

/**
 * Compute the diff between old and new card sets.
 * Matches the Rust DocumentDiff::compute implementation.
 */
export function computeCardDiff(
  oldMarkdown: string,
  newMarkdown: string
): CardDiff {
  const oldCards = parseCards(oldMarkdown);
  const newCards = parseCards(newMarkdown);
  
  // Build maps of CardId -> Card for fast lookup
  const oldMap = new Map<CardId, Card>();
  const newMap = new Map<CardId, Card>();
  
  for (const card of oldCards) {
    oldMap.set(computeCardId(card), card);
  }
  
  for (const card of newCards) {
    newMap.set(computeCardId(card), card);
  }
  
  const oldIds = new Set(oldMap.keys());
  const newIds = new Set(newMap.keys());
  
  // Cards in new but not in old = added
  const added: Card[] = [];
  for (const id of newIds) {
    if (!oldIds.has(id)) {
      added.push(newMap.get(id)!);
    }
  }
  
  // Cards in old but not in new = deleted
  const deleted: Card[] = [];
  for (const id of oldIds) {
    if (!newIds.has(id)) {
      deleted.push(oldMap.get(id)!);
    }
  }
  
  // Cards with same ID but different content = updated
  // With content-based hashing, this should be empty since any content
  // change produces a different CardId
  const updated: Array<{ old: Card; new: Card }> = [];
  for (const id of oldIds) {
    if (newIds.has(id)) {
      const oldCard = oldMap.get(id)!;
      const newCard = newMap.get(id)!;
      
      // Check if cards are actually different (shouldn't happen with content-based IDs)
      if (JSON.stringify(oldCard) !== JSON.stringify(newCard)) {
        updated.push({ old: oldCard, new: newCard });
      }
    }
  }
  
  return { added, deleted, updated };
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Format a card for display in the diff.
 */
function formatCard(card: Card): string {
  const arrow = card.cardType === "Bidirectional" ? "<->" : "->";
  return `${card.fields[0]} ${arrow} ${card.fields[1]}`;
}

/**
 * Generate HTML for an inline card-level diff.
 * This shows the sync operations that will be performed.
 */
export function getCardDiffHtml(oldMarkdown: string, newMarkdown: string): string {
  if (!oldMarkdown) {
    const cards = parseCards(newMarkdown);
    const htmlParts = ['<pre class="diff card-diff">'];
    htmlParts.push('<span class="diff-header">Initial cards (all will be added)</span>');
    
    for (const card of cards) {
      const formatted = formatCard(card);
      htmlParts.push(`<span class="diff-add">+ ${escapeHtml(formatted)}</span>`);
    }
    
    htmlParts.push("</pre>");
    return htmlParts.join("");
  }
  
  const diff = computeCardDiff(oldMarkdown, newMarkdown);
  const htmlParts = ['<pre class="diff card-diff">'];
  
  // Show the sync operations in the order they'll be performed
  htmlParts.push('<span class="diff-header">Sync operations (matching acli logic)</span>');
  
  // Show added cards
  if (diff.added.length > 0) {
    htmlParts.push('<span class="diff-header">');
    htmlParts.push(`Added (${diff.added.length} card${diff.added.length === 1 ? '' : 's'}):`);
    htmlParts.push('</span>');
    for (const card of diff.added) {
      const formatted = formatCard(card);
      const typeLabel = card.cardType === "Bidirectional" ? "[bidir]" : "[basic]";
      htmlParts.push(`<span class="diff-add">+ ${escapeHtml(formatted)} ${typeLabel}</span>`);
    }
  }
  
  // Show deleted cards
  if (diff.deleted.length > 0) {
    if (diff.added.length > 0) {
      htmlParts.push('<span class="diff-ctx"> </span>');
    }
    htmlParts.push('<span class="diff-header">');
    htmlParts.push(`Deleted (${diff.deleted.length} card${diff.deleted.length === 1 ? '' : 's'}):`);
    htmlParts.push('</span>');
    for (const card of diff.deleted) {
      const formatted = formatCard(card);
      const typeLabel = card.cardType === "Bidirectional" ? "[bidir]" : "[basic]";
      htmlParts.push(`<span class="diff-del">- ${escapeHtml(formatted)} ${typeLabel}</span>`);
    }
  }
  
  // Show updated cards (should be empty with content-based IDs)
  if (diff.updated.length > 0) {
    if (diff.added.length > 0 || diff.deleted.length > 0) {
      htmlParts.push('<span class="diff-ctx"> </span>');
    }
    htmlParts.push('<span class="diff-header">');
    htmlParts.push(`Updated (${diff.updated.length} card${diff.updated.length === 1 ? '' : 's'}):`);
    htmlParts.push('</span>');
    for (const { old, new: newCard } of diff.updated) {
      const oldFormatted = formatCard(old);
      const newFormatted = formatCard(newCard);
      htmlParts.push(`<span class="diff-del">- ${escapeHtml(oldFormatted)}</span>`);
      htmlParts.push(`<span class="diff-add">+ ${escapeHtml(newFormatted)}</span>`);
    }
  }
  
  // Show unchanged cards
  const oldCards = parseCards(oldMarkdown);
  const newCards = parseCards(newMarkdown);
  const oldIds = new Set(oldCards.map(computeCardId));
  const newIds = new Set(newCards.map(computeCardId));
  const unchangedIds = [...oldIds].filter(id => newIds.has(id));
  
  if (unchangedIds.length > 0) {
    if (diff.added.length > 0 || diff.deleted.length > 0 || diff.updated.length > 0) {
      htmlParts.push('<span class="diff-ctx"> </span>');
    }
    htmlParts.push('<span class="diff-header">');
    htmlParts.push(`Unchanged (${unchangedIds.length} card${unchangedIds.length === 1 ? '' : 's'}):`);
    htmlParts.push('</span>');
    
    // Show a few examples
    const samplesToShow = Math.min(3, unchangedIds.length);
    for (let i = 0; i < samplesToShow; i++) {
      const id = unchangedIds[i];
      const card = newCards.find(c => computeCardId(c) === id);
      if (card) {
        const formatted = formatCard(card);
        htmlParts.push(`<span class="diff-ctx">  ${escapeHtml(formatted)}</span>`);
      }
    }
    
    if (unchangedIds.length > samplesToShow) {
      htmlParts.push(`<span class="diff-ctx">  ... and ${unchangedIds.length - samplesToShow} more</span>`);
    }
  }
  
  // Summary
  if (diff.added.length === 0 && diff.deleted.length === 0 && diff.updated.length === 0) {
    htmlParts.push('<span class="diff-ctx"> </span>');
    htmlParts.push('<span class="diff-header">No changes</span>');
  }
  
  htmlParts.push("</pre>");
  return htmlParts.join("");
}
