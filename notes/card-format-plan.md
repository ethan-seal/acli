# Card Format — Feature Plan

Five features. Nothing else.

## 1. Basic card

```
a -> b
```

One direction. Left is the question, right is the answer.

## 2. Bidirectional card

```
a <-> b
```

Generates two cards, one in each direction.

## 3. Ordered sequence

```
Troubleshoot Wi-Fi connection
=> check Wi-Fi is enabled
=> restart device
=> forget network and reconnect
```

A plain-text label line immediately above the `=>` block provides context. It is required. Generates cards with a two-line front:

- First item → front: `<label>` / `First:` — back: `<first step>`
- Each subsequent item → front: `<label>` / `After: <previous step>` — back: `<next step>`

## 4. Attribute cards

```
# Hydrogen
- symbol -> H
- atomic number -> 1
- phase -> gas
```

A heading becomes the subject. Each `key -> value` line beneath it generates a card with a two-line front:

- Front: `<subject>` / `<key> →?` — back: `<value>`

Example: front line 1 `Hydrogen`, line 2 `symbol →?` — back `H`.

Without a preceding heading, the bullet is treated as a plain basic card, not an attribute card.

## 5. Pipe block tables

```
subject | conjugation <->
yo | soy
tú | eres
él/ella | es
```

A contiguous block of pipe-separated lines. The first line is the header — arrow markers on a column header define whether that column generates cards and in what direction. A blank line ends the block.

Context for card fronts always comes from the column headers, joined as `<subject col> → <value col>`. Cards have a two-line front with context on line 1:

- Forward: `<subject col> → <value col>` / `<row value> →?` — back: `<cell value>`
- Reverse: `<subject col> → <value col>` / `? ← <cell value>` — back: `<row value>`

Arrow markers:
- `col <->` — bidirectional (row ↔ cell)
- `col ->` — forward only (row → cell)
- `col <-` — backward only (cell → row)
- no arrow — display only, no cards

Empty trailing cells make missing values visible:

```
case | singular | plural
nominative | puella | puellae
accusative | puellam |
```

## Cut

| Feature | Reason |
|---------|--------|
| `[[text]]` cloze deletion | Redundant with `->` for most cases; adds parser complexity |
| Template metaprogramming | Vim-hostile; write cards directly instead |
| `{{c1::text}}` multi-cloze | Never committed to |
| `a <- b` reversed arrow | Write `b -> a` instead |
