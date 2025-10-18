Update Planner Parser
=====================

Small library that turns parsed documents (cards) into a sync plan
for downstream application of operations (e.g., to Anki via wrappers).

Features
- Fresh sync planner that adds all parsed cards to a target deck
- Minimal data types that decouple parsing from planning
- `#![deny(missing_docs)]` to enforce documentation discipline

Example
```rust
use update_planner_parser::{SimplePlanner, UpdatePlanner, DocumentSet, Operation};

let docs = DocumentSet {
    cards: vec![],
    source_files: vec![],
};
let planner = SimplePlanner;
// planner.plan_fresh_sync(&docs, "MyDeck")?; // returns an error on empty docs

```

Status
- Incremental planning is not implemented yet. The trait provides a placeholder.

