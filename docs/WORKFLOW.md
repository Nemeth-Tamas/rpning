# Daily terminal workflow

Run `rpn` for the Windows TUI (`rpn --repl` for the ordinary REPL),
`rpn --eval "3 4 +"` for one expression, or
`rpn --run FILE` for one atomic RPN source file. `:help` lists commands.
RPN scripts contain the macro language, not application commands.
See the [TUI guide](TUI.md) and [full program guide](PROGRAM_GUIDE.md) for the
complete user reference, layer tables and embedded help topics. Linux/macOS
continue to default to the ordinary REPL.

## History and completion

On Windows the ordinary REPL's focused console editor supports insertion, Left/Right,
Home/End, Backspace/Delete, Up/Down history, Ctrl+U to clear, Ctrl+R to search
backward using current input as a literal substring, and Tab completion.
Press Ctrl+R again for an earlier match. Esc clears the line; Ctrl+C cancels
input; Ctrl+D on an empty line exits. Input is limited to 4096 characters,
displayed through a horizontally scrolling 60-character window. UTF-16
surrogate pairs are supported; alignment assumes single-column characters.

One matching completion replaces the current word; ambiguous matches are
listed (up to 30). Completions include commands, built-ins, macros and registers.
`:complete PREFIX` and `:history [TEXT]` provide completion listing and literal
history search in any terminal. History retains 1000 entries, deduplicates
consecutive identical commands, restores at startup and saves on normal exit.
Explicit `:history-save`/`:history-load` also work with piped input. Failed
commands can appear in history; numeric undo contains successful state changes.
Linux/macOS currently use ordinary line input, with command-based history/search
and completion listing rather than the Windows interactive editing keys.

## Settings and display

`:mode exact|binary`, `:precision BITS`, `:round nearest|zero|up|down|away`
and `:display MODE` control calculation/presentation. `:settings` shows them.
Enter any of those four setting commands without an argument to show the current
value and allowed options without changing state. TUI CMD also displays hints
while typing and offers setting-argument Tab completion; F6 browses all choices.

| Display | Behavior |
| --- | --- |
| `auto` | Exact terminating decimal up to 128 places, otherwise exact fraction |
| `scientific` | Lossless scientific notation for terminating representations |
| `fraction` | Canonical exact integer/fraction |
| `binary`, `octal`, `hex` | Prefixed integer representation; fractions remain lossless |
| `fixed:N` | N decimal places (0..128), using the selected rounding direction |

Rounded fixed output is prefixed by `~`: 1/3 at fixed:2 and nearest rounding
is `~0.33`. Switching to fraction still shows the stored `1/3`. Formatting
uses exact integers/rationals, never f32/f64, and never changes stored values.

Interactive changes save automatically. `:settings-save`/`:settings-load`
explicitly save/load numeric settings, keypad bindings and command shortcuts.
Malformed settings are rejected before application. Settings are outside
numeric undo/redo. `--eval`, scripts and pipes start fresh; use explicit load
commands in piped sessions when saved state is wanted.

Files live beside registers: `%APPDATA%/rpning` on Windows,
`$XDG_DATA_HOME/rpning` or `~/.local/share/rpning` elsewhere. `RPNING_HOME`
overrides the directory. UTF-8 versioned files are settings.txt, history.txt,
macros.rpn and registers.txt. Settings/macros use unique same-directory
temporary files and atomic replacement. Registers use explicit `:save`/`:load`.
History saves on normal exit; abrupt termination can lose current-session
history. Macro definition undo/redo is also reflected in automatic persistence.

## Command shortcuts and keypad macros

```text
: price.gross "27 percent VAT" 1.27 * ;
:shortcut vat price.gross
10000
:do vat
:bind nav home price.gross
:bind alt 9 @shortcut:vat
:settings-save
```

`:shortcut NAME EXPRESSION` defines a named expression, `:do NAME` executes it,
and `:shortcuts` lists it. Bindings call macros directly or `@shortcut:NAME`.
`:bind LAYER KEY ACTION_OR_EXPRESSION`, `:unbind LAYER KEY` and `:bindings`
manage mappings. Expressions can use arithmetic/control flow, but cannot
define macros. Macro references resolve at execution time.

Layers: base for normal numeric input, alt (F2 toggles base/alt), nav selected
automatically for navigation events, or any custom named layer. Reported
NumLock flags are not used. `@layer:NAME` selects a character-key layer;
navigation events still use nav. Esc/Ctrl+C always exit keypad mode. Other
actions: `@enter`, `@undo`, `@redo`, `@sign`, `@cancel`, `@clear`, `@swap`,
`@backspace`, `@quit`. Unknown/unbound keys retain ordinary keypad behavior.

Key names are lower-case letters/digits, decimal, add, subtract, multiply,
divide, enter, backspace, tab, home, up, pageup, left, right, end, down,
pagedown, insert and delete. Pending number plus expression is one atomic
transaction; errors preserve entry, stack, registers and redo history.

| Physical key | Default nav action | Default alt digit action |
| --- | --- | --- |
| 7 / Home | dup | square |
| 8 / Up | swap | sqrt |
| 9 / PageUp | drop | reciprocal |
| 4 / Left | negate | swap |
| 5 | No event observed | dup |
| 6 / Right | sqrt | drop |
| 1 / End | undo | undo |
| 2 / Down | redo | redo |
| 3 / PageDown | clear stack (undoable) | clear stack (undoable) |
| 0 or 00 / Insert | push depth | toggle sign |
| Decimal / Delete | cancel entry | push pi |

The observed 00 key emits two ordinary zero presses and has no distinct
identity. In alt mode its default sign action applies twice, restoring the
original sign. Avoid assigning destructive single-zero shortcuts if using 00.
