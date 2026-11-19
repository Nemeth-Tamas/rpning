# Macro and script language

Definitions use `: NAME "optional description" BODY ;`. Names are case
sensitive ASCII identifiers, optionally containing dots for namespaces.
Built-in words cannot be shadowed. Definitions compile once to instructions;
calls resolve at execution time, supporting forward references and recursion.

```text
: math.square "Square X" dup * ;
: math.fourth math.square math.square ;
: price.gross "Hungarian 27 percent VAT" 1.27 * ;
10000 price.gross
```

The result is exactly `12700`. `:alias sq math.square` defines an ordinary
compiled wrapper. `:macros` lists names/descriptions, `:see math.square` shows
source and instructions, and `:trace 5 math.square` traces instruction execution
(up to 4096 trace lines, each at most 240 characters).

## Control flow and local variables

Comparisons `= != < <= > >=` consume Y/X and push 1 or 0. `not` replaces X
with 1 if zero, otherwise 0. `neg` negates X; `abs` makes it nonnegative.

```text
: absolute dup 0 < if neg then ;
: factorial dup 1 <= if drop 1 else dup 1- factorial * then ;
8 factorial
0 5 times 2+ end
0 while dup 10 < do 1+ end
: rectangle let:width get:width * ;
4 7 rectangle
```

`if ... [else ...] then` consumes a flag; zero means false. `N times ... end`
consumes a nonnegative integer count. `while CONDITION do BODY end` reruns the
condition and consumes its flag before each iteration. Nesting is supported.
`let:name` consumes X into a variable; `get:name` pushes its value. Variables
belong to the current call and do not leak to other or recursive calls. Use
existing `sto:name`/`rcl:name` for shared named registers.

One input evaluation is atomic: failed calls, loops or definitions restore
stack, registers and macro definitions, preserve redo, and create no history
entry. A successful state-changing expression is one undo step regardless of
the number of instructions or nested calls. Definitions themselves are undoable.
Settings remain outside numeric undo/redo.

Limits: 1 MiB source, 100000 source tokens, 64 nested parser blocks, call depth
64, 100000 executed instructions per evaluation, 10000 defined macros. The VM
also bounds explicit pushes to 100000 stack entries. Individual arithmetic
words retain their own domain/size limits. These bounds control execution, not
wall-clock time for arbitrary-precision arithmetic. No shell or OS operations
are exposed as macro words.

## Persistence and scripts

Interactive sessions restore `macros.rpn` automatically and save definitions
after successful expressions/macro commands. `:macros-save` and `:macros-load`
explicitly save/load in interactive or piped sessions. Default directory is
the same as the register file (`%APPDATA%/rpning` on Windows). Set
`RPNING_HOME` to use an isolated directory for all application state.

`:export PATH` writes the macro library, and `:import PATH` merges definitions.
Paths may contain spaces and may be surrounded by double quotes. Import is
atomic and accepts definitions/comments only; it does not execute top-level
calculations. Exports are UTF-8 source and can be reviewed or edited manually.
Descriptions support escaped quotes, backslashes, newlines and tabs.

```powershell
cargo run -- --eval ': square dup * ; 9 square'
cargo run -- --run examples/invoice.rpn
```

`--run`/`-f` executes a complete UTF-8 RPN source file as one transaction,
prints X, and returns nonzero on error. Macro definitions may span lines;
`#` starts a comment until newline. Script files contain RPN language, not
interactive colon commands such as `:precision`. Piped input still evaluates
one complete expression/definition per line. `--eval`, scripts and pipes start
fresh unless explicit load/import commands are used in the piped session.
