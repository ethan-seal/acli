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
=> check Wi-Fi is enabled
=> restart device
=> forget network and reconnect
```

Lines prefixed with `=>` form a sequence. Generates a "First?" card for the first item and an "After X?" card for each subsequent item.

## 4. Attribute cards

```
# Hydrogen
- symbol -> H
- atomic number -> 1
- phase -> gas
```

A heading becomes the subject. Each `key -> value` line beneath it generates a card: "Hydrogen: symbol → ?" / "H".

## 5. Pipe block tables

```
subject | conjugation <->
yo | soy
tú | eres
él/ella | es
```

A contiguous block of pipe-separated lines. The first line is the header — arrow markers on a column header define whether that column generates cards and in what direction. A blank line ends the block. Context comes from the nearest preceding heading.

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
