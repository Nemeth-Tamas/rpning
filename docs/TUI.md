# rpning TUI user guide

This guide covers operating the calculator screen and all default keypad
layers. The attached [full program guide](PROGRAM_GUIDE.md) is the segmented
reference for every implemented number format, mathematical word, setting,
register operation, macro feature, script command and persistence rule.

Both guides are also built into the TUI: CMD `:help tui` opens this guide;
`:help program` opens the full reference. `:help layers`, `:help macros` and
other topics jump to the relevant section. F6 focuses the output for scrolling.

## Launch and first calculation

Download [rpning v0.2.1](https://github.com/Nemeth-Tamas/rpning/releases/tag/v0.2.1)
for a ready-to-run binary, or build from source below. The v0.2.1 Windows
binary includes the TUI, extended mathematics and both embedded guides.

```powershell
git pull --ff-only
cargo run --release
```

On Windows, plain `rpn` selects the TUI when input and output are both terminals.
`rpn --tui` explicitly requests it. `rpn --repl` selects the ordinary line
editor. Pipes, `--eval` and `--run` keep their existing behavior; Linux/macOS
still default to the ordinary REPL.

Start in NUM input and Base layer. Press `12`, Enter, `3`, `+`, `2`, `*`.
The result is 30. Enter commits a typed number; an operator commits pending
input and executes immediately. Enter with no pending input duplicates X.
Press F5 for CMD, type `5 2 pow`, then Enter: X becomes 25.

## Screen and focus

The upper pane always contains the committed result X and the input editor.
A red error/status line is separate from the result. Arithmetic errors leave
the last committed result and the pending edit intact.

The lower area normally shows the stack and newest-first calculation journal.
At 90 rendered columns or wider they appear side by side; narrower screens
show one at a time. F4 selects the stack/history view. Result labels X/Y/Z
mean first/second/third stack value, with zero-based indices thereafter.

Help, settings queries, completion lists, macro inspection, bindings and
command-history search use a full-width **Output** view in the lower area.
F6 focuses Output for browsing; F4 returns to stack/history. Input focus can
remain active while output is visible, so you can type the next command.
A new calculation returns the lower area to stack/history.

Focus controls what navigation keys do. The header names the current focus;
the footer always shows the current character layer, numeric mode, precision
and rounding. Press F1 to return to input. F5 also focuses input.

| Key | Action |
| --- | --- |
| F1 | Focus input |
| F2 | Toggle Base/Alt; from a custom layer return to Base |
| F3 | Cycle Input -> Stack -> History -> Result -> Output -> Input |
| F4 | Switch/focus the lower stack/history pane and close Output |
| F5 | Toggle NUM/CMD input and focus input |
| F6 | Open/focus the last command output |
| Ctrl+Z / Ctrl+Y | Undo / redo; pending NUM edit is canceled before numeric undo |
| Esc / Ctrl+C | Exit and restore the terminal |
| Arrows, PageUp/PageDown | Browse a focused pane; scroll Result horizontally |
| Home / End | First/last position in a browsing pane |
| Enter with History focused | Recall selected source into CMD without executing |

Input and browsing are separate: pressing a digit/operator while a browsing
pane has focus does not calculate. Press F1 before continuing arithmetic.
F2 is a persistent layer toggle, not the physical Alt modifier key.

## NUM input

NUM is the immediate calculator interface. It accepts decimal digits and a
dot or comma decimal key. Both separators create a dot in the pending number.
Repeated decimal presses do not add a second dot. A leading decimal becomes
`0.`; a trailing dot is accepted when committing.

| Base key | Behavior |
| --- | --- |
| 0..9 | Append digit to pending number |
| Decimal dot/comma | Append decimal point |
| +, -, *, / | Commit pending number and perform Y OP X immediately |
| Enter | Commit pending number, or duplicate X when entry is empty |
| Backspace | Delete last pending character |
| Delete | Cancel pending entry (default nav-layer mapping) |
| S | Toggle pending sign; with no entry negate X |
| C | Clear stack and pending entry; stack clear is undoable |
| Tab | Commit pending number then swap X/Y |
| Ctrl+Z | Cancel pending entry first, otherwise undo |
| Ctrl+Y | Redo |
| Esc / Ctrl+C | Exit |

For negative NUM input use S before/after digits; the minus key is subtraction.
Use CMD for fractions, prefixed base literals, named mathematical words and
application commands. Unmapped keys do nothing. Pending input is limited to
4096 characters.

A failing operation preserves the pending edit and calculator state. For
example, after X=30, typing `0 /` reports division by zero. Backspace removes
that zero; type `2 /` to obtain 15. Delete cancels the pending number instead.

F5 preserves pending NUM input when changing editors. A CMD RPN expression
prepends and commits it in the same transaction. Application commands such as
settings, `:do`, tracing and register/macro inspection operate on the committed
calculator state and leave the pending number untouched. Commit/cancel pending
input before mixing those workflows if you do not want that distinction.

## CMD input and discoverable settings

CMD accepts an entire RPN expression, macro definition or application command.
Enter submits; empty Enter does nothing. Successful input clears the command;
failed input remains editable. Left/Right/Home/End move the cursor;
Backspace/Delete edit; ordinary printable text inserts at the cursor.
Input is limited to 4096 characters and scrolls horizontally.

Tab completes the current word. Completions include built-ins, commands, macros,
register references and setting values. One match completes it; ambiguous
matches appear in Output without executing anything. F6 then PageDown/End
browses every match; F1 returns to editing. File paths are not completed.

For `:display`, `:mode`, `:precision` and `:round`:

1. Type the command to show its current value and allowed-value hints below input.
2. Press Enter without an argument for a read-only query in Output.
3. Or press Tab to list allowed arguments, then type more to narrow the list.
4. Enter an argument to change the setting.

Examples: `:display h` + Tab completes `hex`; `:display fixed:` + Tab lists
`fixed:0` through `fixed:128`. Precision completion suggests common bit counts;
any integer from 2 through 4096 is accepted. See
[Settings and display](PROGRAM_GUIDE.md#settings-and-display) for every value.

```text
:display
:display fraction
:precision 128
:round nearest
:mode exact
:settings
```

Settings queries do not change stack, registers, macros, pending entry or
numeric settings. They can appear in command history/journal. `:display binary`
is integer presentation; `:mode binary` is significant-bit arithmetic rounding.
They are independent.

## Stack, history and Output

F3 selects Stack; arrows/pages scroll values. X/Y/Z and indices always describe
their position in the actual stack, even after scrolling. F3 selects History;
its first displayed entry is selected. Enter copies that source into CMD and
cancels pending NUM input. It does not execute or restore an earlier stack.
Press Enter again to run it against the current state.

The journal retains at most 256 actions/errors for the current session. It
records committed calculations, macros and undo/redo, rather than every digit
edit. This is separate from persisted command history and numeric undo.

`:history` focuses the journal; `:history TEXT` searches saved command history
by literal substring in Output. `:stack` focuses Stack. `:registers`,
`:bindings`, `:shortcuts`, `:macros`, `:see NAME`, `:trace EXPRESSION` and
`:help TOPIC` also use Output. Long output lines wrap to the terminal width.
F6 focuses it; arrows/pages/Home/End browse; F4 returns to stack/history.

Output retains the last list/guide while you calculate, so F6 can reopen it.
Tracing and `:do NAME` execute calculations; inspection/help do not. Undo/redo
affects calculator snapshots, not files, displayed help or saved command history.

## Keypad layers

Read this section inside the TUI with `:help layers`. Use F6 to scroll it.
The footer's `layer=base` / `layer=alt` / custom name identifies the selected
character layer. F2 toggles Base/Alt; from any non-Base layer it returns to Base.
The physical Alt modifier is not needed and does not select this layer.

**Base** provides normal numeric entry. **Alt** replaces the digit and decimal
keys with quick calculations. Unbound keys in either layer keep their ordinary
Base fallback. **Base** uses the **Nav** bindings for NumLock-off numpad events.
**Alt and custom layers follow physical numpad positions with NumLock ON or
OFF**, using measured virtual-key/scan-code pairs. Thus physical 7 remains
Alt-square and physical 6 remains Alt-drop, even when they emit Home/Right.
Dedicated enhanced navigation keys still use Nav. NUM input must have focus;
browsing and CMD editing retain their own navigation controls. Use NumLock ON
for ordinary Base number entry; routing does not rely on the NumLock flag alone.

### Complete default Base and Alt mappings

**v0.2.0 Alt/NumLock OFF uses the last column; v0.2.1 Alt uses the third in both states.**

| Physical key/event | Base, NumLock ON | Alt: v0.2.0 ON / v0.2.1 ON or OFF | v0.2.0 Alt, NumLock OFF: actual action |
| --- | --- | --- | --- |
| 7 | Append 7 | Square X: `dup *` | Duplicate X: `dup` |
| 8 | Append 8 | Exact square root: `sqrt` | Swap X/Y: `swap` |
| 9 | Append 9 | Reciprocal: `1 swap /` | Drop X: `drop` |
| 4 | Append 4 | Swap X/Y: `swap` | Negate X: `neg` |
| 5 | Append 5 | Duplicate X: `dup` | Ignored (Clear event, no mapping) |
| 6 | Append 6 | Drop X: `drop` | Exact square root: `sqrt` |
| 1 | Append 1 | Undo: `@undo` | Undo: `@undo` |
| 2 | Append 2 | Redo: `@redo` | Redo: `@redo` |
| 3 | Append 3 | Clear stack: `clear` | Clear stack: `clear` |
| 0 | Append 0 | Toggle sign: `@sign` | Push stack depth: `depth` |
| 00 | Append two zeros | Toggle sign twice | Push stack depth twice |
| Decimal . or , | Decimal entry | Push approximate pi: `pi` | Cancel pending entry: `@cancel` |
| + / - / * / / | Immediate arithmetic | Same Base fallback | Same Base fallback |
| Enter | Commit entry / duplicate X | Same Base fallback | Same Base fallback |
| Backspace | Edit entry | Same Base fallback | Same Base fallback |
| S | Pending sign / negate X | Same Base fallback | Same Base fallback |
| C | Clear stack and pending input | Same Base fallback | Same Base fallback |
| Tab | Commit pending entry and swap X/Y | Same Base fallback | Same Base fallback |
| Dedicated Delete | Nav `@cancel` | Nav `@cancel` | Nav `@cancel` |
| F1..F6, Ctrl+Z/Y, Esc/Ctrl+C | Global TUI controls | Same global controls | Same global controls |
**Read the column matching your EXE version and NumLock state.** In v0.2.0,
NumLock-off numpad events select Nav even while the footer says Alt. Thus the
configured `alt.7 = dup *` binding does NOT describe what physical 7 actually
does in that state. v0.2.1 fixes that routing. Check your executable using
`rpn.exe --version` from the terminal; pulling source does not update an older
downloaded EXE.

These are effective actions with default saved bindings. `:bindings` lists
configuration records; it is not proof that v0.2.0 routes an event to that layer.
In Base with NumLock OFF, use the Nav table below.

To cancel pending input in Alt with NumLock OFF, use Ctrl+Z or the dedicated
Delete key. The physical numpad decimal/Delete position remains the Alt pi key.

Bound expressions commit pending input first. For example, enter 9 in Base,
then F2 and Alt-8: sqrt operates on 9. Without pending entry Alt-8 uses X.
Alt-Decimal commits pending input (if any), then pushes pi on top. Pi requires
precision at most 512 bits. Square-root irrational/domain errors preserve state.

The dedicated 00 key reports two ordinary zeros, not a distinct key. In Base
it appends two zeros. In Alt it toggles sign twice, restoring the original sign.
Avoid expecting a separate `00` binding.

### Complete default Nav mappings

| Emitted event / usual numpad position | Default action |
| --- | --- |
| Home / 7 | `dup` |
| Up / 8 | `swap` |
| PageUp / 9 | `drop` |
| Left / 4 | `neg` |
| Center / 5 | Clear event emitted; no Base/Nav action assigned |
| Right / 6 | `sqrt` |
| End / 1 | `@undo` |
| Down / 2 | `@redo` |
| PageDown / 3 | `clear` |
| Insert / 0 or 00 | `depth` |
| Delete / decimal | `@cancel` |

When Stack/History/Result/Output has focus, navigation keys browse instead of
running Nav calculations. In CMD, arrows/Home/End/Delete edit instead of calling
bindings. Layer bindings apply to NUM input. Global F1..F6, Ctrl+Z/Y and
Esc/Ctrl+C controls take priority and cannot be overridden by a mapping.

### Inspect and customize layers

`:bindings` lists every configured mapping, including defaults for Alt/Nav and
your overrides. Built-in Base fallbacks are not stored mappings; use the table
above to see them. `:shortcuts` lists named shortcuts. Output is scrollable.

```text
: price.gross "Add 27 percent" 1.27 * ;
:shortcut vat price.gross
:bind alt 9 @shortcut:vat
:bindings
```

After F5 returns to NUM, F2 selects Alt, then 9 executes the shortcut. With
pending 100 it commits 100 and produces 127. With an empty entry it uses X.
Changes save automatically; `:settings-save` explicitly saves them too.

Use `:unbind alt 9` to remove a mapping. That reveals ordinary digit-9 behavior,
rather than restoring reciprocal automatically. To restore the original:
`:bind alt 9 1 swap /`. Never assume unbind disables a key.

Key names: lowercase letters/digits, `decimal`, `add`, `subtract`, `multiply`,
`divide`, `enter`, `backspace`, `tab`, `home`, `up`, `pageup`, `left`,
`right`, `end`, `down`, `pagedown`, `insert`, `delete`.
Custom layers can use `@layer:NAME` actions:

```text
:bind base l @layer:finance
:bind finance 7 1.27 *
:bind finance l @layer:base
```

Press L in NUM/Base to select finance; its 7 applies 1.27 times X/pending entry.
Press L or F2 to return. Physical numpad positions use the custom layer in both
NumLock states; dedicated navigation keys still select Nav. Supported actions:
`@enter`, `@undo`, `@redo`, `@sign`, `@cancel`, `@clear`, `@swap`,
`@backspace`, `@quit`, `@layer:NAME`, `@shortcut:NAME`. Direct expressions can
call compiled macros but cannot define a macro as a binding/shortcut body.

## Full program reference and examples

The [full program guide](PROGRAM_GUIDE.md) contains separate sections for:

- RPN number formats, operand order, stack operations and transactions.
- Every setting/display value and numeric precision/rounding semantics.
- Arithmetic, roots, logs/trig, bases/bits, statistics and probability.
- Named registers, explicit saves/loads and shared versus local variables.
- Macro definitions, aliases, namespaces, inspection, tracing, conditions,
  loops, recursion, local variables and library import/export.
- Scripts, pipes, one-shot invocation, shortcuts, persistence and limits.
- A complete application command index and troubleshooting table.

For a macro calculation in CMD:

```text
: math.square "Square X" dup * ;
5 math.square
:see math.square
```

X is 25. Inspection opens Output without changing X. F6 browses the instructions;
F1 returns to input. `:help macros` opens the full macro chapter, including
advanced control flow. `:help math` opens the mathematical reference.

## Saving, limits and recovery

Interactive settings/mappings/macros save after successful changes. Command
history saves on normal exit. Registers require explicit `:save` and `:load`.
The numeric stack and calculation journal are session-only. See the full
guide's [persistence chapter](PROGRAM_GUIDE.md#persistence-and-files) for paths,
profiles and file formats.

Long results are clipped only for display; stored values remain unchanged.
F3 selects Result, then Left/Right or Home/End scrolls horizontally.
Output wraps long lines; F6 then scrolling reveals the entire text. The screen
adapts to resize events and viewport polling. Allow at least 33 terminal columns
and 13 rows; smaller screens show a recovery message without discarding input.
Rendering is bounded to 400 columns and 120 rows and assumes single-column glyphs.

Esc/Ctrl+C exit and restore console input/output modes, original screen,
cursor and text attributes. Normal errors/unwinding also run cleanup; forced
process termination cannot do so. Linux/macOS TUI support remains deferred.

The test suite covers calculator transactions, setting queries/completion,
Output scrolling, embedded guides, long-value clipping, responsive layouts,
macro integration and history recall. The isolated Windows-console smoke test
also checks live frames, default-TUI launch, explicit REPL, errors, settings
hints/queries, help/layers, completion, pane navigation and terminal restoration.
