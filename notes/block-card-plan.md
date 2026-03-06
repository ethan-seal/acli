# Block Cards & Templates — Design Plan

## Problem

Inline cards (`- question -> answer`) are single-line. Multiline content
requires hacks like `<br>` tags or cramming everything onto one line:

```
- 30 ml Cognac, 30 ml Crème de Cacao (Brown), 30 ml Fresh Cream -> Alexander<br>![](cocktail-images/alexander.jpg)
```

Users need multiline questions (ingredient lists, code snippets, context
paragraphs) and multiline answers (text + images, bulleted lists, multi-line
explanations).

## Design

A `->` on its own line acts as a block separator. Content above is the
question, content below is the answer. Both sides support full markdown.

```markdown
- 30 ml Cognac
- 30 ml Crème de Cacao (Brown)
- 30 ml Fresh Cream
- Fresh ground nutmeg
->
Alexander
![](cocktail-images/alexander.jpg)
```

Blank lines separate one block card from the next:

```markdown
- 30 ml Cognac
- 30 ml Crème de Cacao (Brown)
- 30 ml Fresh Cream
->
Alexander
![](cocktail-images/alexander.jpg)

- 30 ml Campari
- 30 ml Sweet Vermouth
- splash Soda Water
->
Americano
![](cocktail-images/americano.jpg)
```

## Rules

1. `->` with text on both sides = **inline card** (unchanged, existing behavior)
2. `->` on its own line = **block card separator**
3. Question = contiguous content above `->`, back to the previous blank line
   (or start of file)
4. Answer = contiguous content below `->`, until the next blank line
   (or end of file)
5. Both sides support full markdown: bullet lists, images, code blocks, etc.
6. `<->` on its own line = **bidirectional block card**

## Examples

### Bullet list as question, text + image as answer

```markdown
- 30 ml Cognac
- 30 ml Crème de Cacao (Brown)
- 30 ml Fresh Cream
- Fresh ground nutmeg
->
Alexander
![](cocktail-images/alexander.jpg)
```

### Simple question, list as answer

```markdown
What are the three states of matter?
->
- Solid
- Liquid
- Gas
```

### Multiline question only (answer stays on arrow line)

Not supported — if you want a single-line answer with a multiline question,
put the answer on the line after `->`:

```markdown
- 30 ml Cognac
- 30 ml Crème de Cacao (Brown)
- 30 ml Fresh Cream
->
Alexander
```

### Bidirectional block card

```markdown
hello
bonjour
hola
<->
hi
salut
hola (informal)
```

### Mixed with inline cards in the same file

```markdown
# Spanish Vocabulary

- hello <-> hola
- goodbye <-> adiós

# IBA Cocktails

- 30 ml Cognac
- 30 ml Crème de Cacao (Brown)
- 30 ml Fresh Cream
->
Alexander
![](cocktail-images/alexander.jpg)

- 30 ml Campari
- 30 ml Sweet Vermouth
- splash Soda Water
->
Americano
![](cocktail-images/americano.jpg)
```

## Implementation

Block cards are processed as a **pre-processor stage**, like pipe tables and
ordered sequences. The pre-processor:

1. Scans raw lines for `->` or `<->` on its own line (trimmed)
2. Looks backward to collect question content (up to blank line or file start)
3. Looks forward to collect answer content (until blank line or file end)
4. Emits a `Card` with the raw markdown for each side
5. Removes consumed lines from the text before pulldown-cmark processes it

Each side's raw markdown is rendered to HTML for Anki fields (bullet lists
become `<ul>/<li>`, images become `<img>`, etc.).

## Templates

Templates solve the problem of **repetitive structured data with multiline
content**. Block cards handle one-off multiline cards well, but for 35
cocktails that all follow the same pattern, you want the structural safety
of a table (header defines schema, each row is consistent) with the
flexibility of block cards.

A code fence with `template` language defines the card shape. A pipe table
immediately after supplies the data. Each row is expanded through the
template, producing text that is then parsed for card syntax (including
block cards).

### Cocktail example

~~~markdown
```template
{{ ingredients }}
->
{{ name }}
![](cocktail-images/{{ image }}.jpg)
```

name | image | ingredients
Alexander | alexander | 30 ml Cognac, 30 ml Crème de Cacao (Brown), 30 ml Fresh Cream, Fresh ground nutmeg
Americano | americano | 30 ml Campari, 30 ml Sweet Vermouth, splash Soda Water, Orange slice and lemon zest
Angel Face | angel_face | 30 ml Gin, 30 ml Apricot Brandy, 30 ml Calvados
~~~

Row 1 expands to:

```
30 ml Cognac, 30 ml Crème de Cacao (Brown), 30 ml Fresh Cream, Fresh ground nutmeg
->
Alexander
![](cocktail-images/alexander.jpg)
```

Which is parsed as a block card: question = ingredient list, answer = name +
image.

### Inline card templates

Templates compose with any card syntax. The expanded text is parsed the same
way as hand-written text:

~~~markdown
```template
- {{ english }} <-> {{ spanish }}
```

english | spanish
hello | hola
goodbye | adiós
good morning | buenos días
~~~

Expands to:

```markdown
- hello <-> hola
- goodbye <-> adiós
- good morning <-> buenos días
```

Which produces inline bidirectional cards, same as writing them by hand.

### Template syntax

Simple `{{ column }}` substitution only. No conditionals, no loops, no
filters. The template is a string with placeholders that get replaced by
cell values.

| Syntax | Meaning | Example |
|--------|---------|---------|
| `{{ column }}` | Substitute column value | `{{ name }}` → Alexander |

The expanded text is then parsed for all card syntax: `->`, `<->`, block
`->`, etc.

### Why simple substitution

The earlier exploration proposed full Jinja2/Tera with conditionals, loops,
and filters. That was cut as too complex. Simple substitution covers the
core use case (repetitive structured data with a consistent shape) without
the complexity. If a row needs different structure, write it as a standalone
block card instead.

### Template processing order

Templates are expanded **first**, before any other pre-processing. The
pipeline becomes:

1. Expand templates (template code fence + pipe table → raw text)
2. Extract pipe table cards (from non-template pipe tables)
3. Extract sequence cards
4. Extract block cards
5. pulldown-cmark event walk (inline cards)

## Decisions

1. **Heading context** — No. Block cards are standalone and control their own
   content. Headings above block cards do not provide context. If you want
   context on a block card, include it in the question content.

2. **Rendering** — Full pulldown-cmark render. Each side of a block card is
   raw markdown run through pulldown-cmark to produce HTML. This correctly
   handles bullet lists, images, code blocks, emphasis, etc.

3. **Card type** — Reuse existing types. Block cards map to `CardType::Basic`,
   `<->` block cards to `CardType::Bidirectional`. No new card types needed.
   Block cards are a different authoring syntax, not a different card type.

4. **Ingredient formatting** — No filters. Templates are pure substitution.
   Comma-separated values in a cell stay comma-separated on the card. If you
   need different formatting, structure the template and data differently.

5. **Empty cells** — Error. If a template references `{{ column }}` and the
   cell is empty, that is an error. Every row must provide a value for every
   placeholder used in the template. If some rows lack a field, use a
   different template or write those cards as standalone block cards.
