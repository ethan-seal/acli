#!/usr/bin/env python3
"""
Query cards from an Anki collection and output JSON.

This script reads the SQLite database directly without starting Anki.
Used by the demo runner to generate a card table in the report.

Supports both old (pre-2.1.28) and new Anki database schemas.

Usage:
    python anki-query-cards.py --collection /path/to/collection.anki2 --deck "DeckName"

Output (JSON):
    [
        {"front": "Hello", "back": "Hola", "tags": "Spanish Vocabulary::Greetings", "cardType": "reversible"},
        ...
    ]
"""

import argparse
import json
import re
import sqlite3
import sys
from pathlib import Path


def parse_args():
    parser = argparse.ArgumentParser(description='Query cards from Anki collection')
    parser.add_argument('--collection', '-c', required=True, 
                        help='Path to collection.anki2 file or containing directory')
    parser.add_argument('--deck', '-d', required=True, help='Deck name to filter')
    return parser.parse_args()


def find_collection_db(path: str) -> Path:
    """Find the collection.anki2 file from a path (file or directory)."""
    p = Path(path)
    
    # If it's a file, use it directly
    if p.is_file() and p.name.endswith('.anki2'):
        return p
    
    # If it's a directory, look for collection.anki2 in various locations
    if p.is_dir():
        # Check direct path
        direct = p / 'collection.anki2'
        if direct.exists():
            return direct
        
        # Check User 1 profile
        profile = p / 'User 1' / 'collection.anki2'
        if profile.exists():
            return profile
    
    # Maybe the path is to the parent of collection directory
    parent = p.parent
    if parent.is_dir():
        direct = parent / 'collection.anki2'
        if direct.exists():
            return direct
    
    raise FileNotFoundError(f"Could not find collection.anki2 at or near: {path}")


def strip_html(text: str) -> str:
    """Remove HTML tags from text."""
    if not text:
        return ""
    # Remove HTML tags, but preserve arrows like <-> and ->
    # Match HTML tags: < followed by a letter/! (tag start), then anything, then >
    # This won't match <-> or -> since they don't start with a letter
    clean = re.sub(r'<[a-zA-Z!][^>]*>', '', text)
    # Decode common HTML entities
    clean = clean.replace('&nbsp;', ' ')
    clean = clean.replace('&amp;', '&')
    clean = clean.replace('&lt;', '<')
    clean = clean.replace('&gt;', '>')
    clean = clean.replace('&quot;', '"')
    clean = clean.replace('&#39;', "'")
    return clean.strip()


def extract_question_from_front(front_field: str) -> tuple[str, str]:
    """Extract the question and path from acli's front field format.
    
    acli stores cards like:
        "- Spanish Vocabulary
            - Greetings
                - Hello <-> ?"
    
    Returns (question, path) where:
        question = "Hello"
        path = "Spanish Vocabulary::Greetings"
    """
    # Decode HTML entities first
    front = strip_html(front_field)
    
    # Split into lines
    lines = front.strip().split('\n')
    
    # The last line contains the question
    last_line = lines[-1].strip() if lines else ""
    
    # Extract hierarchy path from earlier lines
    path_parts = []
    for line in lines[:-1]:
        # Remove leading "- " and whitespace
        part = re.sub(r'^\s*-\s*', '', line).strip()
        if part:
            path_parts.append(part)
    
    # Extract question from last line
    # Pattern: "- Question <-> ?" or "- Question -> ?"
    last_line = re.sub(r'^\s*-\s*', '', last_line)  # Remove leading "- "
    
    # Try reversible pattern first: "Question <-> ?" or "Question  ?" (already processed)
    match = re.match(r'^(.+?)\s*(?:<->|<-&gt;)\s*\??\s*$', last_line)
    if match:
        question = match.group(1).strip()
    else:
        # Try one-way pattern: "Question -> ?" or "Question  ?"
        match = re.match(r'^(.+?)\s*(?:->|-&gt;)\s*\??\s*$', last_line)
        if match:
            question = match.group(1).strip()
        else:
            # Try pattern where arrow is already stripped, leaving "Question  ?"
            match = re.match(r'^(.+?)\s+\?\s*$', last_line)
            if match:
                question = match.group(1).strip()
            else:
                # Fallback: just use the line, removing trailing ?
                question = re.sub(r'\s*\?\s*$', '', last_line).strip()
    
    path = '::'.join(path_parts)
    return question, path


def table_exists(cursor, table_name: str) -> bool:
    """Check if a table exists in the database."""
    cursor.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
        (table_name,)
    )
    return cursor.fetchone() is not None


def get_decks_new_schema(cursor) -> dict[int, str]:
    """Get deck ID to name mapping from new schema (decks table).
    
    Returns names in internal format (with \x1f separators).
    """
    decks = {}
    cursor.execute("SELECT id, name FROM decks")
    for row in cursor.fetchall():
        # Store in internal format, we'll handle conversion in matching
        decks[row[0]] = row[1]
    return decks


def get_decks_old_schema(cursor) -> dict[int, str]:
    """Get deck ID to name mapping from old schema (col.decks JSON)."""
    cursor.execute("SELECT decks FROM col")
    row = cursor.fetchone()
    if not row or not row[0]:
        return {}
    
    try:
        decks_json = json.loads(row[0])
        return {int(k): v.get('name', '') for k, v in decks_json.items()}
    except (json.JSONDecodeError, AttributeError):
        return {}


def get_decks(cursor) -> dict[int, str]:
    """Get deck ID to name mapping, auto-detecting schema version."""
    if table_exists(cursor, 'decks'):
        return get_decks_new_schema(cursor)
    else:
        return get_decks_old_schema(cursor)


def get_notetypes_new_schema(cursor) -> dict[int, str]:
    """Get notetype ID to name mapping from new schema (notetypes table)."""
    notetypes = {}
    cursor.execute("SELECT id, name FROM notetypes")
    for row in cursor.fetchall():
        notetypes[row[0]] = row[1]
    return notetypes


def get_notetypes_old_schema(cursor) -> dict[int, str]:
    """Get notetype ID to name mapping from old schema (col.models JSON)."""
    cursor.execute("SELECT models FROM col")
    row = cursor.fetchone()
    if not row or not row[0]:
        return {}
    
    try:
        models_json = json.loads(row[0])
        return {int(k): v.get('name', '') for k, v in models_json.items()}
    except (json.JSONDecodeError, AttributeError):
        return {}


def get_notetypes(cursor) -> dict[int, str]:
    """Get notetype ID to name mapping, auto-detecting schema version."""
    if table_exists(cursor, 'notetypes'):
        return get_notetypes_new_schema(cursor)
    else:
        return get_notetypes_old_schema(cursor)


def normalize_deck_name(name: str) -> str:
    """Convert internal deck name format to display format.
    
    Anki stores deck hierarchy using \x1f (unit separator) internally,
    but displays it as :: to users.
    """
    return name.replace('\x1f', '::')


def internal_deck_name(name: str) -> str:
    """Convert display deck name to internal format.
    
    Anki stores deck hierarchy using \x1f (unit separator) internally.
    """
    return name.replace('::', '\x1f')


def get_deck_ids_with_children(decks: dict[int, str], deck_name: str) -> list[int]:
    """Get deck ID and all child deck IDs for a parent deck.
    
    Handles both internal (\x1f) and display (::) deck name formats.
    """
    matching_ids = []
    
    # Convert search name to internal format for comparison
    internal_name = internal_deck_name(deck_name)
    
    for deck_id, name in decks.items():
        # Normalize the stored name for comparison
        # Match exact name or child decks
        if name == internal_name or name.startswith(internal_name + '\x1f'):
            matching_ids.append(deck_id)
        # Also try with :: format in case the DB uses that
        elif name == deck_name or name.startswith(deck_name + '::'):
            matching_ids.append(deck_id)
    
    return matching_ids


def query_cards(collection_path: Path, deck_name: str) -> list[dict]:
    """Query all cards from the specified deck."""
    # Check file exists and show size
    if not collection_path.exists():
        print(f"ERROR: Collection file does not exist: {collection_path}", file=sys.stderr)
        return []
    
    file_size = collection_path.stat().st_size
    print(f"Collection file size: {file_size} bytes", file=sys.stderr)
    
    # Connect with immutable mode to avoid WAL issues
    # This works even if Anki has the file open
    uri = f"file:{collection_path}?immutable=1"
    try:
        conn = sqlite3.connect(uri, uri=True)
    except sqlite3.OperationalError as e:
        print(f"Failed to open with immutable mode: {e}", file=sys.stderr)
        # Fall back to regular connection
        conn = sqlite3.connect(str(collection_path))
    
    # Register unicase collation (Anki's case-insensitive collation)
    # We use a simple case-folding implementation
    def unicase_collation(a: str, b: str) -> int:
        a_lower = a.casefold()
        b_lower = b.casefold()
        if a_lower < b_lower:
            return -1
        elif a_lower > b_lower:
            return 1
        return 0
    
    conn.create_collation("unicase", unicase_collation)
    
    cursor = conn.cursor()
    
    # Debug: List all tables
    cursor.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
    tables = [row[0] for row in cursor.fetchall()]
    print(f"Tables in database: {tables}", file=sys.stderr)
    
    # Debug: Count rows in key tables
    for table in ['decks', 'cards', 'notes', 'notetypes']:
        if table in tables:
            cursor.execute(f"SELECT COUNT(*) FROM {table}")
            count = cursor.fetchone()[0]
            print(f"  {table}: {count} rows", file=sys.stderr)
    
    try:
        # Get deck and notetype mappings
        decks = get_decks(cursor)
        notetypes = get_notetypes(cursor)
        
        print(f"Found {len(decks)} decks, {len(notetypes)} notetypes", file=sys.stderr)
        # Debug: show all deck names (normalized to display format)
        deck_names = [normalize_deck_name(n) for n in decks.values()]
        print(f"Deck names: {deck_names}", file=sys.stderr)
        
        # Get all deck IDs (parent + children)
        deck_ids = get_deck_ids_with_children(decks, deck_name)
        
        if not deck_ids:
            print(f"Warning: Deck '{deck_name}' not found", file=sys.stderr)
            print(f"Available decks: {list(decks.values())}", file=sys.stderr)
            return []
        
        print(f"Matching deck IDs: {deck_ids}", file=sys.stderr)
        
        # Build query for cards in these decks
        placeholders = ','.join('?' * len(deck_ids))
        
        # Query cards with their note data
        # Cards table has: id, nid (note id), did (deck id), ord (card ordinal)
        # Notes table has: id, mid (model id), flds (fields), tags
        query = f"""
            SELECT 
                c.id,
                c.ord,
                n.flds,
                n.tags,
                n.mid,
                c.did
            FROM cards c
            JOIN notes n ON c.nid = n.id
            WHERE c.did IN ({placeholders})
            ORDER BY n.id, c.ord
        """
        
        cursor.execute(query, deck_ids)
        rows = cursor.fetchall()
        
        print(f"Found {len(rows)} cards", file=sys.stderr)
        
        cards = []
        seen_notes = set()  # Track notes to avoid duplicates from reversed cards
        
        for card_id, ord_num, fields_str, tags_str, model_id, did in rows:
            # Parse fields (separated by \x1f)
            fields = fields_str.split('\x1f')
            
            # Get raw front and back fields (keep HTML)
            raw_front = fields[0] if len(fields) > 0 else ""
            raw_back = fields[1] if len(fields) > 1 else ""
            
            # For extracting question and path, we still need the text version
            front_text = strip_html(raw_front)
            
            # Extract question and path from acli's front field format
            question, context_path = extract_question_from_front(raw_front)
            
            # Skip duplicate cards from reversed card notes (only show once)
            note_key = (question, raw_back)
            if note_key in seen_notes:
                continue
            seen_notes.add(note_key)
            
            # Get model name to determine card type
            model_name = notetypes.get(model_id, 'Unknown')
            
            # Determine card type based on model name or structure
            # acli typically creates "Basic" for one-way and "Basic (and reversed card)" for reversible
            card_type = "one-way"
            if 'reversed' in model_name.lower():
                card_type = "reversible"
            
            # Use context path from card front, or fall back to deck path
            path = context_path or normalize_deck_name(decks.get(did, ''))
            
            # Parse tags
            tags = tags_str.strip() if tags_str else ""
            
            cards.append({
                "front": raw_front,
                "back": raw_back,
                "cardType": card_type,
                "deckPath": path,
                "tags": tags,
                "cardId": card_id,
                "ordinal": ord_num,
                "question": question  # Extracted question for comparison
            })
        
        return cards
        
    finally:
        conn.close()


def main():
    args = parse_args()
    
    try:
        collection_path = find_collection_db(args.collection)
        print(f"Using collection: {collection_path}", file=sys.stderr)
        
        cards = query_cards(collection_path, args.deck)
        
        # Output JSON to stdout
        print(json.dumps(cards, indent=2))
        
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
