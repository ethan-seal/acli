# High level plan

- Library for creating/updating/deleting individual cards/notes (anki-wrapper)
- Parser for card documents (doc-parser)
- Simple diff generator (update-planner):
    - generate either add, delete, or updates
- CLI for syncing (acli)
    - Goes through all the documents in the directories and subdirectories, use the parser, then run the merger, and do the creates/updates/deletes.
    - This is the main application and is defined at the high level of the repo
- LSP server (wait to do) (acli-lsp)
    - wait until a mvp with the other parts are done to even think about this
    - Show errors when the parser is partially broken
    - syntax highlighting

# Architecture Decisions

## Dependencies Philosophy
- Prefer minimal libraries over feature-rich ones
- Consider no-std compatibility where possible to keep crates lightweight
- Avoid unnecessary dependencies to reduce compilation time and binary size

## Card Identification & Syncing
- One-way sync: documents → Anki (documents are source of truth)
- Card identification system will be implemented later (table for MVP)
- For now, cards will be created fresh on each sync

## Anki Integration
- Use Anki's Rust APIs directly via git subrepo
- Target default Anki collection initially
- Deck assignment: hardcoded deck name passed to anki-wrapper with each card

## Document Organization
- Valid markdown files organized by deck/subject
- Multiple documents and nested directories supported per subject
- Multiple cards can be defined within each document
- Directory structure does not automatically map to Anki deck structure

## Error Handling Strategy
- Parser failures: fail entire document (don't skip malformed cards)
- Prefer explicit failures over silent data corruption


# Starting tasks:
- Create a subdirectory that is a rust library (call it anki-wrapper)
    - This will implement a simple library around anki that makes adding/updating/deleting cards easy
    - Initialize a git subrepo in that subdirectory for anki (https://github.com/ankitects/anki)
- Commit once that is created

- Do an investigation of the anki subcrate and look for the types necessary
    - I expect we'll use a type called Card or something which is a list of strings (or maybe a list of structs, which mostly just contains a string)
    - Write down your notes on it and save it in a markdown document
- Create a individual plan for part of the above (skipping the lsp server) in a separate markdown document
    - Include the examples from below and expand them
        - For the cli and diff generation, create coherent examples
    - Think about which dependencies to use and ways to reduce the need for them
    - Include a testing strategy
        - Prefer fakes over stubbing
    - For the library anki-wrapper, include the doc your planned trait and types
- Then ask for my review of each planned part


# Parser Behavior

## Card Type Mapping
- `<->` syntax creates bidirectional cards (maps to Anki's "Basic (and reversed card)")
- `->` syntax creates basic cards (maps to Anki's "Basic" note type)
- All parent context (background material) is preserved in question field

## Processing Steps
1. Parse markdown to AST
2. Second pass: extract card syntax from AST
3. Generate Card structs with preserved context

## Examples for the parser:

---

Input string:
One <-> 1
Output:
Card {
  type: CardType::Bidirectional,
  fields: [
      "One <-> ?",
      "1"
  ]
}

---

Input string:
- Background material
    - hello -> world
Output:
Card {
  type: CardType::Basic,
  fields: [
      "- Background material\n    - hello -> ?",
      "world"
  ]
}

---

Input string:
- Extremely detailed background material
    - alternate Background material
        - determinants <- suck
Output:
Card {
  type: CardType::Basic,
  fields: [
      "- Extremely detailed background material\n    - alternate Background material\n        - ? <- suck",
      "determinants"
  ]
}

---

Examples

Bring everything from above:
