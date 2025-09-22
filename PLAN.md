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


Examples for the parser:

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
