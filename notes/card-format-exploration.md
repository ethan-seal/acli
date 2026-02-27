# Flashcard Format Exploration

## Current Syntax

- `front -> back` — Basic card (question → answer)
- `front <-> back` — Bidirectional card (both directions)
- `back <- front` — Reversed basic card
- Nested bullet points become context for the question

---

## Part 1: Use Case Categories

### Academic

1. **Languages** — vocabulary, conjugations, grammar rules, idioms
2. **Math** — formulas, theorems, proofs, symbol meanings
3. **Science** — terminology, processes, equations, constants
4. **History** — dates, figures, events, causes/effects
5. **Music** — notation, theory, composers, pieces
6. **Art** — artists, movements, techniques, famous works
7. **Medicine** — anatomy, pharmacology, diagnoses, procedures
8. **Law** — cases, statutes, definitions, elements of crimes
9. **Philosophy** — thinkers, arguments, fallacies, terms
10. **Computer Science** — algorithms, complexity, syntax, patterns

### Practical/Professional

11. **Troubleshooting** — error codes, diagnostic steps, fixes
12. **Procedures** — checklists, workflows, safety protocols
13. **Standards/Codes** — building codes, regulations, specs
14. **Keyboard shortcuts** — application-specific hotkeys
15. **CLI commands** — flags, options, common invocations
16. **API reference** — endpoints, parameters, return values

### Personal

17. **People** — names, faces, relationships, birthdays
18. **Places** — addresses, directions, landmarks
19. **Passwords/PINs** — (mnemonic hints only!)
20. **Household** — cleaning products, maintenance schedules
21. **Health** — medications, dosages, allergies
22. **Recipes** — ratios, techniques, substitutions
23. **Quotes** — attribution, source, context

---

## Part 2: Concrete Examples by Category

### Languages

```
Spanish vocabulary:
- hello <-> hola
- goodbye <-> adios
- thank you <-> gracias

Verb conjugation (ser, present):
- yo -> soy
- tu -> eres
- el/ella -> es
- nosotros -> somos
- ellos -> son

Irregular past tense:
- go -> went
- be -> was/were
- have -> had
```

### Math

```
Quadratic formula:
- x = -> (-b +/- sqrt(b^2 - 4ac)) / 2a

Trig identities:
- sin^2(x) + cos^2(x) = -> 1
- sin(2x) = -> 2sin(x)cos(x)

Derivatives:
- d/dx[x^n] -> nx^(n-1)
- d/dx[e^x] -> e^x
- d/dx[ln(x)] -> 1/x
```

### Science

```
Periodic table:
- H -> hydrogen, 1
- He -> helium, 2
- Li -> lithium, 3

SI units:
- force -> newton (N)
- energy -> joule (J)
- power -> watt (W)

Physics constants:
- speed of light -> 3 x 10^8 m/s
- gravitational constant -> 6.67 x 10^-11 N m^2/kg^2
```

### History

```
US Presidents (chronological):
- 1st -> Washington
- 16th -> Lincoln
- 32nd -> FDR

World War II:
- started -> 1939
- ended -> 1945
- D-Day -> June 6, 1944

Roman emperors:
- first -> Augustus
- built the wall -> Hadrian
- split the empire -> Diocletian
```

### Troubleshooting

```
HTTP status codes:
- 200 -> OK
- 404 -> Not Found
- 500 -> Internal Server Error
- 401 -> Unauthorized
- 403 -> Forbidden

Exit codes (Unix):
- 0 -> success
- 1 -> general error
- 126 -> permission problem
- 127 -> command not found
```

### Household

```
Stain removal:
- red wine -> salt, then cold water
- grease -> dish soap
- blood -> cold water, never hot
- ink -> rubbing alcohol

Cleaning products:
- granite -> pH neutral cleaner
- stainless steel -> vinegar or specific cleaner
- wood floors -> damp mop only
```

### Personal

```
Family birthdays:
- Mom -> March 15
- Dad -> July 22
- Sister -> November 3

Contacts at work:
- IT help desk -> ext 4357
- HR -> ext 2100
- Facilities -> ext 3200
```

---

## Part 3: Format Ideas for Dense Encoding

### 3.1 Current Syntax Limitations

The current `->` and `<->` syntax works well for:
- Simple term ↔ definition pairs
- Single question/answer
- Context provided by nesting

But struggles with:
- Multiple related items (conjugation tables)
- Ordered sequences (steps, chronologies)
- Multi-field cards (term, definition, example, etymology)
- Cloze deletions (fill-in-the-blank in context)
- Image/audio associations
- Conditional relationships (if X then Y)

---

### 3.2 Table Format

For conjugations, declensions, or any matrix of related data. A contiguous block of pipe-separated lines forms a table — the first line is the header (where arrow markers live), a blank line ends it. No alignment required.

```
ser (present):
subject | form <->
yo | soy
tú | eres
el/ella | es
nosotros | somos
ellos | son
```

Generates cards:
- "ser (present): yo → ?" / "soy"
- "ser (present): tú → ?" / "eres"
- etc.

**More complex table (2D — missing values are visible):**
```
puella (girl):
case | singular | plural
nominative | puella | puellae
genitive | puellae | puellarum
dative | puellae | puellis
accusative | puellam |
ablative | puella | puellis
```

The empty trailing cell makes gaps obvious — you can see accusative plural hasn't been filled in yet.

Could generate cards with context:
- "puella: nominative singular → ?" / "puella"
- "puella: genitive plural → ?" / "puellarum"

---

### 3.3 Sequence/List Format

For ordered steps, chronologies, or rankings:

```
# Troubleshooting: Computer won't boot
1. Check power cable connected
2. Check outlet has power
3. Listen for fans/beeps
4. Try different power cable
5. Open case, check internal connections
```

Could generate:
- "Computer won't boot: step 1 → ?" / "Check power cable connected"
- "Computer won't boot: after 'Check power cable' → ?" / "Check outlet has power"

**Or a chronology:**
```
# Roman emperors (chronological)
1. Augustus (27 BC - 14 AD)
2. Tiberius (14 - 37)
3. Caligula (37 - 41)
4. Claudius (41 - 54)
5. Nero (54 - 68)
```

---

### 3.4 Cloze Format

Fill-in-the-blank within context:

```
The {{mitochondria}} is the powerhouse of the cell.

In 1969, {{Neil Armstrong}} became the first person on the moon.

E = {{mc^2}}
```

Generates cloze cards where the bracketed text is hidden.

**Multiple clozes in one sentence:**
```
{{c1::Columbus}} sailed in {{c2::1492}} with {{c3::three}} ships.
```

---

### 3.5 Multi-Field Format

For cards needing more than front/back:

```
[term] photosynthesis
[definition] process by which plants convert sunlight to energy
[example] leaves turning green in spring
[etymology] Greek: photo (light) + synthesis (putting together)
```

Or more compact:
```
photosynthesis | plants convert sunlight to energy | Greek photo+synthesis
```

---

### 3.6 Hierarchical Definitions

For terms with multiple meanings or aspects:

```
- bank
  - financial institution -> where you deposit money
  - river edge -> the land alongside a river
  - to tilt -> the plane banked left
```

Each sub-bullet becomes a card with "bank" context.

---

### 3.7 Q&A Blocks

For more complex questions:

```
Q: What are the three branches of US government?
A: Legislative (Congress), Executive (President), Judicial (Supreme Court)

Q: Why did Rome fall?
A: Multiple factors: economic troubles, military overspending, 
   political instability, barbarian invasions
```

---

### 3.8 Attribute Lists

For items with multiple properties:

```
# Hydrogen
- symbol: H
- atomic number: 1
- atomic weight: 1.008
- state at room temp: gas
- discovered by: Cavendish, 1766

# Helium  
- symbol: He
- atomic number: 2
- atomic weight: 4.003
- state at room temp: gas
- discovered by: Janssen & Lockyer, 1868
```

Each `key: value` under a heading generates a card.

---

### 3.9 Comparison Format

For contrasting similar items:

```
term | definition | example
affect | verb, to influence | "The weather affects my mood"
effect | noun, a result | "The effect was immediate"
```

No arrows in the header = no cards generated (reference only). Add arrow markers to generate cards testing the difference.

---

### 3.10 Conditional/Procedural Format

For if/then or decision trees:

```
# Stain type decision tree
- stain is protein-based (blood, egg, milk)?
  - use: cold water
  - avoid: hot water (sets protein)
- stain is oil-based (grease, butter)?
  - use: dish soap, then launder
- stain is tannin (wine, coffee, tea)?
  - use: vinegar or enzyme cleaner
```

---

### 3.11 Shorthand/Symbol Definitions

Compact syntax using special markers:

```
:: vocabulary
en:hello = es:hola = fr:bonjour = de:hallo
en:goodbye = es:adios = fr:au revoir = de:auf wiedersehen
```

Each `=` creates bidirectional pairs between all languages.

---

### 3.12 Reference Cards (Read-only context)

Sometimes you want background that isn't itself a card:

```
> The French Revolution lasted from 1789 to 1799.

- French Revolution
  - started -> 1789
  - ended -> 1799
  - caused by -> financial crisis, social inequality
```

Lines starting with `>` are context but not questions.

---

## Part 4: Syntax Proposals

### Minimal Extensions to Current Syntax

| Syntax | Meaning |
|--------|---------|
| `a -> b` | basic card |
| `a <-> b` | bidirectional |
| `[[x]]` | cloze deletion (x is hidden) |
| `=> step` | ordered sequence (line prefix) |
| `# heading` + `key -> value` | attribute cards |
| table with arrows in headers | table cards (see below) |

#### Sequence syntax

Steps are prefixed with `=>`. Order is inferred from line order:

```
=> check Wi-Fi is enabled
=> restart device
=> forget network and reconnect
=> restart router
```

Generates "After X?" cards automatically from the sequence.

#### Table syntax

A contiguous block of pipe-separated lines forms a table. The first line is the header, where arrow markers define which columns generate cards. A blank line or non-pipe line ends the block. The nearest preceding heading or `label:` line provides context. No alignment required.

```
## Elements

element | symbol <-> | atomic # -> | phase
hydrogen | H | 1 | gas
helium | He | 2 | gas
lithium | Li | 3 | solid
```

Header markers:
- `column <->` = bidirectional (row ↔ cell)
- `column ->` = forward only (row → cell)
- `column <-` = backward only (cell → row)
- No arrow = no cards (display only, can be shown as context)

The first column is always the "concept" (what's being described). Other columns are "descriptors" (properties of the concept).

**Missing values are explicit:**

```
puella (girl):
case | singular | plural
nominative | puella | puellae
genitive | puellae | puellarum
accusative | puellam |
```

The empty trailing cell shows accusative plural hasn't been filled in yet — visible at a glance.

**Reference tables (no cards):**

```
term | definition | example
affect | verb, to influence | weather affects mood
effect | noun, a result | the effect was immediate
```

No arrows = no cards generated.

**Template block (metaprogramming)**

A code fence with `template` language, followed by a pipe block. Each row is expanded through the template using **Jinja2/Tera syntax**, then parsed for card syntax.

~~~markdown
```template
{{ name }} ([[{{ born }}]]-[[{{ died }}]]) 
revolutionized {{ field }}.
```

name | born | died | field
einstein | 1879 | 1955 | physics
darwin | 1809 | 1882 | biology
~~~

Expands to:
```
Einstein ([[1879]]-[[1955]]) revolutionized physics.
Darwin ([[1809]]-[[1882]]) revolutionized biology.
```

Which generates cloze cards:
- "Einstein ([...]-[...]) revolutionized physics." → 1879, 1955
- "Darwin ([...]-[...]) revolutionized biology." → 1809, 1882

**Template syntax (Jinja2/Tera):**

| Syntax | Meaning | Example |
|--------|---------|---------|
| `{{ column }}` | Substitute column value | `{{ name }}` → Einstein |
| `{% if x %}...{% endif %}` | Conditional | `{% if died %}(d. {{ died }}){% endif %}` |
| `{% for x in list %}...{% endfor %}` | Loop | Iterate over values |
| `{{ x \| filter }}` | Filters | `{{ name \| upper }}` → EINSTEIN |

After expansion, normal card syntax applies (`->`, `<->`, `[[]]`, `=>`).

**Example with conditionals:**

~~~markdown
```template
{{ name }} ({{ born }}{% if died %}-{{ died }}{% endif %}) was a {{ field }}.
{% if contribution %}Known for: [[{{ contribution }}]].{% endif %}
```

name | born | died | field | contribution
einstein | 1879 | 1955 | physicist | theory of relativity
hawking | 1942 | 2018 | physicist | black hole radiation
penrose | 1931 | | physicist | Penrose tilings
~~~

Penrose (no death date) expands to:
```
Penrose (1931) was a physicist. Known for: [[Penrose tilings]].
```

#### Removed (redundant)

| Removed | Use instead |
|---------|-------------|
| `a <- b` | Write `b -> a` |
| `a ->> b` | Write `a [[b]].` |
| `a \| b \| c` multi-field | Use table or multiple attributes |

### Example Document (All Syntax Demonstrated)

```markdown
# Spanish Vocabulary

## Greetings
- hello <-> hola
- goodbye <-> adiós
- good morning <-> buenos días

## Ser (present tense)
subject | conjugation <->
yo | soy
tú | eres
él/ella | es
nosotros | somos
ellos | son

## Irregular Verbs (reverse practice)
- was (ser) -> fue
- did/made (hacer) -> hizo

# Biology 101

## The Cell
The [[mitochondria]] is the powerhouse of the cell.
DNA is stored in the [[nucleus]].
Proteins are synthesized by [[ribosomes]].

## Photosynthesis
- chlorophyll -> pigment that captures light
- stomata -> pores for gas exchange

# Chemistry

## Elements
element | symbol <-> | atomic # -> | phase
hydrogen | H | 1 | gas
helium | He | 2 | gas
lithium | Li | 3 | solid

# Medicine

## Drug Classes
drug | class -> | uses ->
aspirin | NSAID | pain relief, fever reduction, heart attack prevention
metformin | biguanide | type 2 diabetes (first-line), PCOS treatment
lisinopril | ACE inhibitor | hypertension, heart failure, kidney protection in diabetics

# Troubleshooting: Wi-Fi

## Steps
=> check Wi-Fi is enabled
=> restart device
=> forget network and reconnect
=> restart router
=> check firmware updates

# US Government

## Branches
- makes laws -> Legislative (Congress)
- enforces laws -> Executive (President)
- interprets laws -> Judicial (Supreme Court)

# Comparison (no cards, just reference)

term | definition | example
affect | verb, to influence | weather affects mood
effect | noun, a result | the effect was immediate

# History

## Scientists

```template
{{ name }} ([[{{ born }}]]-[[{{ died }}]]) was a {{ nationality }} {{ field }}
known for [[{{ contribution }}]].
```

name | born | died | nationality | field | contribution
einstein | 1879 | 1955 | German | physicist | theory of relativity
darwin | 1809 | 1882 | English | naturalist | theory of evolution
curie | 1867 | 1934 | Polish | chemist | discovering radium

## US Presidents

```template
{{ name }} was the [[{{ number }}]] president of the United States,
serving from [[{{ start }}]] to [[{{ end }}]].
```

name | number | start | end
washington | 1st | 1789 | 1797
lincoln | 16th | 1861 | 1865
fdr | 32nd | 1933 | 1945

## Battles (with conditionals)

```template
The [[{{ battle }}]] ({{ year }}) was fought between {{ side1 }} and {{ side2 }}.
{% if victor %}{{ victor }} was victorious.{% endif %}
{% if significance %}Significance: {{ significance }}{% endif %}
```

battle | year | side1 | side2 | victor | significance
thermopylae | 480 BC | Greeks | Persians | Persians | Spartan last stand
hastings | 1066 | Normans | Saxons | Normans | Norman conquest of England
gettysburg | 1863 | Union | CSA | Union | Turning point of Civil War
```

### What the Cards Would Look Like

---

#### `<->` Bidirectional

Each line produces TWO cards, one in each direction.

| Syntax | Card 1 Front | Card 1 Back | Card 2 Front | Card 2 Back |
|--------|--------------|-------------|--------------|-------------|
| `hello <-> hola` | **Greetings:** hello | hola | **Greetings:** hola | hello |
| `goodbye <-> adiós` | **Greetings:** goodbye | adiós | **Greetings:** adiós | goodbye |

---

#### `->` Basic

One-direction cards. Question on left, answer on right.

| Syntax | Front | Back |
|--------|-------|------|
| `yo -> soy` | **Ser (present tense):** yo | soy |
| `makes laws -> Legislative (Congress)` | **Branches:** makes laws | Legislative (Congress) |
| `was (ser) -> fue` | **Irregular Verbs:** was (ser) | fue |

*Note:* For "reverse practice" (see Spanish, produce English), just write the card that direction.

---

#### `[[...]]` Cloze

Text with a hidden word. The sentence IS the question; the answer is just the hidden word.

| Syntax | Front | Back |
|--------|-------|------|
| `The [[mitochondria]] is the powerhouse of the cell.` | The **[...]** is the powerhouse of the cell. | mitochondria |
| `DNA is stored in the [[nucleus]].` | DNA is stored in the **[...]**. | nucleus |
| `Proteins are synthesized by [[ribosomes]].` | Proteins are synthesized by **[...]**. | ribosomes |

---

#### `=>` Ordered Sequence

Lines prefixed with `=>` form a sequence. The system generates "After X?" cards.

```
=> check Wi-Fi is enabled
=> restart device
=> forget network and reconnect
=> restart router
=> check firmware updates
```

| # | Front | Back |
|---|-------|------|
| 1 | **Steps:** First? | check Wi-Fi is enabled |
| 2 | **Steps:** After "check Wi-Fi is enabled"? | restart device |
| 3 | **Steps:** After "restart device"? | forget network and reconnect |
| 4 | **Steps:** After "forget network and reconnect"? | restart router |
| 5 | **Steps:** After "restart router"? | check firmware updates |

*Note:* The first item gets a "First?" card. Each subsequent item gets an "After X?" card.

---

#### `# Heading` + `key -> value` Attribute Cards

Heading becomes the subject, each `key -> value` underneath becomes a card.

| Source | Front | Back |
|--------|-------|------|
| `# Hydrogen` | | |
| `- symbol -> H` | **Hydrogen:** symbol | H |
| `- atomic number -> 1` | **Hydrogen:** atomic number | 1 |
| `- discovered by -> Cavendish` | **Hydrogen:** discovered by | Cavendish |

*Use case:* Single entities with multiple properties.

---

#### Pipe Block Tables

Arrows in column headers define which columns generate cards and in what direction. A contiguous block of pipe-separated lines forms a table; a blank line ends it.

**Example 1: Conjugation table (bidirectional)**

```
subject | conjugation <->
yo | soy
tú | eres
```

| Direction | Front | Back |
|-----------|-------|------|
| Forward | **Ser (present tense):** yo, Conjugation? | soy |
| Forward | **Ser (present tense):** tú, Conjugation? | eres |
| Backward | **Ser (present tense):** Conjugation: soy | yo |
| Backward | **Ser (present tense):** Conjugation: eres | tú |

**Example 2: Elements table (mixed directions)**

```
element | symbol <-> | atomic # -> | phase
hydrogen | H | 1 | gas
helium | He | 2 | gas
```

| Column | Direction | Front | Back |
|--------|-----------|-------|------|
| Symbol | Forward | **Elements:** Hydrogen, Symbol? | H |
| Symbol | Backward | **Elements:** Symbol: H | Hydrogen |
| Atomic # | Forward | **Elements:** Hydrogen, Atomic #? | 1 |
| Phase | — | *No cards (no arrow in header)* | — |

**Example 3: Multi-value cells**

```
drug | uses ->
aspirin | pain relief, fever reduction, heart attack prevention
metformin | type 2 diabetes (first-line), PCOS treatment
```

| Drug | Front | Back |
|------|-------|------|
| Aspirin | **Drug Classes:** Aspirin, Uses? | pain relief, fever reduction, heart attack prevention |
| Metformin | **Drug Classes:** Metformin, Uses? | type 2 diabetes (first-line), PCOS treatment |

**Example 4: Reference table (no cards)**

```
term | definition | example
affect | verb, to influence | weather affects mood
effect | noun, a result | the effect was immediate
```

No arrows in any header = no cards generated. Just a reference table.

*Use case:* Conjugations, element properties, vocabulary with multiple attributes, any grid of related data.

---

#### Template + Pipe Block (Metaprogramming)

A `template` code fence transforms each pipe block row using Jinja2/Tera syntax.

**Example: Scientists**

~~~markdown
```template
{{ name }} ([[{{ born }}]]-[[{{ died }}]]) was a {{ nationality }} {{ field }} 
known for [[{{ contribution }}]].
```

name | born | died | nationality | field | contribution
einstein | 1879 | 1955 | German | physicist | theory of relativity
darwin | 1809 | 1882 | English | naturalist | theory of evolution
~~~

**Step 1: Expand template for each row**
```
Einstein ([[1879]]-[[1955]]) was a German physicist known for [[theory of relativity]].
Darwin ([[1809]]-[[1882]]) was an English naturalist known for [[theory of evolution]].
```

**Step 2: Parse expanded text for card syntax**

| # | Front | Back |
|---|-------|------|
| 1 | Einstein ([...]-[...]) was a German physicist known for theory of relativity. | 1879, 1955 |
| 2 | Einstein (1879-1955) was a German physicist known for [...]. | theory of relativity |
| 3 | Darwin ([...]-[...]) was an English naturalist known for theory of evolution. | 1809, 1882 |
| 4 | Darwin (1809-1882) was an English naturalist known for [...]. | theory of evolution |

**Example with conditionals:**

~~~markdown
```template
{{ name }} ({{ born }}{% if died %}-{{ died }}{% endif %}).
{% if contribution %}Known for: [[{{ contribution }}]].{% endif %}
```
~~~

If `died` is empty, that part is omitted. If `contribution` is empty, no cloze card for that row.

**More template examples:**

```template
- {{ english }} <-> {{ spanish }}
```
→ Generates bidirectional vocabulary cards

```template
{{ term }} -> {{ definition }}
{% if example %}Example: {{ example }}{% endif %}
```
→ Basic cards with optional example text

```template
=> {{ step }}
```
→ Generates sequence cards from a "step" column

```template
{% for lang in ["english", "spanish", "french"] %}
- {{ lang }}: {{ self[lang] }}
{% endfor %}
```
→ Loop over columns dynamically

*Use case:* Any structured data you want to learn in sentence context, with full control over formatting.

---

### Syntax Summary

| Syntax | Meaning | Cards Generated | Example Output |
|--------|---------|-----------------|----------------|
| `a -> b` | basic | 1 | Q: a → A: b |
| `a <-> b` | bidirectional | 2 | Q: a → A: b, Q: b → A: a |
| `text [[x]] text` | cloze | 1 | Q: text [...] text → A: x |
| `=> step` (line prefix) | sequence | 1 per step | Q: First? / After X? → A: step |
| `# Heading` + `k -> v` | attribute | 1 per line | Q: Heading: k → A: v |
| pipe block + `col ->` header | table (forward) | 1 per row | Q: Row, Col? → A: cell |
| pipe block + `col <->` header | table (bidirectional) | 2 per row | both directions |
| pipe block (no arrows) | reference only | 0 | No cards, just display |
| ` ```template ` + pipe block | metaprogramming | varies | Template expanded per row, then parsed |

---

## Part 4B: Applying the 20 Rules of Formulating Knowledge

Reference: [SuperMemo's 20 Rules](https://super-memory.com/articles/20rules.htm)

### Key Principles That Reshape Our Format

#### 1. Minimum Information Principle (Rule 4)

**The most violated rule.** Answers should be as SHORT as possible. The brain should light up the right memory instantly.

| Bad | Good |
|-----|------|
| Q: What is the Dead Sea?<br>A: Salt lake located on the border between Israel and Jordan. Its shoreline is the lowest point on Earth's surface, averaging 396 m below sea level. It is 74 km long and seven times as salty as the ocean. | Q: Where is the Dead Sea?<br>A: **Israel/Jordan border** |

This means one "complex" fact should become **many simple cards**:

```
Dead Sea:
- location -> Israel/Jordan border
- lowest point on Earth -> its shoreline
- depth below sea level -> 400m
- length -> 70 km
- saltier than ocean by -> 7x
```

Each answer is 1-4 words. The brain retrieves ONE fact per repetition.

#### 2. Avoid Sets (Rule 9)

Sets are **nearly impossible to memorize**. Never ask "What are the X?"

| Terrible (set) | Better (individual items with context) |
|----------------|----------------------------------------|
| Q: What are the three branches of US government?<br>A: Legislative, Executive, Judicial | Q: US government: which branch makes laws?<br>A: **Legislative (Congress)**<br><br>Q: US government: which branch enforces laws?<br>A: **Executive (President)**<br><br>Q: US government: which branch interprets laws?<br>A: **Judicial (Supreme Court)** |

The "better" version teaches the same content but each card has ONE answer and includes the *function* which aids memory.

#### 3. Avoid Enumerations (Rule 10)

Ordered sequences are hard. Use **overlapping cloze deletions**:

| Hard (full sequence) | Easy (overlapping cloze) |
|----------------------|--------------------------|
| Q: What are the steps to troubleshoot Wi-Fi?<br>A: 1. Check Wi-Fi enabled 2. Restart device 3. Forget network 4. Restart router 5. Check firmware | Q: Wi-Fi fix: first step?<br>A: **Check Wi-Fi is enabled**<br><br>Q: Wi-Fi fix: after checking Wi-Fi is enabled?<br>A: **Restart the device**<br><br>Q: Wi-Fi fix: after restarting device?<br>A: **Forget network and reconnect** |

Each card asks for ONE step. Overlapping means "restart device" appears as both answer and context.

#### 4. Cloze Deletion (Rule 5)

Cloze is the **fastest way to create good cards** from text. The context remains, only one piece is hidden:

```
The [[mitochondria]] is the powerhouse of the cell.
```

Becomes:
| Front | Back |
|-------|------|
| The **[...]** is the powerhouse of the cell. | mitochondria |

The sentence provides context. The answer is ONE WORD.

#### 5. Combat Interference (Rule 11)

Similar items confuse each other. Solutions:
- Make items **unambiguous**
- Add **distinguishing context**
- Use **different examples** for similar concepts

| High interference | Low interference |
|-------------------|------------------|
| Q: 401 HTTP status?<br>A: Unauthorized<br><br>Q: 403 HTTP status?<br>A: Forbidden | Q: HTTP 401: you need to [...] first<br>A: **log in** (Unauthorized)<br><br>Q: HTTP 403: you're logged in but [...] to access this<br>A: **not allowed** (Forbidden) |

The improved version encodes the *meaning* difference, not just the label.

#### 6. Optimize Wording (Rule 12)

Fewer words = faster processing = better retention.

| Wordy | Optimized |
|-------|-----------|
| Q: What is the keyboard shortcut in VS Code on Mac that opens the command palette?<br>A: Cmd+Shift+P | Q: VS Code Mac: command palette<br>A: **Cmd+Shift+P** |

The question is a **label**, not a sentence. Your brain knows the context from the category.

#### 7. Context Cues (Rule 16)

Labels/prefixes put the brain in the right mode instantly:

| Without context | With context |
|-----------------|--------------|
| Q: What does GRE stand for?<br>A: ??? (Graduate Record Exam? Glucocorticoid Response Element?) | Q: **bioch:** GRE<br>A: glucocorticoid response element |

Our nested bullet structure provides this naturally:
```
- HTTP Status Codes
  - 2xx: success
    - 200 -> OK
```
Generates: **HTTP Status Codes › 2xx: success:** 200 → ?

---

### Anti-Patterns to Avoid

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| Long answers | Brain can't retrieve in one path | Split into multiple cards |
| "List the X" | Sets are unmemorable | One card per item |
| "What are the steps?" | Enumerations fail | Overlapping cloze |
| Similar Q, different A | Interference | Add distinguishing context |
| Full sentences as Q | Slow processing | Use labels + minimal words |
| No context | Wrong memory activates | Add category prefix |

---

## Part 5: Priority Ranking for Implementation

### High Value, Low Complexity
1. **Cloze deletions** `[[text]]` — extremely useful, simple to parse
2. **Attribute cards** — `# Heading` + `key -> value`, natural pattern

### High Value, Medium Complexity
3. **Tables with arrow headers** — conjugations, element properties, vocabulary grids
4. **Ordered sequences** `=> step` — line prefix, requires tracking position

### Future Consideration
5. **Multi-cloze** `{{c1::text}}` — Anki-native syntax for multiple blanks
6. **Q&A blocks** — clear syntax, more verbose
7. **Decision trees** — complex structure

---

## Part 6: Extended Use Case Examples

### 6.1 Language Learning — Deep Dive

**Spanish verb conjugations (pipe block format):**
```
## hablar (to speak)

### Present
subject | form <->
yo | hablo
tú | hablas
él/ella | habla
nosotros | hablamos
ellos | hablan

### Preterite
subject | form <->
yo | hablé
tú | hablaste
él/ella | habló
nosotros | hablamos
ellos | hablaron
```

*Alternative (nested attributes, if you prefer):*
```
- hablar (to speak)
  - present
    - yo -> hablo
    - tú -> hablas  
    - él/ella -> habla
  - preterite
    - yo -> hablé
    - tú -> hablaste
```

**Idioms with context (cloze would help):**
```
Spanish idioms:
- "No hay moros en la costa" -> The coast is clear (literally: no Moors on the coast)
- "Estar en las nubes" -> To be daydreaming (literally: in the clouds)
- "Meter la pata" -> To put your foot in it / make a mistake
```

**Grammar rules (Q&A format):**
```
Q: When do you use ser vs estar in Spanish?
A: Ser for permanent traits (identity, profession, origin). 
   Estar for temporary states (location, mood, condition).

Q: What triggers the subjunctive in Spanish?
A: WEIRDO: Wishes, Emotions, Impersonal expressions, 
   Recommendations, Doubt/denial, Ojalá
```

**Pronunciation rules:**
```
French pronunciation:
- silent final consonants -> except C, R, F, L (CaReFuL)
- "oi" -> /wa/ sound
- "eau" -> /o/ sound
- liaison occurs -> when word ends in consonant, next starts with vowel
```

---

### 6.2 Mathematics — Deep Dive

**Calculus formulas (dense table):**
```
Integration rules:
- ∫ x^n dx -> (x^(n+1))/(n+1) + C (n ≠ -1)
- ∫ 1/x dx -> ln|x| + C
- ∫ e^x dx -> e^x + C
- ∫ sin(x) dx -> -cos(x) + C
- ∫ cos(x) dx -> sin(x) + C
- ∫ sec²(x) dx -> tan(x) + C
```

**Linear algebra (definitions with examples):**
```
- eigenvalue
  - definition -> scalar λ where Av = λv for some nonzero v
  - how to find -> solve det(A - λI) = 0
  - geometric meaning -> factor by which eigenvector is scaled

- determinant
  - 2x2 formula -> ad - bc for [[a,b],[c,d]]
  - geometric meaning -> signed area/volume scaling factor
  - when zero -> matrix is singular (not invertible)
```

**Proof techniques (ordered steps):**
```
Proof by induction:
1. Base case: prove P(1) or P(0) is true
2. Inductive hypothesis: assume P(k) is true
3. Inductive step: prove P(k) implies P(k+1)
4. Conclusion: P(n) holds for all n ≥ base case
```

**Set theory symbols:**
```
Set notation:
- ∈ <-> element of
- ⊂ <-> proper subset
- ⊆ <-> subset or equal
- ∪ <-> union
- ∩ <-> intersection
- ∅ <-> empty set
- ℕ <-> natural numbers
- ℤ <-> integers
- ℚ <-> rationals
- ℝ <-> reals
```

---

### 6.3 Medicine/Anatomy — Deep Dive

**Cranial nerves (ordered sequence):**
```
Cranial nerves (I-XII):
1. Olfactory -> smell
2. Optic -> vision
3. Oculomotor -> eye movement, pupil
4. Trochlear -> eye movement (superior oblique)
5. Trigeminal -> face sensation, chewing
6. Abducens -> eye movement (lateral rectus)
7. Facial -> facial expression, taste
8. Vestibulocochlear -> hearing, balance
9. Glossopharyngeal -> taste, swallowing
10. Vagus -> parasympathetic to organs
11. Accessory -> shoulder shrug, head turn
12. Hypoglossal -> tongue movement

Mnemonic: "Oh Oh Oh To Touch And Feel Very Good Velvet, AH"
```

**Drug classes (attribute lists):**
```
# Aspirin
- class: NSAID, antiplatelet
- mechanism: irreversibly inhibits COX-1 and COX-2
- uses: pain, fever, MI prevention, stroke prevention
- contraindications: bleeding disorders, children with viral illness (Reye's)
- side effects: GI bleeding, tinnitus at high doses

# Metformin
- class: biguanide
- mechanism: decreases hepatic glucose production, increases insulin sensitivity
- uses: type 2 diabetes (first-line)
- contraindications: renal impairment, contrast dye procedures
- side effects: GI upset, lactic acidosis (rare)
```

**Diagnostic criteria (lists):**
```
Diagnosis: Rheumatoid Arthritis (need 4 of 7):
1. Morning stiffness > 1 hour
2. Arthritis of 3+ joint areas
3. Arthritis of hand joints
4. Symmetric arthritis
5. Rheumatoid nodules
6. Positive serum RF
7. Radiographic changes
```

---

### 6.4 Computer Science — Deep Dive

**Big-O complexities:**
```
Algorithm complexities:
- binary search -> O(log n)
- linear search -> O(n)
- bubble sort -> O(n²)
- merge sort -> O(n log n)
- quicksort average -> O(n log n)
- quicksort worst -> O(n²)
- hash table lookup -> O(1) average
```

**Data structure operations (table format would help):**
```
Array:
- access by index -> O(1)
- search -> O(n)
- insert at end -> O(1) amortized
- insert at middle -> O(n)
- delete -> O(n)

Linked List:
- access by index -> O(n)
- search -> O(n)
- insert at head -> O(1)
- insert at middle -> O(n) to find, O(1) to insert
- delete -> O(n) to find, O(1) to delete
```

**Git commands:**
```
git commands:
- stage all changes -> git add -A
- unstage file -> git reset HEAD <file>
- discard changes -> git checkout -- <file>
- see staged diff -> git diff --staged
- amend last commit -> git commit --amend
- revert a commit -> git revert <hash>
- cherry-pick -> git cherry-pick <hash>
- interactive rebase -> git rebase -i <base>
```

**Regex patterns:**
```
Regex:
- any single char -> .
- zero or more -> *
- one or more -> +
- zero or one -> ?
- word boundary -> \b
- digit -> \d
- whitespace -> \s
- word char -> \w
- start of string -> ^
- end of string -> $
- capture group -> (...)
- non-capturing -> (?:...)
- lookahead -> (?=...)
- lookbehind -> (?<=...)
```

**Keyboard shortcuts (app-specific):**
```
VS Code shortcuts (Mac):
- command palette -> Cmd+Shift+P
- go to file -> Cmd+P
- go to symbol -> Cmd+Shift+O
- go to line -> Ctrl+G
- toggle sidebar -> Cmd+B
- split editor -> Cmd+\
- close tab -> Cmd+W
- find in files -> Cmd+Shift+F
- replace -> Cmd+Option+F
- toggle terminal -> Ctrl+`
```

---

### 6.5 History — Deep Dive

**Timeline events (ordered, with context):**
```
American Revolution timeline:
1. 1765 -> Stamp Act, first direct tax on colonies
2. 1770 -> Boston Massacre
3. 1773 -> Boston Tea Party
4. 1775 -> Battles of Lexington and Concord, war begins
5. 1776 -> Declaration of Independence signed
6. 1777 -> Battle of Saratoga, French alliance
7. 1781 -> Siege of Yorktown, British surrender
8. 1783 -> Treaty of Paris, independence recognized
```

**Cause and effect chains:**
```
WWI causes:
- Assassination of Franz Ferdinand -> Austria declares war on Serbia
- Austria-Serbia war -> Russia mobilizes to defend Serbia
- Russia mobilizes -> Germany declares war on Russia
- Germany at war -> France mobilizes (alliance with Russia)
- Germany invades Belgium -> Britain declares war on Germany
```

**Historical figures (attribute cards):**
```
# Alexander the Great
- lived: 356-323 BC
- title: King of Macedon
- teacher: Aristotle
- famous for: conquered Persian Empire, spread Hellenistic culture
- died: Babylon, age 32, cause unknown (fever)

# Julius Caesar
- lived: 100-44 BC
- title: Dictator of Rome
- famous for: conquered Gaul, crossed the Rubicon, reformed calendar
- died: assassinated, Ides of March (March 15), 44 BC
```

---

### 6.6 Music Theory — Deep Dive

**Intervals:**
```
Musical intervals:
- unison -> 0 semitones
- minor 2nd -> 1 semitone
- major 2nd -> 2 semitones
- minor 3rd -> 3 semitones
- major 3rd -> 4 semitones
- perfect 4th -> 5 semitones
- tritone -> 6 semitones
- perfect 5th -> 7 semitones
- minor 6th -> 8 semitones
- major 6th -> 9 semitones
- minor 7th -> 10 semitones
- major 7th -> 11 semitones
- octave -> 12 semitones
```

**Chord construction:**
```
Chord formulas (from root):
- major -> 1, 3, 5
- minor -> 1, b3, 5
- diminished -> 1, b3, b5
- augmented -> 1, 3, #5
- major 7 -> 1, 3, 5, 7
- dominant 7 -> 1, 3, 5, b7
- minor 7 -> 1, b3, 5, b7
```

**Key signatures:**
```
Key signatures (sharps, circle of 5ths):
- C major -> 0 sharps
- G major -> 1 sharp (F#)
- D major -> 2 sharps (F#, C#)
- A major -> 3 sharps (F#, C#, G#)
- E major -> 4 sharps (F#, C#, G#, D#)
- B major -> 5 sharps
- F# major -> 6 sharps
```

---

### 6.7 Chemistry — Deep Dive

**Functional groups:**
```
Organic functional groups:
- hydroxyl (-OH) -> alcohols
- carbonyl (C=O) -> aldehydes, ketones
- carboxyl (-COOH) -> carboxylic acids
- amino (-NH2) -> amines
- sulfhydryl (-SH) -> thiols
- phosphate (-PO4) -> nucleotides, ATP
- ester (-COO-) -> fats, polyesters
```

**Periodic table trends:**
```
Periodic trends:
- atomic radius -> increases down, decreases right
- ionization energy -> decreases down, increases right
- electronegativity -> decreases down, increases right
- metallic character -> increases down, decreases right
```

**Reaction types:**
```
Reaction types:
- synthesis -> A + B → AB
- decomposition -> AB → A + B
- single replacement -> A + BC → AC + B
- double replacement -> AB + CD → AD + CB
- combustion -> fuel + O2 → CO2 + H2O
```

---

### 6.8 Practical/Household — Deep Dive

**Cooking conversions:**
```
Kitchen conversions:
- 3 teaspoons -> 1 tablespoon
- 4 tablespoons -> 1/4 cup
- 16 tablespoons -> 1 cup
- 2 cups -> 1 pint
- 2 pints -> 1 quart
- 4 quarts -> 1 gallon
- 1 stick butter -> 1/2 cup (8 tablespoons)
```

**Laundry care symbols:**
```
Laundry symbols:
- tub with water -> machine wash
- tub with hand -> hand wash only
- tub with X -> do not wash
- triangle -> bleach allowed
- triangle with X -> do not bleach
- square with circle -> tumble dry
- iron -> iron allowed
- iron with X -> do not iron
- circle -> dry clean
```

**First aid responses (procedural):**
```
Choking response (conscious adult):
1. Ask "Are you choking?"
2. Call 911 or have someone call
3. Stand behind victim
4. Find navel, place fist above
5. Grasp fist with other hand
6. Give quick upward thrusts
7. Repeat until object expelled or unconscious
8. If unconscious, begin CPR
```

**Tool uses:**
```
Which screwdriver:
- Phillips -> cross-shaped, most common
- flathead -> single slot
- Torx -> 6-point star, security screws
- Robertson -> square, common in Canada
- hex/Allen -> hexagonal, furniture assembly
```

---

### 6.9 Philosophy/Logic — Deep Dive

**Logical fallacies:**
```
Fallacies:
- ad hominem -> attacking the person, not the argument
- straw man -> misrepresenting opponent's position
- appeal to authority -> citing authority without evidence
- false dichotomy -> presenting only two options when more exist
- slippery slope -> claiming one thing inevitably leads to extreme
- circular reasoning -> conclusion is also the premise
- red herring -> introducing irrelevant topic
- post hoc -> assuming causation from sequence
```

**Philosophical terms:**
```
Philosophy terms:
- epistemology <-> study of knowledge
- ontology <-> study of being/existence
- ethics <-> study of right and wrong
- aesthetics <-> study of beauty/art
- a priori <-> knowledge before experience
- a posteriori <-> knowledge from experience
- phenomenology <-> study of conscious experience
```

---

### 6.10 Finance/Economics — Deep Dive

**Financial ratios:**
```
Financial ratios:
- current ratio -> current assets / current liabilities
- quick ratio -> (current assets - inventory) / current liabilities
- debt-to-equity -> total debt / total equity
- ROE -> net income / shareholders' equity
- ROA -> net income / total assets
- P/E ratio -> price per share / earnings per share
```

**Economic terms:**
```
Economics:
- GDP <-> total value of goods/services produced in a country
- inflation <-> general increase in prices, decrease in purchasing power
- deflation <-> general decrease in prices
- fiscal policy <-> government spending and taxation
- monetary policy <-> central bank control of money supply
- opportunity cost <-> value of next best alternative forgone
```

---

## Part 7: Implementation Priority Matrix

### Scoring Criteria

**Value (1-5):**
- How many use cases benefit?
- How much density improvement?
- How natural is the syntax?

**Effort (1-5):**
- Parser complexity
- Anki note type requirements
- Edge cases to handle

**Risk (1-5):**
- Breaking changes to current syntax?
- Ambiguity with existing patterns?
- Learning curve for users?

### Priority Matrix

| Format | Value | Effort | Risk | Score | Notes |
|--------|-------|--------|------|-------|-------|
| **Cloze `[[text]]`** | 5 | 2 | 1 | **8** | Universal, simple regex, maps to Anki cloze |
| **Attribute cards** | 4 | 2 | 1 | **7** | `# Heading` + `key -> value` pattern |
| **Tables with arrow headers** | 5 | 3 | 1 | **7** | Conjugations, elements, vocabulary grids |
| **Template + table** | 5 | 3 | 2 | **6** | Powerful metaprogramming, sentence context |
| **Ordered sequences `=> step`** | 4 | 3 | 1 | **6** | Line prefix, generates "After X?" cards |
| **Multi-cloze `{{c1::}}`** | 4 | 3 | 2 | **5** | Anki-compatible, multiple blanks |

### Recommended Roadmap

**Phase 1: Core Syntax**
1. Basic `->` and `<->` cards (already implemented)
2. Cloze deletions `[[text]]`
3. Attribute cards (heading context)

**Phase 2: Structured Data**
4. Tables with arrow headers
5. Ordered sequence cards with `=> step` prefix

**Phase 3: Metaprogramming**
6. Template + table expansion

**Future Consideration**
- Multi-cloze with ordering `{{c1::text}}`
- Image/audio references
- Tags inline syntax

---

## Part 8: Open Questions

1. **Cloze syntax:** `[[text]]` vs `{{text}}` vs Anki-native `{{c1::text}}`?
   - `[[text]]` is wiki-link style, might conflict with Obsidian links
   - `{{text}}` is Anki-compatible
   - Could support both?

2. **Sequence card generation:** What kinds of cards from `=> step` lines?
   - "First?" for the first item (current proposal)
   - "After X?" for subsequent items (current proposal)
   - Also generate "Before Y?" (reverse direction)?
   - Also generate "Step N?" (by position)?

3. **Backward compatibility:** Do any new syntaxes conflict with current `->` parsing?
   - `[[` and `]]` shouldn't conflict
   - `{{` and `}}` shouldn't conflict
   - `=>` as line prefix shouldn't conflict

4. **Tags:** How to add tags to cards inline?
   - `#tag` at end of line?
   - `@tag` to avoid markdown heading conflict?
   - `[tags: foo, bar]` block syntax?

5. **Difficulty/priority:** How to mark cards as "must know" vs "nice to know"?
   - `!` prefix for important?
   - `*` for optional?
   - Separate into different sections?

6. **Table context columns:** Should non-arrow columns be shown as context on cards?
   - e.g., "Phase" column shown on Hydrogen/Symbol card as hint?
   - RemNote supports "Extra Properties on Front/Back of Card"
   - Could use a different marker like `(Phase)` or `Phase?` for "show as context"

7. **Template syntax details:**
   - What if column name has spaces? `{column name}` or `{column_name}`?
   - Should we support conditionals? `{if field}{field}{/if}`
   - Should we support filters? `{name|uppercase}` or `{date|format:YYYY}`
   - Keep it simple initially, extend later?
