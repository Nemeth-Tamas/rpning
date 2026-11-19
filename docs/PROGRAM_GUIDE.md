# rpning full program guide

This is the complete reference for the currently implemented program. Start
with the [TUI user guide](TUI.md) for the screen, keys and keypad layers, then
use the sections below for calculations and programming. In the TUI, enter
`:help program` in CMD mode to read this guide without leaving the calculator.
Topic help is available through `:help settings`, `:help math`, `:help stack`,
`:help registers`, `:help macros`, `:help scripts` and `:help persistence`.
Use F6 to focus help output, then arrows/PageUp/PageDown/Home/End to browse.

Contents:

- [Starting the program](#starting-the-program)
- [Numbers and RPN basics](#numbers-and-rpn-basics)
- [Stack and transactions](#stack-and-transactions)
- [Settings and display](#settings-and-display)
- [Mathematics reference](#mathematics-reference)
- [Named registers](#named-registers)
- [Macros and programming](#macros-and-programming)
- [Scripts and automation](#scripts-and-automation)
- [Shortcuts and keypad customization](#shortcuts-and-keypad-customization)
- [History and completion](#history-and-completion)
- [Persistence and files](#persistence-and-files)
- [Command index](#command-index)
- [Limits and troubleshooting](#limits-and-troubleshooting)

## Starting the program

These instructions describe rpning v0.2.0, including extended mathematics and
the Windows TUI. Download a [prebuilt binary](https://github.com/Nemeth-Tamas/rpning/releases/tag/v0.2.0)
or build from source:

```powershell
git pull --ff-only
cargo build --release --locked
.\target\release\rpn.exe
```

| Invocation | Behavior |
| --- | --- |
| `rpn` | Windows TUI when both input and output are terminals; otherwise line REPL |
| `rpn --tui` | Explicit Windows TUI; requires terminal input/output |
| `rpn --repl` | Explicit ordinary REPL; also accepts piped input |
| `rpn --keypad` | Immediate keypad mode with one status line per action, Windows |
| `rpn --keypad-diagnose` | Windows console key-event diagnostics |
| `rpn --eval "3 4 +"` or `rpn -e "3 4 +"` | Evaluate once, print X and exit |
| `rpn --run FILE` or `rpn -f FILE` | Evaluate a whole UTF-8 RPN script atomically |
| `rpn --help` or `rpn -h` | CLI help |
| `rpn --version` or `rpn -V` | Version |

Linux/macOS default to the ordinary REPL. Their TUI/keypad adapters and manual
acceptance remain deferred. Pipes and redirected output never select the TUI
automatically. Interactive modes load saved settings/macros/history; pipes,
`--eval` and `--run` start fresh. In piped input, explicit application load
commands can restore state. In one-shot/scripts, use RPN definitions directly.

## Numbers and RPN basics

RPN puts operands before the operation. The stack is bottom to top; X is its
top, Y the next value, Z the third. `12 3 -` calculates Y minus X, giving 9.
Space-separated numbers push values. New input retains previous values in an
interactive or piped session. Named words are case sensitive.

```rpn
12 3 + 2 *
```

X is 30. In TUI NUM mode, press `12`, Enter, `3`, `+`, `2`, `*`. In CMD mode
type the whole expression and press Enter. NUM operators execute immediately;
CMD waits for Enter. See the TUI guide for pending-number behavior.

| Literal | Meaning |
| --- | --- |
| `42`, `-42`, `+42` | Exact signed decimal integer |
| `0.125`, `-2.5` | Exact decimal rational |
| `1.2e5`, `1.2e-5` | Exact scientific notation, 120000 and 0.000012 |
| `1/3`, `-7/4` | Exact rational fraction; denominator cannot be zero |
| `0b101`, `0o17`, `0xff`, `-0x80` | Exact prefixed integers; uppercase prefixes accepted |

Unprefixed numbers stay decimal even when hexadecimal display is selected.
Text literals use a dot; NUM mode accepts dot or comma as the decimal key.
Use `2 10 pow` for powers; scientific notation uses e, not a power operator.
Attached arithmetic such as `3+`, `1-` and `2*` is accepted; whitespace makes
operand boundaries clearest. `#` begins a comment until the next newline.

Numbers are arbitrary-precision integers/rationals. Exact mode stores `0.1 0.2 +`
as exactly 0.3 and `1 3 /` as exactly 1/3. Approximate functions store finite
binary approximations as exact rationals; that representation does not make
an approximation to pi mathematically exact.

## Stack and transactions

In this table the rightmost item is X; older items below the shown values
remain untouched. Stack indices start at X=0, Y=1, Z=2.

| Word | Before -> after | Purpose |
| --- | --- | --- |
| `dup` | `a -> a a` | Duplicate X |
| `drop` | `a ->` | Remove X |
| `swap` | `a b -> b a` | Exchange X/Y |
| `over` | `a b -> a b a` | Copy Y |
| `rot` | `a b c -> b c a` | Rotate the top three left |
| `nip` | `a b -> b` | Remove Y |
| `tuck` | `a b -> b a b` | Copy X behind Y |
| `clear` | `... ->` | Clear the numeric stack, preserving registers/macros |
| `depth` | `... -> ... N` | Push stack depth before this push |
| `pick` | `... N -> ... value` | Consume N and copy the indexed remaining value |
| `roll` | `... N -> ... value` | Consume N and move the indexed remaining value to X |

`:stack` focuses the TUI stack; in the REPL it lists values. Use F3 and the
browsing keys to scroll in the TUI. `:undo`/`:redo` or Ctrl+Z/Y restore/reapply
state. With pending NUM input, undo first cancels that edit.

Each successful state-changing expression is one undo step, including all
nested macro calls. Undo restores stack, registers and macro definitions.
At most 128 snapshots are retained. New state changes after undo discard redo.
Settings, saved files, command history and the TUI journal are outside numeric
undo. Expressions that leave state unchanged need not create an undo step.

Failures roll back the entire evaluation. For example, with X=10, evaluating
`5 + 0 /` leaves X=10. In NUM mode the pending entry also survives a failed
operator. Read the red error, correct/cancel the input and retry. A failed
command can appear in history/journal while numeric undo remains unchanged.

## Settings and display

In TUI CMD mode, type a setting command to see hints below the input. Press
Enter without an argument for current value, allowed values and usage. Press
Tab for choices; F6 and browsing keys inspect long lists. `:display h` + Tab
completes `hex`. `:display fixed:` + Tab lists all 129 fixed-place values.
`:precision` completion suggests common values; every integer 2..4096 is valid.

| Command | Values | Effect |
| --- | --- | --- |
| `:mode` | `exact`, `binary` | Exact rational arithmetic or significant-bit rounding |
| `:precision` | Integer 2..4096 | Binary significant-bit precision, default 53 |
| `:round` | `nearest`, `zero`, `up`, `down`, `away` | Rounding direction |
| `:display` | `auto`, `scientific`, `fraction`, `binary`, `octal`, `hex`, `fixed:N` | Presentation of stored values |
| `:settings` | No argument | Show all current numeric/display settings |

Nearest uses ties to even; zero truncates toward zero; up means toward positive
infinity; down means toward negative infinity; away means away from zero.
Changing settings does not retroactively recalculate existing stack values.
`approx` explicitly rounds X using the current bit count/direction.

| Display mode | Behavior |
| --- | --- |
| `auto` | Exact terminating decimal up to 128 places, otherwise exact fraction |
| `scientific` | Lossless scientific representation of terminating values; fractions remain lossless |
| `fraction` | Exact canonical integer or fraction |
| `binary`, `octal`, `hex` | Prefixed integer output; nonintegers fall back to exact ordinary formatting |
| `fixed:N` | N decimal places, N=0..128, using selected rounding; rounded output starts with `~` |

`:display binary` changes integer presentation. `:mode binary` changes numeric
rounding; they are independent. Fixed display never rounds the stored value:
`1/3` at fixed:2 is `~0.33`; switching to fraction still reveals `1/3`.
Binary mode is bounded-significand arithmetic with an unbounded exponent,
not an IEEE float format. Large exact integers may be rounded in binary mode.
Use exact mode when doing large integer/bitwise/statistical exact calculations.

```text
:display fixed:2
1 3 /
:display fraction
:precision 128
:round nearest
:mode binary
1 3 /
:mode exact
```

Transcendental functions/constants use the selected precision even in exact
mode and currently support at most 512 bits. Settings persist automatically
in interactive sessions. `:settings-save`/`:settings-load` explicitly save/load
numeric settings, display, shortcuts and mappings. Settings are not undoable.

## Mathematics reference

Each example below is an independent fresh calculation in exact mode. Binary
mode can round intermediate operands/results; see Settings and display.

### Arithmetic, roots and integer mathematics

| Words | Inputs and meaning | Example -> X |
| --- | --- | --- |
| `+ - * /` | `Y X OP`; division is rational | `7 2 /` -> 3.5 |
| `div mod` | Integer quotient truncated toward zero; remainder has dividend sign | `-17 5 div` -> -3; `-17 5 mod` -> -2 |
| `neg abs` | Negate / absolute value of X | `-7 abs` -> 7 |
| `pow` | `BASE INTEGER_EXPONENT pow` | `2 -3 pow` -> 0.125 |
| `sqrt` | Exact rational square root | `9/16 sqrt` -> 0.75 |
| `root` | `VALUE DEGREE root`, exact rational root | `-8 3 root` -> -2 |
| `fact` | Nonnegative integer factorial | `5 fact` -> 120 |
| `gcd lcm` | Integer greatest common divisor / least common multiple | `12 18 gcd` -> 6; `12 18 lcm` -> 36 |
| `approx` | Round X to selected binary precision/direction | `1/3 approx` -> a finite binary approximation |

Roots reject irrational results, including `2 sqrt`, even in binary mode.
Power exponent magnitude is limited to 100000; 0 to a negative power errors,
and `0 0 pow` is 1. Root degree is 1..1024; even roots of negative values error.
Factorial is limited to 10000. Integer-only operations reject fractional inputs.

### Constants, logarithms, trigonometry and angles

| Words | Meaning/domain |
| --- | --- |
| `pi e` | Push approximate mathematical constants |
| `exp` | e to X, with absolute argument at most 256 |
| `ln log2 log10` | Natural/base-2/base-10 logarithm of positive X |
| `sin cos tan` | Radian trigonometry, absolute input at most 1000 radians |
| `rad` | Convert degrees to radians using approximate pi |
| `deg` | Convert radians to degrees using approximate pi |

Examples: `1 ln` -> 0; `0 exp` -> 1; `90 rad sin` is close to 1;
`pi deg` is close to 180. Accuracy depends on precision and argument reduction.
These functions use guard bits and independent reference tests but are not
certified correctly rounded for every input. Tangent rejects values too close
to a pole at the selected precision. Logarithms also bound binary input
exponents to +/-100000; series have an 8192-term budget. Precision is 2..512.

### Bases, bitwise and fixed-width interpretations

| Words | Inputs/effect | Example -> X |
| --- | --- | --- |
| `band bor bxor` | Integer Y/X bitwise AND, OR, XOR | `12 10 band` -> 8 |
| `bnot` | Integer bitwise complement | `0 bnot` -> -1 |
| `shl shr` | `VALUE COUNT`, left shift / arithmetic right shift | `-8 1 shr` -> -4 |
| `u8 i8 u16 i16 u32 i32 u64 i64 u128 i128` | Wrap low bits and interpret unsigned/signed | `255 i8` -> -1; `-1 u8` -> 255 |
| `asuint asint` | `VALUE BITS`, same conversion for width 1..4096 | `255 8 asint` -> -1 |

Signed bitwise operations use unbounded two's-complement semantics. Shift counts
are 0..100000. Width conversions are explicit operations, not a persistent
machine-word mode. A prefixed integer literal is limited to 100000 digits.

### Counted statistics

Push the dataset, then count N, then the operation. Items below the dataset
remain untouched. For example, `1 2 3 3 mean` -> 2 and
`4 1 3 2 4 median` -> 2.5. The final 3/4 is the count, not another observation.

| Words | Result |
| --- | --- |
| `sum` | Sum; empty dataset is 0 |
| `mean` | Arithmetic average |
| `median` | Middle value, or exact average of the two middle values |
| `min max` | Minimum / maximum |
| `var stdev` | Population variance / standard deviation |
| `svar sstdev` | Sample variance / standard deviation, dividing by N-1 |

N is an integer 0..10000; all operations except sum reject an empty dataset.
Sample statistics require N>=2. Standard deviation uses exact square roots,
so irrational results error. `1 2 3 3 svar` -> 1 and
`1 2 3 3 sstdev` -> 1. Mean/variance use rational arithmetic.

### Probability and combinatorics

| Word | Inputs | Example -> X |
| --- | --- | --- |
| `choose` | `N K choose`, combinations | `5 2 choose` -> 10 |
| `perm` | `N K perm`, ordered selections | `5 2 perm` -> 20 |
| `binomp` | `N K P binomp`, probability of exactly K successes | `2 1 1/2 binomp` -> 0.5 |

Require integer 0<=K<=N<=10000 and rational probability 0<=P<=1.
Binomial probability is choose(N,K) times P^K times (1-P)^(N-K).
Endpoint probabilities and N=0 are supported. No floating-point conversion
is needed in exact mode. See [extended mathematics](EXTENDED.md).

## Named registers

Registers are shared named values, separate from macro-local variables.

| Word/command | Effect |
| --- | --- |
| `sto:NAME` | Copy X into NAME without popping it |
| `rcl:NAME` | Push a copy of NAME; missing register errors |
| `del:NAME` | Delete NAME; missing register errors |
| `clearmem` | Clear all registers |
| `:registers` | List registers and formatted values; TUI uses scrollable output |
| `:save` | Save exact registers to the default register file |
| `:load` | Restore registers from that file; stack is retained |

Names are case sensitive, 1..64 ASCII characters, starting with a letter or
underscore, followed by letters/digits/underscores. Macro names can additionally
contain dots; register names do not. Storage/loading is explicit, not automatic.

```text
100 sto:net
clear
rcl:net 1.27 * sto:gross
:registers
:save
```

X is 127 and both net/gross remain stored. Register writes/deletions/loads are
undoable calculator changes; writing a file is not reversed by undo. Malformed
register files fail before replacing live register contents.

## Macros and programming

Use CMD mode for macro definitions and application commands. A definition is
`: NAME "optional description" BODY ;`. Keep one interactive definition on one
line; script definitions may span lines. Macro names are case-sensitive ASCII
identifiers, optionally containing dots for namespaces, at most 64 characters.
Start with a letter/underscore; built-in words/control words cannot be shadowed.

```rpn
: math.square "Square X" dup * ;
: math.fourth math.square math.square ;
: price.gross "Add 27 percent" 1.27 * ;
100 price.gross
```

X is exactly 127. A macro consumes/pushes whatever its body does; there is no
declared parameter list. Definitions compile once; calls resolve at execution
time, allowing forward references and recursion. Redefining a valid existing
macro replaces it and is undoable. Definitions persist automatically in
interactive modes after successful changes, including undo/redo.

### Inspection, aliases and tracing

| Command | Action |
| --- | --- |
| `:macros` | List names and descriptions |
| `:see NAME` | Show source and compiled instructions |
| `:alias NAME TARGET_WORD` | Define a compiled wrapper around one target word |
| `:trace EXPRESSION` | Execute and show an instruction trace; this changes calculator state |
| `:import PATH` | Merge a UTF-8 library containing definitions/comments only |
| `:export PATH` | Export the current macro library as source |
| `:macros-save` / `:macros-load` | Explicit default-library persistence |

Example: `:alias sq math.square`, then `9 sq` -> 81. `:trace 5 sq` executes the
expression and leaves 25 in X; use undo if the calculation was only exploratory.
Trace output is capped at 4096 lines of up to 240 characters per trace line.
TUI inspection/trace output can be scrolled with F6. Macro words themselves
perform calculator operations; file I/O is available through application commands.

### Comparisons, truth and conditionals

`= != < <= > >=` compare Y with X, consuming both and pushing 1 for true or 0
for false. `not` maps zero to 1 and any nonzero value to 0.
`if ... [else ...] then` consumes a flag; any nonzero value is true.

```rpn
: absolute dup 0 < if neg then ;
-7 absolute
: classify 0 < if -1 else 1 then ;
```

The first calculation yields 7. Classify consumes its input and produces -1
for a negative input, 1 otherwise. Conditionals and loops can be nested.

### Counted loops, while loops and recursion

`N times BODY end` consumes a nonnegative integer count. `while CONDITION do
BODY end` reevaluates CONDITION before every iteration and consumes its flag.

```rpn
0 5 times 2 + end
0 while dup 10 < do 1 + end
: factorial dup 1 <= if drop 1 else dup 1 - factorial * then ;
8 factorial
```

Run these as independent calculations: results are 10, 10 and 40320. The built-in
`fact` is preferable for ordinary factorial calculations; the recursive example
demonstrates macro calls and requires a nonnegative integer input.

### Local variables and shared state

`let:NAME` consumes X into a local variable; `get:NAME` pushes it. Variables
belong to the current call, so recursive/nested calls have distinct locals.
Reading an undefined local errors. Use registers for shared state across calls.

```rpn
: rectangle "WIDTH HEIGHT -> AREA" let:height get:height * ;
4 7 rectangle
```

X is 28. Descriptions are quoted strings, with escapes for quote, backslash,
newline, carriage return and tab. Strings are metadata in definitions, not
general text values on the numeric stack. `#` comments run to newline.

### Import/export and execution limits

```text
:export "C:/calc/my macros.rpn"
:import "C:/calc/my macros.rpn"
```

Paths are relative to the current working directory unless absolute; enclosing
quotes allow spaces. Import merges definitions atomically and rejects top-level
calculations. It does not run those calculations. Exports preserve source and
descriptions. There is no dedicated undefine command; undo can reverse a recent
definition and replacing a library file alone does not delete live definitions.

Bounds: source 1 MiB, 100000 tokens, 10000 macros, 64 nested parser blocks,
64 nested calls, 100000 executed instructions per evaluation and at most
100000 explicitly pushed stack values. Arithmetic has additional size/domain
limits. These bounds do not promise a fixed time for large exact calculations.
Failed calls/loops/definitions roll back the entire expression and preserve redo.
See [language details](LANGUAGE.md) for implementation-oriented examples.

## Scripts and automation

A script is UTF-8 RPN source. It can contain definitions, comments, expressions
and control flow, with multiline definitions. Interactive application commands
such as `:display` and `:import` are not RPN script words.

```rpn
# example.rpn
: gross 1.27 * ;
100 gross
```

```powershell
.\target\release\rpn.exe --run example.rpn
.\target\release\rpn.exe --eval ': square dup * ; 9 square'
"12", "3+", "2*" | .\target\release\rpn.exe
```

These print 127, 81, and the successive pipe results 12, 15, 30. `--run`/`--eval`
evaluate one whole transaction and return nonzero on error. Piped input runs one
complete expression/application command per line; a failed line rolls back
that line and later lines still run, with a nonzero final status. Pipe blank
lines are ignored. To explicitly load settings/macros in a pipe, send
`:settings-load` and `:macros-load` before expressions. Registers require `:load`.
One-shot scripts start fresh and do not automatically load or save app state.

## Shortcuts and keypad customization

The [TUI layer guide](TUI.md#keypad-layers) lists every default mapping.
An Alt layer is a persistent F2 toggle, not holding the keyboard Alt modifier.
Navigation events use the nav layer even when a different character layer is
active. Browsing panes reserve navigation keys for browsing.

```text
: price.gross 1.27 * ;
:shortcut vat price.gross
:bind alt 9 @shortcut:vat
:bindings
:shortcuts
```

`:shortcut NAME EXPRESSION` creates/replaces a named expression; `:do NAME`
executes it. `:bind LAYER KEY ACTION_OR_EXPRESSION` creates/replaces a mapping.
Both accept compiled RPN expressions but reject macro definitions inside them.
Macro references resolve when executed. In NUM mode a bound expression commits
the pending number and expression atomically. `:do` is an application command:
it evaluates its shortcut on the committed stack, leaving a pending NUM entry
untouched. Commit/cancel that entry first when mixing modes.

`:unbind LAYER KEY` removes a configured mapping, revealing the built-in
fallback. It does not create an explicit disabled mapping. `:bindings` lists
configured mappings; built-in base fallbacks are documented in the TUI guide.
Actions are `@enter`, `@undo`, `@redo`, `@sign`, `@cancel`, `@clear`, `@swap`,
`@backspace`, `@quit`, `@layer:NAME` and `@shortcut:NAME`.
Custom layer names can be selected using an `@layer:NAME` binding; F2 returns
from any non-base layer to base. Bind another key in the custom layer to return
or use F2. Settings/mappings persist automatically in interactive sessions.

## History and completion

Three kinds of history are different:

| History | Contents and lifetime |
| --- | --- |
| TUI calculation journal | Newest-first session actions/errors, up to 256 entries; Enter recalls into CMD, without execution |
| Command history | Up to 1000 commands, consecutive duplicates collapsed; interactive load/save across sessions |
| Numeric undo/redo | Up to 128 successful state snapshots, in memory only |

`:history TEXT` searches saved command history by literal substring, shown in
scrollable TUI output. Bare `:history` focuses the TUI calculation journal; in
REPL it lists command history. `:history-save`/`:history-load` explicitly persist
the command history. Journal recall replaces CMD input and cancels pending NUM
input to prevent accidental extra operands. Press Enter again to evaluate the
recalled expression against the current stack; recall does not restore old state.

TUI CMD Tab completes calculator words, commands, macro names, register reads/
writes and numeric setting values. Ambiguous choices appear in scrollable output;
type more to narrow them. It completes the token at the cursor. File paths,
locals and custom layer names are not automatically completed.
`:complete PREFIX` lists ordinary command/word completions in all modes.

The ordinary Windows REPL supports Up/Down command history, Ctrl+R substring
search (repeat for earlier matches), Ctrl+U line clear, Left/Right/Home/End,
Backspace/Delete and Tab word completion. Esc clears the REPL line; Ctrl+C
cancels it; Ctrl+D with an empty line exits. These REPL controls differ from
the TUI, where Esc/Ctrl+C exit and F3 selects browsing focus. Linux/macOS use
ordinary line input plus command-based history/search/completion listing.

## Persistence and files

| File | Contents | When written/loaded |
| --- | --- | --- |
| `settings.txt` | Numeric/display settings, shortcuts and configured mappings | Interactive startup and changes; explicit settings-save/load |
| `macros.rpn` | Exported macro definitions | Interactive startup and changes, including undo/redo; explicit macros-save/load |
| `history.txt` | Command history | Interactive startup/normal exit; explicit history-save/load |
| `registers.txt` | Exact named register values | Explicit :save/:load only |

The numeric stack and TUI journal are not saved/restored. Interactive exit can
lose unsaved command history if the process is forcibly terminated. Settings,
macros and history use same-directory temporary files and replacement writes;
register writes also use a temporary file. Files are UTF-8 with versioned
headers where applicable. Invalid loads report errors before applying content.
Loading settings cannot change the current selected layer or replay calculations.

Default directory: `%APPDATA%/rpning` on Windows; otherwise
`$XDG_DATA_HOME/rpning`, falling back to `~/.local/share/rpning`.
`RPNING_HOME` overrides the state directory for all application files:

```powershell
$env:RPNING_HOME = 'C:/calc/profile'
.\target\release\rpn.exe
```

An existing malformed startup settings/macros/history file can prevent startup.
Inspect/back up the affected file or use a new RPNING_HOME directory. Use
`--repl` to select the ordinary editor; it still loads interactive state.
Piped input and one-shot/script evaluation start with defaults.

## Command index

Application commands are entered one per CMD/REPL input, separately from RPN
expressions. A leading colon followed by a space starts a macro definition.

| Command | Arguments / purpose |
| --- | --- |
| `:help` | TUI topic index; REPL command help |
| `:help tui/program/layers/settings/math/stack/registers/macros/scripts/persistence` | TUI-only embedded guide topics; use one topic |
| `:stack` | View/focus stack |
| `:registers` | List named registers |
| `:undo`, `:redo` | Numeric/definition history |
| `:save`, `:load` | Default register file |
| `:mode [exact/binary]` | Query/set arithmetic mode |
| `:precision [BITS]` | Query/set precision |
| `:round [MODE]` | Query/set rounding |
| `:display [MODE]` | Query/set presentation |
| `:settings` | Current settings |
| `:settings-save`, `:settings-load` | Explicit settings/mappings/shortcut persistence |
| `:history [TEXT]` | TUI journal focus, or saved-command search/list |
| `:history-save`, `:history-load` | Command-history persistence |
| `:complete PREFIX` | Ordinary word/command completion listing |
| `:shortcut NAME EXPRESSION`, `:shortcuts`, `:do NAME` | Define/list/execute shortcut |
| `:bind LAYER KEY ACTION_OR_EXPRESSION`, `:unbind LAYER KEY`, `:bindings` | Key mappings |
| `:macros`, `:see NAME`, `:trace EXPRESSION` | Macro inspection/execution trace |
| `:alias NAME TARGET_WORD` | Alias definition |
| `:import PATH`, `:export PATH` | Macro library merge/export |
| `:macros-save`, `:macros-load` | Default macro-library persistence |
| `:quit`, `:exit` | Exit interactive input |

## Limits and troubleshooting

The current value type is real rational. Complex numbers, vectors/matrices,
units, physical constants, finance, numerical calculus, equation solvers and
custom math extensions remain roadmap items. Named macros already allow custom
compositions of implemented operations; they do not introduce new value types.

| Symptom | Action |
| --- | --- |
| Digits execute unexpected operations | Check footer layer; F2 returns Alt/custom to base; F1 focuses input; F5 selects NUM |
| Arrows do maths rather than browse | Focus Stack/History/Output with F3/F6; input-focus navigation uses nav bindings |
| Number remains in NUM while using CMD | Commit with NUM Enter or cancel with NUM Delete before changing workflows |
| A display value ends with an ellipsis | Stored value is intact; focus Result and scroll horizontally, or use fraction output in REPL |
| `root is not rational` | Exact-root policy rejects irrational results; changing precision does not add approximate sqrt |
| Transcendental precision error | Set precision to at most 512 for pi/e/logs/trig/conversions |
| Help/mappings appear cut off | F6 focuses output; browse its wrapped lines with arrows/pages/Home/End |
| Empty Enter errors | NUM Enter duplicates X; an empty stack has no X to duplicate |
| Key appears ignored | Unmapped keys are ignored; diagnostic mode shows reported events |
| Export/import cannot find a path | Paths use current working directory; quote paths containing spaces |

TUI viewport rendering is bounded to 400 columns and 120 rows and assumes
single-column glyphs. Entry editors are limited to 4096 characters; shortcut
expressions to 4096 bytes. General persisted text/script files are bounded to
1 MiB; register files to 16 MiB and 10000 registers. The TUI requires a Windows
console with VT output. Terminal state
is restored on ordinary exit/errors/unwinding; forced process termination
cannot run cleanup. See the TUI guide for resizing and recovery controls.
