# Writing Cards

acli turns Markdown files into Anki flashcards. You write cards using a small set of arrow-based syntax rules, and the tool handles creating, updating, and deleting cards in your collection.

## Basic cards

Use `->` to create a one-direction card. The left side is the question, the right side is the answer.

```
- hello -> world
```

Front: `hello -> ?` | Back: `world`

## Bidirectional cards

Use `<->` to create a card that works in both directions. This maps to Anki's "Basic (and reversed card)" note type.

```
- hello <-> hola
```

Card 1 - Front: `hello <-> ?` | Back: `hola`
Card 2 - Front: `hola <-> ?` | Back: `hello`

## Nested context

Bullet nesting gives cards context. Parent bullets that contain sub-lists become context lines on the question side, not cards themselves.

```
- Spanish vocabulary
    - Greetings
        - hello <-> hola
```

Front:
```
- Spanish vocabulary
    - Greetings
        - hello <-> ?
```
Back: `hola`

Only the innermost bullet with an arrow produces a card. The parents are context.

## Attribute cards

A heading followed by `->` bullets creates attribute cards. The heading becomes the subject, shown on a separate first line.

```
# Hydrogen
- symbol -> H
- atomic number -> 1
- phase -> gas
```

| Front | Back |
|-------|------|
| Hydrogen<br>symbol ->? | H |
| Hydrogen<br>atomic number ->? | 1 |
| Hydrogen<br>phase ->? | gas |

Without a heading above the bullets, they are treated as plain basic cards.

## Ordered sequences

Lines prefixed with `=>` define an ordered sequence. A plain-text label line must appear immediately above the block.

```
Troubleshoot Wi-Fi connection
=> check Wi-Fi is enabled
=> restart device
=> forget network and reconnect
=> restart router
```

| Front | Back |
|-------|------|
| Troubleshoot Wi-Fi connection<br>First: | check Wi-Fi is enabled |
| Troubleshoot Wi-Fi connection<br>After: check Wi-Fi is enabled | restart device |
| Troubleshoot Wi-Fi connection<br>After: restart device | forget network and reconnect |
| Troubleshoot Wi-Fi connection<br>After: forget network and reconnect | restart router |

The label line is required. Without it the `=>` lines are ignored.

## Block cards

When a question or answer needs multiple lines, put `->` on its own line. Content above is the question, content below is the answer. Both sides support full Markdown.

```
- 30 ml Cognac
- 30 ml Crème de Cacao (Brown)
- 30 ml Fresh Cream
- Fresh ground nutmeg
->
Alexander
![](cocktail-images/alexander.jpg)
```

The question side renders as a bullet list, and the answer side as text + image.

Separate multiple block cards with blank lines:

```
- 30 ml Cognac
- 30 ml Crème de Cacao (Brown)
- 30 ml Fresh Cream
->
Alexander

- 30 ml Campari
- 30 ml Sweet Vermouth
- splash Soda Water
->
Americano
```

Use `<->` on its own line for bidirectional block cards:

```
hello
bonjour
<->
hi
salut
```

**Note:** Block cards are standalone — headings above them do **not** provide context the way they do for inline cards. If you want context, include it in the question content.

An arrow on its own line that is missing a question or answer side is skipped with a warning.

## Pipe-block tables

A contiguous block of pipe-separated lines forms a table. The first line is the header row, and arrow markers on column headers control which columns generate cards.

```
subject | conjugation <->
yo | soy
tu | eres
el/ella | es
```

Arrow markers in the header:

| Marker | Meaning |
|--------|---------|
| `col <->` | Bidirectional - generates a forward and a reverse card per row |
| `col ->` | Forward only - row value to cell value |
| `col <-` | Backward only - cell value to row value |
| *(no arrow)* | Display only, no cards generated |

Each data row produces cards for every column that has an arrow marker. The first column is always the "subject" column.

**Multi-column example:**

```
element | symbol <-> | atomic # ->
hydrogen | H | 1
helium | He | 2
```

This generates:
- `hydrogen` <-> `H` (both directions)
- `hydrogen` -> `1` (forward only)
- `helium` <-> `He` (both directions)
- `helium` -> `2` (forward only)

**Reference table (no cards):**

If no column header has an arrow marker, no cards are generated.

```
term | definition | example
affect | verb, to influence | weather affects mood
effect | noun, a result | the effect was immediate
```

Empty trailing cells are allowed and simply produce no card for that cell:

```
case | singular | plural
nominative | puella | puellae
accusative | puellam |
```

Here, accusative/plural is empty so no card is generated for that cell.

## Templates

Templates let you generate many cards from a pattern and a data table. A code fence with language `template` defines the card shape using `{{ column }}` placeholders. A pipe table immediately after supplies the data. Each row is expanded through the template, then parsed for card syntax like any other text.

**Inline card template:**

~~~
```template
- {{ english }} <-> {{ spanish }}
```
english | spanish
hello | hola
goodbye | adiós
good morning | buenos días
~~~

This expands to three bidirectional cards, identical to writing them by hand.

**Block card template:**

~~~
```template
{{ ingredients }}
->
{{ name }}
![](cocktail-images/{{ image }}.jpg)
```
name | image | ingredients
Alexander | alexander | 30 ml Cognac, 30 ml Crème de Cacao, 30 ml Fresh Cream
Americano | americano | 30 ml Campari, 30 ml Sweet Vermouth, splash Soda Water
Angel Face | angel_face | 30 ml Gin, 30 ml Apricot Brandy, 30 ml Calvados
~~~

Each row expands into a block card whose question is the ingredient list and whose answer is the name + image.

**Rules:**

- Placeholders use `{{ column }}` syntax (whitespace around the name is flexible).
- Substitution only — no conditionals, loops, or filters.
- Every placeholder must match a column header. Unknown placeholders are an error.
- Every cell referenced by the template must have a value. Empty cells are an error.
- The pipe table must start on the line immediately after the closing fence — no blank lines between them. A blank line (or no table) produces a warning.
- The expanded text is parsed for all card syntax (`->`, `<->`, block cards, etc.), so templates compose freely with every other card format.
- Prefer hyphens over underscores in column names (e.g. `{{ first-name }}` not `{{ first_name }}`). Any characters work except `|` and `}}`.

## Images

Standard Markdown image syntax works on either side of an arrow. Images become `<img>` tags in the card fields, and the image file is tracked for syncing to Anki's media folder.

```
- ![](cat.jpg) -> Cat
- question -> ![A parrot](birds/parrot.png)
```

## Inline code

Arrows inside backtick code spans are **not** treated as delimiters. This lets you write cards about code that contains `->` or `<->` without false splits.

```
- `->` is the Rust return type arrow -> used in function signatures
```

Only the `->` outside the backticks is treated as a card delimiter.

## Tips

- **Keep answers short.** One concept per card. Split complex facts into multiple cards.
- **Avoid sets.** Instead of "list all X", make one card per item with distinguishing context.
- **Use nesting for context.** Parent bullets put your brain in the right mode before you see the question.
- **Use `<->` for vocabulary.** Bidirectional cards test recall in both directions.
- **Use sequences for procedures.** The "After X?" format tests each step individually.
