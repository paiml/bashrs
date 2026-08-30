# CRUX: how the field solves the six defects bashrs has

**Lane:** CRUX (competitive research). This document writes no fix. It establishes what
ShellCheck, mvdan/sh and tree-sitter-bash actually *do* about each defect class, so the
fixing lanes have a reference implementation to be judged against instead of inventing one.

**Date:** 2026-08-30 · **bashrs measured:** 6.67.0 · **References:** ShellCheck 0.8.0
(binary) + `master` source, mvdan/sh `shfmt` v3.14.0, tree-sitter-bash `master` grammar.

---

## 1. The headline measurement

Corpus: `/home/noah/src/rmedia/scripts/*.sh` — 45 scripts of ordinary production shell.

| Tool | Result on the corpus |
|---|---|
| `bash -n` | **0** syntax errors, 45/45 |
| `shellcheck -S error` | **0** findings, 45/45 |
| `mvdan/sh` (shfmt v3.14.0, `-ln bash`) | **0** parse failures, 45/45 |
| **bashrs 6.67.0** | **150 errors** across 45 files |

Reproduce:

```sh
cd /home/noah/src/rmedia
tot=0; for f in scripts/*.sh; do n=$(bashrs lint "$f" 2>/dev/null | grep -cE '\[error\]'); tot=$((tot+n)); done; echo $tot
shellcheck -S error -f gcc scripts/*.sh | wc -l                 # 0
for f in scripts/*.sh; do shfmt -ln bash "$f" >/dev/null || echo "FAIL $f"; done   # silent
```

> A methodological note, because it nearly produced a false claim in this very document:
> `shfmt -d` exits non-zero for a *formatting diff*, not a parse error. Measured with `-d`,
> mvdan/sh appears to "fail" 40/45 files. The correct probe is plain `shfmt -ln bash f >/dev/null`,
> which exits non-zero only on a parse error. That is 0/45. Any future comparison must use
> the second form.

Of the 150, **121 are false positives** and 29 are real (18 SEC011, 7 DET002, 4 SEC010).
The cost is not the noise. SEC011 found a genuine `rm -rf "$src_raw"` hazard in this corpus
where source and destination could resolve to one directory through a symlink. A 5:1
false-positive ratio is what buries that finding.

---

## 2. The single root cause, stated once

Every one of the six defect classes is the same architectural decision:

> **bashrs's lint rules run on raw source text, each rule re-implementing its own partial
> shell lexer. The field runs every rule against one shared parse tree.**

Measured in `rash/src/linter/rules/` (496 non-test rule files):

| Property | Count |
|---|---|
| Rules whose entry point is `pub fn check(source: &str)` | **431 / 496** |
| Rules that iterate `source.lines()` | **428 / 496** |
| Rules built on `regex` | **303 / 496** |
| Rules that hand-roll their own single/double quote state | **27** |
| Rules that hand-roll their own heredoc tracking | **22** |

A `bash_parser/` module with a real lexer, parser and AST exists in the same crate. The
linter calls it **twice**, and both are comments explaining that it doesn't:

```rust
// rash/src/linter/rules/mod_lint.rs:68
// In production, this would use the bash_parser AST
```

```rust
// rash/src/linter/shell_words.rs:30
//! It deliberately does *not* reuse `bash_parser::lexer`: that lexer collapses
//! `'x'` and `"x"` into the same token (erasing the single-vs-double distinction
//! this analysis is built on) and returns `Err` on unterminated strings.
```

The second comment is the important one, and it is a *correct* diagnosis of a real problem
— the transpiler's lexer is lossy and fails closed, both wrong for a linter. But the
conclusion drawn from it ("so each rule will re-lex") is the one that produced 121 false
positives. The field's conclusion from the same premise is different, and is covered in §9.

**Consequence: quoting correctness is per-rule, not global.** 27 rules know what a quote is;
469 do not. 22 rules know what a heredoc is; 474 do not. Whether a given rule false-positives
on the corpus is therefore a function of which rule you ask, not of what the shell means —
which is exactly what the histogram shows.

---

## 3. T1 + T4 — lexer state: multi-line strings and heredoc bodies

### The defects

```sh
# T4 — SC1078 ×73. A multi-line double-quoted string, reported unterminated.
STATS=$(python3 -c "
import json, sys
print('ok')
")

# T1 — SC2188 ×4. A heredoc body whose lines start with '<' read as redirections.
cat > out.svg << FEOF
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1920 1080">
  <rect width="1920" height="1080" fill="#0f1114"/>
</svg>
FEOF

# T1 — SC1128 ×1. A shebang inside a heredoc body read as a shebang.
cat > cmd.sh <<EOF
#!/usr/bin/env bash
echo x
EOF
```

All three are valid bash. `bash -n` ok, `shellcheck` silent, `mvdan/sh` parses.

### (a) What the field does

**ShellCheck — heredocs are a deferred read attached to the newline token.** The parser is
Parsec-based with a user state carrying `pendingHereDocs :: [HereDocContext]`. At the `<<`
operator it emits an *empty* `T_HereDoc` node and queues the pending read
(`src/ShellCheck/Parser.hs:1848`):

```haskell
    -- add empty tokens for now, read the rest in readPendingHereDocs
    let doc = T_HereDoc hid dashed quoted endToken []
    addPendingHereDoc hid dashed quoted endToken
```

and the *newline primitive itself* drains the queue (`Parser.hs:61-65`):

```haskell
linefeed :: Monad m => SCParser m Char
linefeed = do
    ...
    readPendingHereDocs
```

There is exactly one place in the program that knows a newline happened, and it is the same
place that knows a heredoc is pending. That is the whole mechanism. A rule cannot forget to
handle heredocs, because no rule ever sees the body as source lines — `readPendingHereDocs`
attaches it to the redirect's node via `addToHereDocMap`.

The delimiter's own quoting decides whether the body is code or data:

```haskell
        list <- parseHereData quoted (docStartPos, docEndPos) hereData
```

`<<'EOF'` → literal data. `<<EOF` → re-parsed for expansions. ShellCheck tests this
explicitly, including the exact shape bashrs fails — a heredoc delimited by `#!`:

```haskell
prop_readHereDoc13 = isOk readScript "cat <<'#!'\nHello World\n#!\necho Done"
```

**ShellCheck — a double-quoted string is a node, not a line region.** `readDoubleQuoted`
returns `T_DoubleQuoted id x` spanning whatever it spans; newlines inside are ordinary
literal content. The parser has no concept of "this rule looks at one line".

**mvdan/sh — the same answer in a Go AST.** `syntax.DblQuoted` carries `Left, Right Pos`
and `Parts []WordPart`; a newline inside is just a `Lit` part. Heredoc bodies hang off
`Redirect` (operators `Hdoc` `<<`, `DashHdoc` `<<-`, `WordHdoc` `<<<`), never appearing in
the statement stream. Every node exposes `Pos()`/`End()` with offset+line+column, so a
multi-line construct has one span rather than N line-fragments.

**tree-sitter-bash — heredocs need an external scanner, by design.** `heredoc_start`,
`heredoc_content` and `heredoc_end` are declared `externals` and handled by a hand-written
C scanner, because — in the grammar's own framing — heredoc syntax requires state tracking
across newlines that a context-free rule cannot express.

Three independent implementations, three different languages, one conclusion: **heredoc
bodies and multi-line strings cannot be handled by line-local logic, so none of them tries.**

### (b) What bashrs does today

`sc1078.rs` is the *good* case and still shows the problem. It is a whole-source scan, it
tracks single- and double-quote state, and it skips heredoc bodies — its module docs
describe fixing precisely this bug once already. It still produced 73 findings, because a
per-rule scanner has to re-derive all of shell's quoting from scratch and this one still
misses cases (`$(...)`-nested quotes, `$'...'`, backslash-continuations across the `python3 -c "` boundary).

`sc2188.rs` is the bad case: it has **no heredoc handling at all** — the entire rule is
`Regex::new(r"^\s*[<>]")` plus continuation folding.

Its test suite claims otherwise, and this is worth calling out on its own:

```rust
#[test]
fn heredoc_still_exempt_under_continuation_folding() {
    let code = "cat <<EOF\nhello\nEOF\n";
    assert_eq!(check(code).diagnostics.len(), 0);
}
```

**This test is vacuous.** The body line is `hello`, which `^\s*[<>]` could never match. The
test passes whether or not heredoc handling exists — and it does not exist. Falsified
directly:

```console
$ printf 'cat <<EOF\n> not a redirect, just data\nEOF\n' > hd.sh
$ bashrs lint hd.sh
✗ 2:1-28 [error] SC2188: Redirection without command
$ shellcheck hd.sh
(silent)
```

A test named for the property it does not test is the fleet's signature defect shape, and it
is why this rule shipped at `Severity::Error`.

### (c) Does the fix match the field?

**Only if the fix is shared, not per-rule.** Making `sc2188` skip heredocs the way `sc1078`
does would fix the 4 findings and leave the other 474 line-oriented rules exposed to the
next heredoc containing something that looks like syntax. That is not what any of the three
references do, and it is the change most likely to be proposed because it is the smallest.

The defensible minimum that still matches the field: **one shared pre-pass that produces, for
the whole source, the byte ranges that are heredoc bodies and the byte ranges that are inside
quotes, computed once and consumed by every rule.** bashrs already has the shape of this in
`linter::quoting::mask_literals` (used by `mod_lint.rs` for the SC1xxx batch) — it is
under-used, not absent.

### (d) Defensible vs. mistake

- **Mistake:** 22 independent heredoc trackers and 27 independent quote trackers. There is no
  reading of shell where those are 49 different questions. This is the root cause of five of
  the six classes and it should not be fixed one rule at a time.
- **Defensible:** *not* adopting a full Parsec-style parser. bashrs's linter has a hard
  requirement ShellCheck does not: it must produce findings on input that does not parse, and
  it must keep byte-accurate columns for `--fix` autofix splicing. ShellCheck's answer to
  unparseable input is to bail with SC1072/SC1073 and stop checking. A recovering,
  error-tolerant token stream that never returns `Err` is a legitimate and *better* design
  for a linter — see §9.
- **Mistake, specifically:** shipping `sc2188` at `Severity::Error` behind a test that could
  not fail. Every rule that claims heredoc- or quote-awareness needs a test whose fixture
  actually contains the trap (`> data` inside a heredoc, `#!` as a delimiter), and the fix
  lane must prove it discriminates by reverting the fix and watching it go red.

---

## 4. T2 — `[ ]` scanning: parens and operators that are not in the test

### The defects

```sh
# SC1028 ×33 — the parens are in a STRING, in a different command, after the test closed.
[ "$rc" = 0 ] && echo "lint: OK ($checked spec(s) checked)"

# SC2104 ×6 — the "]" is inside a quoted grep pattern.
if [ "$rc" = 0 ] && grep -q "1 #\[contract\] annotations" <<<"$out"; then

# SC2122 ×1 — the ">=" is Python, two levels of command substitution deep.
elif [ "$(python3 -c "print(int($cov >= 85))")" != "1" ]; then
```

### (a) What the field does

ShellCheck parses `[` as a *command* whose arguments are read by a dedicated
`readCondition` grammar (`TC_And`, `TC_Or`, `TC_Unary`, `TC_Binary`, …). The condition
parser stops at the matching `]` because that is where the grammar says the command ends.
Everything after it belongs to a different node, and everything inside a `T_DoubleQuoted`
part is a string literal. A rule about `[ ]` receives condition nodes; it is structurally
incapable of seeing a paren in a neighbouring `echo`.

mvdan/sh is the same: `TestClause`/`TestExpr` for `[[ ]]`, and `[` as a `CallExpr` whose
`Args` are `Word`s. A `Lit` inside a `DblQuoted` is not an operator anywhere.

The general principle: **the field never asks "does this line contain X?" It asks "does this
node contain X?"** Line containment is not a shell concept.

### (b) What bashrs does today

Recently improved and still text-based. `sc1028.rs` HEAD (`da7442c3d1`, "SC1028 flagged
every paren on a line that merely contained a test") now bounds the scan to byte ranges
between a `[` and its `]`, and carries a `Quotes` struct fed one byte at a time. Its own
module docs are candid about the history:

> SC1028 used to ask two independent questions — "does this line contain a `[ ` anywhere?"
> and "does this line contain a bare paren anywhere?" — and report the second whenever the
> first was true. […] On a 1200-line script of ordinary shell, 148 of 148 SC1028 findings
> were parens that live somewhere a test cannot reach.

The residual 33 findings are the cases a byte scanner still gets wrong: the closing `]`
inside `"…#\[contract\]…"` terminates the range early, and nested `$( … "…" … )` defeats a
flat two-flag quote model.

### (c) Does the fix match the field?

**Directionally yes, structurally no.** Bounding the scan to the test's own range is the
right *idea* — it is what a parser gives you for free. Implementing it as a hand-written
byte scanner reproduces the parser's job at 1/50th of its fidelity, and the remaining 33
findings are the measure of the gap. `sc1028.rs` is 493 lines and still wrong; ShellCheck's
`readCondition` is correct because it is a grammar, not a scanner.

### (d) Defensible vs. mistake

- **Defensible:** the bounded-range approach as an interim step. It converted a 148/148
  wrong rule into a mostly-right one and the module documents why.
- **Mistake:** believing byte-scanning converges. Three widenings of this rule are recorded
  in its own comments. The fourth will be the `#\[contract\]` case, the fifth the nested
  `$( )` case, and each new scanner is a new place to get quoting wrong. This rule is the
  clearest argument in the codebase for a shared tokenizer.
- **Note for the fix lane:** SC2122 (`>=` in `[ ]`) fires on Python source inside
  `$(python3 -c "…")`. Any fix that only tracks quote *depth* without tracking command
  substitution *nesting* will still be wrong here.

---

## 5. T3 — word boundaries: `git for-each-ref` is not a `for` loop

### The defect

```sh
heads=$(git for-each-ref --format='%(refname:short)' refs/heads)   # SC1035 ×4
```

`for-each-ref` matched because it starts with `for`.

### (a) What the field does

ShellCheck recognises a reserved word only when a **keyword separator** follows it, and
backtracks completely otherwise. One line is the whole answer (`Parser.hs:3181`):

```haskell
keywordSeparator =
    eof <|> void (try allspacingOrFail) <|> void (oneOf ";()[<>&|")
```

used by the shared keyword primitive (`Parser.hs:3104-3125`):

```haskell
tryParseWordToken keyword t = try $ do
    str <- anycaseString keyword
    ...
    lookAhead keywordSeparator
```

with every keyword defined through it — `g_For = tryWordToken "for" T_For`, `g_Do`, `g_Done`,
`g_While`, `g_Case`, … Because the whole thing is inside `try`, failing the `lookAhead` rewinds
the input and `for-each-ref` is re-read as an ordinary word. `-` is not in the separator set,
so the keyword never matches. This is *reserved-word position* semantics, which is what POSIX
actually specifies, rather than prefix matching.

Note the separator set **includes `(`**: `for((i=0;i<10;i++))` is a `for` keyword followed by
`((`, and ShellCheck parses it (`prop_readForClause4`). It also emits *distinct* codes for
distinct suspect followers rather than one blanket message — SC1069 for `[`, SC1099 for `#`,
SC1129 for `!`, SC1130 for `:`.

mvdan/sh does the same via a `Pos`-tracked lexer with a reserved-word table applied only in
command position. tree-sitter-bash relies on lexer precedence between the literal `'for'`
token and the generic `word` token, which is the same rule expressed as token priority.

### (b) What bashrs does today

`sc1035.rs` — a keyword list, scanned per line, with the boundary test on the wrong side:

```rust
const KEYWORDS: &[&str] = &[
    "then", "do", "else", "elif", "fi", "done", "while", "until", "for", "case", "esac", "in",
];
...
for (line_num, line) in source.lines().enumerate() {
    ...
    // Search for keyword followed immediately by a non-whitespace, non-semicolon char
    ...
    // Verify it's a word boundary before the keyword
```

The boundary is checked *before* the keyword only; after it, anything that is not whitespace
or `;` fires. So `-` fires, and so does `(`.

**This produces a second false positive the corpus does not contain but the fix lane must not
miss** — bashrs flags entirely valid arithmetic loops:

```console
$ printf '#!/bin/bash\nfor((i=0;i<3;i++)); do echo $i; done\n' > k.sh
$ bash -n k.sh && echo VALID        # VALID
$ shellcheck k.sh | grep -c SC1035  # 0
$ bashrs lint k.sh
✗ 2:1-4 [error] SC1035: Missing space after 'for' keyword

$ printf '#!/bin/bash\nwhile(true); do break; done\n' > k2.sh   # valid bash
$ bashrs lint k2.sh   # 2 errors
```

### (c) Does the fix match the field?

**Only if the fix adopts the separator set, not a space check.** The naive repair — "require
whitespace after the keyword" — kills the 4 corpus findings and *keeps* `for((`/`while(`
broken, because those legitimately have no space. The field's predicate is
`eof | whitespace | one of ";()[<>&|"`, and it is not a heuristic; it is the boundary POSIX
defines.

Applying it: `for-each-ref` → `-` not in set → not a keyword → silent (correct).
`for((` → `(` in set → keyword → silent (correct). `do{` → `{` not in set → …

### (d) Defensible vs. mistake

- **Mistake:** prefix matching on a keyword list. There is no defence; it is a
  known-wrong technique and the reference is a one-line character set.
- **Must still fire — verified.** `do{` is genuinely invalid bash and every tool rejects it:

  ```console
  $ printf '#!/bin/bash\nfor i in 1 2; do{ echo $i;}\ndone\n' > p.sh
  $ bash -n p.sh          # syntax error near unexpected token `do{'
  $ bashrs lint p.sh      # SC1035  ← must survive the fix
  $ shellcheck p.sh       # SC1058, SC1072, SC1073
  ```

  Under the separator set this still fires, because `{` is not a separator. A fix that
  silences it has traded a false positive for a false negative.
- **Defensible divergence:** bashrs reports one code (SC1035) where ShellCheck splits four
  (SC1069/1099/1129/1130) and otherwise degrades to a parse error. A single actionable
  message is arguably kinder than a three-line parse cascade — see §9.

---

## 6. T5 — `;`-separated lists

### The defect

```sh
# One line, four commands. SC1028 fires on the FUNCTION DEFINITION's parens
# because a `[ ]` appears later in the same line.
is_allowed(){ for a in "${allow[@]}"; do [ "$a" = "$1" ] && return 0; done; return 1; }
```

Reported at `18:11-12` and `18:12-13` — the `(` and `)` of `is_allowed()`.

### (a) What the field does

Nobody in the field treats a line as a unit of analysis. ShellCheck builds a tree:
`readTerm` → `readAndOr` → `T_AndIf` / `T_OrIf` / `T_Pipeline` / `T_Script`, with
`readSequentialSep = void (g_Semi >> readLineBreak) <|> void readNewlineList` — note that
`;` and a newline are the *same* production. A statement separator carries no more meaning
than a line break, which is why `a; b` and `a\nb` behave identically everywhere.

For dataflow specifically, ShellCheck goes further than a tree: `src/ShellCheck/CFG.hs`
(1319 lines) builds a real **control-flow graph** with `CFNode` variants —
`CFApplyEffects [IdTagged CFEffect]`, `CFExecuteCommand`, `CFExecuteSubshell`,
`CFSetExitCode`, `CFImpliedExit`, `CFUnreachable` — and `CFEdge` labels `CFEFlow`,
`CFEFalseFlow` (an edge a human *thinks* exists, e.g. from a backgrounded process to its
parent), `CFEErrExit`, `CFEExit`. Assignment tracking is `CFSetProps (Maybe Scope) String
(S.Set CFVariableProp)` on graph nodes. That is how SC2030/SC2031 ("modified in a subshell")
can be correct at all.

mvdan/sh models the same shape as `File{Stmts []*Stmt}` with `Stmt.Semicolon Pos` and
`BinaryCmd{Op, X, Y}` — the semicolon is a *position on a statement*, not a delimiter in text.

### (b) What bashrs does today

428 of 496 rules iterate `source.lines()`. There is no command-list model in the linter at
all, so "the line" is the de-facto scope for every rule that needs one, and a `;`-separated
list silently merges N commands into one analysis unit. There is no dataflow or CFG pass —
`bash_parser/semantic.rs` exists but the linter does not consume it.

### (c) Does the fix match the field?

The likely fix — splitting a line on unquoted `;` before scanning — is a **text-level
approximation of a parse tree**, and it will be wrong on the constructs where `;` is not a
separator: inside `for((;;))`, inside `case` patterns (`;;`), inside a quoted string, inside
`${x:-a;b}`, and inside a nested `$( )`. It is a strict improvement over the status quo and
it is not what the field does.

### (d) Defensible vs. mistake

- **Defensible short-term:** `;`-splitting that is quote- and nesting-aware, *if* it reuses
  the one shared scanner from §3 rather than adding a 28th quote tracker.
- **Mistake to lock in:** treating the line as the analysis unit anywhere new. Every rule
  added on `source.lines()` is another instance of this bug waiting for a one-line function
  definition.
- **Honest scope note:** bashrs should not build a CFG. `CFG.hs` exists to serve a handful of
  checks (subshell-modification, unreachable code, `errexit` semantics). bashrs has none of
  those rules, and building 1300 lines of graph machinery to fix a paren false-positive would
  be gold-plating. Statement-level structure is the right altitude here; the CFG is context
  for what "the field does" at the far end, not a recommendation.

---

## 7. T6 — code namespace: bashrs is squatting SC****

This is the finding with consequences outside bashrs, and it is not cosmetic.

### (a) What the field does

**There is no formal reservation of the `SCxxxx` namespace.** I looked for a published
ShellCheck policy on third-party reuse and found none — so this is not a rule violation.
It is a practical hazard, and the hazard is demonstrable.

**Rust/clippy is the field's answer to exactly this problem.** Lints are *tool-namespaced*:
`#[allow(clippy::needless_return)]`, `#[allow(rustdoc::broken_intra_doc_links)]`. A tool must
be registered (`#![register_tool(foo)]`) and an unregistered prefix is a **hard compile
error**, E0710:

```
$ rustc --explain E0710
An unknown tool name was found in a scoped lint.
#[allow(clipp::filter_map)] // error!
```

Two properties worth copying: the namespace is explicit at every use site, and a typo or an
unknown tool fails loudly instead of silently doing nothing.

mvdan/sh sidesteps the question — it emits no numbered diagnostics at all.

### (b) What bashrs does today

bashrs ships **385 `SCxxxx` rule files**, and it **honours ShellCheck's suppression
directive**:

```console
$ printf '#!/bin/bash\n# shellcheck disable=SC1035\nheads=$(git for-each-ref refs/heads)\necho "$heads"\n' > t6.sh
$ bashrs lint t6.sh | grep SC1035
(silent — bashrs obeyed a directive addressed to a different tool)
```

There is no `bashrs disable=` namespace of its own.

Combined with **semantic collisions**, that is a live cross-tool defect. Sampling 13 codes
that both tools implement, and comparing bashrs's rule header against ShellCheck's message
*measured from the 0.8.0 binary*:

| Code | ShellCheck 0.8.0 (measured) | bashrs 6.67.0 | |
|---|---|---|---|
| SC1009 | `The mentioned syntax error was in this simple command.` (a **note** attached to a parse error) | `Comment detected where command was expected` | **COLLISION** |
| SC2032 | `Use own script or sh -c '..' to run this from sudo.` (functions through sudo/find) | `Use own script's variable. To set/use it, source script or remove shebang.` | **COLLISION** |
| SC2081 | `[ .. ] can't match globs. Use [[ .. ]] or case statement.` | `Expressions don't expand in single quotes…` (= real SC2016) | **COLLISION** |
| SC1028, SC1035, SC1078, SC1128, SC2016, SC2065, SC2105, SC2111, SC2122, SC2188 | — | — | compatible |

Three of thirteen sampled. This is a **sample, not an audit** — a full 385-code cross-check
is a recommended follow-up, not something this document performed.

Two aggravating details:

1. **SC2032's bashrs wording mimics ShellCheck's phrasing** (`Use own script's variable…`
   vs `Use own script or sh -c '..'…`), which makes the collision read as intentional
   compatibility rather than a clash.
2. **SC2081 duplicates bashrs's own SC2016** — both carry the identical message
   `Expressions don't expand in single quotes, use double quotes for that`. So the corpus
   gets the same finding twice under two codes, one of which is ShellCheck's glob rule.

The failure mode is concrete and bidirectional: a `# shellcheck disable=SC2032` written to
silence ShellCheck's sudo warning **also disables bashrs's unrelated shebang/variable rule**,
and a directive added for bashrs silently changes what ShellCheck reports. Neither tool can
warn about it, because to each of them the directive looks well-formed.

### (c) Does the fix match the field?

No fix is in flight for this in the other lanes, and it is the one class where the correct
action is not a lexer change.

### (d) Defensible vs. mistake

- **Defensible, and genuinely valuable:** implementing ShellCheck-*compatible* codes. Users
  know SC2086. Emitting SC2086 for the same defect with the same meaning is a real
  interoperability win and the reason bashrs is adoptable at all. Reading
  `# shellcheck disable=` is likewise defensible — for codes where bashrs means what
  ShellCheck means.
- **Design mistake bashrs should not repeat:** assigning a *different* meaning to a code
  ShellCheck already owns, while honouring ShellCheck's directive namespace. That is the
  worst of both worlds: users get ShellCheck's suppression semantics with bashrs's
  definitions.
- **Recommended, in the field's shape:**
  1. Audit all 385 SC codes against ShellCheck's published meanings; keep the ones that
     agree, **renumber the ones that do not** into a bashrs-owned range (bashrs already owns
     `SEC`, `DET`, `IDEM`, `POSIX`, `MAKE`, `PERF`, … — the precedent is in-house).
  2. Add a `# bashrs disable=` directive for bashrs-original codes.
  3. Keep honouring `# shellcheck disable=` **only for codes bashrs implements
     compatibly** — a directive naming a code bashrs has redefined should not silence the
     redefined rule.
  4. Follow clippy: make an unknown code in a bashrs directive a *diagnostic*, not a no-op.

---

## 8. Must-still-fire: the true-positive oracle for the fixing lanes

Every rule touched must keep firing on a genuine defect. These are measured, not asserted —
each was run against all three tools. **The fix lanes should adopt these as regression
tests.**

| Fixture | Genuinely broken? | bashrs (must keep) | ShellCheck | `bash -n` |
|---|---|---|---|---|
| `echo "unterminated string` + more lines | yes | **SC1078** | SC1009/1072/1073 | FAIL |
| `if [ ( -f /etc/passwd ) ]; then …` | yes | **SC1028** | SC1028 | FAIL |
| `> out.txt` on its own line | yes | **SC2188** | SC2188 | ok |
| `for i in 1 2; do{ echo $i;}` | yes | **SC1035** | SC1058/1072/1073 | FAIL |

And the false positives that must go silent — all four are valid bash, all four are clean
under ShellCheck *and* mvdan/sh:

| Fixture | bashrs today | ShellCheck | mvdan/sh | `bash -n` |
|---|---|---|---|---|
| multi-line `python3 -c "` (T4) | 2 errors | 0 | parses | ok |
| heredoc containing `<svg …>` (T1) | 3 errors | 0 | parses | ok |
| heredoc containing `#!` (T1) | 1 error | 0 | parses | ok |
| `[ "$rc" = 0 ] && echo "… ($x)"` (T2) | 4 errors | 0 | parses | ok |
| `[ "$(python3 -c "… >= 85")" != 1 ]` (T2) | 3 errors | 0 | parses | ok |
| `git for-each-ref` (T3) | 1 error | 0 | parses | ok |
| one-line fn with `;` list (T5) | 2 errors | 0 | parses | ok |
| `for((i=0;i<3;i++))` (T3, **not in corpus**) | 1 error | 0 | parses | ok |
| `while(true); do break; done` (T3, **not in corpus**) | 2 errors | 0 | parses | ok |

The last two are the trap: they are not in the rmedia corpus, so a fix tuned to drive the
corpus count to 29 can leave them broken and still look finished.

---

## 9. Where bashrs is genuinely better, and where the competition has the same bug

Being honest in both directions, because the point is correct fixes rather than a verdict.

**bashrs is better — error-tolerant diagnosis.** On a genuinely unterminated string,
ShellCheck stops analysing and emits a three-message parse cascade pointing at the *symptom*:

```
2:1: note:  The mentioned syntax error was in this simple command. [SC1009]
2:6: error: Couldn't parse this double quoted string. […]     [SC1073]
4:1: error: Expected end of double quoted string. […]          [SC1072]
```

bashrs emits one message at the *cause* — the column where the quote opened:

```
✗ 2:6-7 [error] SC1078: Did you forget to close this double-quoted string?
```

That is better UX, and the underlying design decision — a linter must keep working on input
that does not parse — is correct and is the right reason to reject "just use `bash_parser`".
The conclusion the field would draw from it is a **recovering tokenizer shared by all rules**,
not 27 private ones.

**bashrs is better — original rule classes.** SEC010/SEC011/DET002 have no ShellCheck
equivalent. The `rm -rf` symlink-aliasing hazard SEC011 found in this corpus is not
something ShellCheck or mvdan/sh would ever report. This is the actual product, and it is
what the false-positive noise is burying.

**ShellCheck's SC1078 is narrower than bashrs's, deliberately.** ShellCheck emits it as a
`WarningC`, and only when the string is *already terminated* but its closing quote looks
misplaced (`Parser.hs:1372-1383`):

```haskell
        try . lookAhead $ suspectCharAfterQuotes <|> oneOf "$\""
        when (any hasLineFeed x && not (startsWithLineFeed x)) $
            suggestForgotClosingQuote startPos endPos "double quoted string"
```

Read that condition carefully, because it is the single most useful line in this document
for the T4 fix: **a multi-line double-quoted string is not suspicious on its own.**
ShellCheck requires a linefeed *and* a suspect following character *and* that the string does
not begin with the linefeed — the last clause exempting exactly the
`STATS=$(python3 -c "\nimport json…")` shape. Fixing bashrs's lexer to track strings across
lines is necessary but **not sufficient**: a correct lexer that still reports every multi-line
string will keep most of the 73 findings. The rule needs the suspicion predicate too, and
bashrs should not make it an `Error` when ShellCheck makes it a `Warning`.

**The competition has bugs of the same family.** ShellCheck's own history includes repeated
widenings of `[ ]` handling, and `readDocLines`'s `checkEnd` carries a comment about
"a plausible false end" — the same class of heuristic bashrs is struggling with, just
solved once inside a parser instead of N times outside one. tree-sitter-bash cannot express
heredocs in its grammar at all and drops to hand-written C. Nobody gets this for free; the
difference is that they pay for it once.

---

## 10. Summary of recommendations

| # | Recommendation | Class |
|---|---|---|
| 1 | One shared, error-tolerant, byte-accurate tokenizer producing quote-ranges, heredoc-body ranges and command-substitution nesting for the whole source, computed once. Extend `linter::quoting::mask_literals` rather than adding a 28th private scanner. | T1 T2 T4 T5 |
| 2 | Adopt ShellCheck's `keywordSeparator` (`eof \| whitespace \| one of ";()[<>&\|"`) in SC1035 instead of "next char is not a space". Verify `for((` and `while(` go silent and `do{` still fires. | T3 |
| 3 | Add the SC1078 suspicion predicate (linefeed **and** suspect following char **and** not linefeed-initial) and downgrade to Warning to match ShellCheck. A lexer fix alone will not clear these 73. | T4 |
| 4 | Give `sc2188` real heredoc awareness and replace `heredoc_still_exempt_under_continuation_folding` with a fixture that contains `> data` inside the body — prove it fails before the fix. | T1 |
| 5 | Audit all 385 SC codes for semantic collision; renumber divergent ones into a bashrs-owned namespace; add `# bashrs disable=`; stop honouring `# shellcheck disable=` for redefined codes. | T6 |
| 6 | Track statement structure so a `;`-separated list is N units, reusing (1). Do **not** build a CFG. | T5 |

**Falsifiability of this document.** Every measurement here is a command in the text. The
corpus counts, the three-tool differential, the vacuous-test falsification, the namespace
collisions and the `for((` regression were each executed against bashrs 6.67.0, ShellCheck
0.8.0 and shfmt v3.14.0 on 2026-08-30. Where a claim is a sample rather than an audit
(§7), it says so.
