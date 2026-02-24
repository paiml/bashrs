# ShellCheck SC1xxx Rules (Source Code Issues)

bashrs implements 60 SC1xxx rules covering source-level shell script issues: shebang problems, quoting mistakes, spacing errors, syntax style, here-document issues, unicode encoding problems, portability concerns, and source/include warnings.

These rules detect issues that occur **before** the shell even begins interpreting the script -- encoding problems, syntax mistakes, and common typos that prevent correct parsing.

## Rule Categories

| Category | Rules | Count | Description |
|----------|-------|-------|-------------|
| Shebang | SC1008, SC1084, SC1104, SC1113-SC1115, SC1127-SC1128 | 8 | Shebang line issues |
| Quoting & Escaping | SC1003-SC1004, SC1012, SC1078-SC1079, SC1098, SC1110-SC1111, SC1117, SC1135 | 10 | Quote and escape problems |
| Spacing & Formatting | SC1007, SC1009, SC1020, SC1035, SC1068-SC1069, SC1095, SC1099, SC1101, SC1129 | 10 | Whitespace issues |
| Syntax Style | SC1014, SC1026, SC1028, SC1036, SC1045, SC1065-SC1066, SC1075, SC1086, SC1097 | 10 | Common syntax mistakes |
| Here-documents | SC1038, SC1040-SC1041, SC1044, SC1120 | 5 | Heredoc issues |
| Unicode & Encoding | SC1017-SC1018, SC1082, SC1100, SC1109 | 5 | Character encoding issues |
| Bash-in-sh Portability | SC1037, SC1076, SC1087, SC1105-SC1106, SC1131, SC1139-SC1140 | 8 | POSIX portability |
| Source/Include | SC1083, SC1090-SC1091, SC1094 | 4 | File sourcing issues |

## Shebang Rules

### SC1084: Use `#!` not `!#`

**Severity:** Error

Detects reversed shebang where `!#` is used instead of `#!`.

```bash
# Bad:
!#/bin/bash
echo "hello"

# Good:
#!/bin/bash
echo "hello"
```

### SC1113: Use `#!` not just `#`

**Severity:** Warning

Detects shebang missing the `!` character.

```bash
# Bad:
# /bin/sh
echo "hello"

# Good:
#!/bin/sh
echo "hello"
```

### SC1114: Leading spaces before shebang

**Severity:** Warning

Shebang must be the very first characters of the file.

```bash
# Bad:
  #!/bin/sh
echo "hello"

# Good:
#!/bin/sh
echo "hello"
```

### SC1115: Space between `#` and `!`

**Severity:** Warning

Detects `# !` instead of `#!`.

### SC1127: Use `#` for comments, not `//`

**Severity:** Warning

Detects C/C++ style comments that will be interpreted as commands.

```bash
# Bad:
// This is a comment

# Good:
# This is a comment
```

### SC1128: Shebang must be first line

**Severity:** Warning

Detects shebang on a non-first line.

```bash
# Bad:
echo "starting"
#!/bin/bash

# Good:
#!/bin/bash
echo "starting"
```

## Quoting & Escape Rules

### SC1003: Want to escape a single quote?

**Severity:** Warning

Detects `'don't'` patterns where a single quote breaks the string.

```bash
# Bad:
echo 'don't do this'

# Good:
echo 'don'\''t do this'
echo "don't do this"
```

### SC1004: Backslash+linefeed in single quotes

**Severity:** Info

In single quotes, `\n` is literal backslash-n, not a newline.

### SC1012: `\t` is literal in single quotes

**Severity:** Info

In single quotes, `\t` is literal, not a tab. Use `$'\t'` or double quotes.

```bash
# Bad:
echo 'line1\tline2'   # Prints literal \t

# Good:
echo "line1\tline2"   # Prints tab
printf 'line1\tline2'  # printf interprets \t
```

### SC1078: Unclosed double-quoted string

**Severity:** Error

Detects unmatched double quotes on a line.

### SC1110 / SC1111: Unicode quotes

**Severity:** Error

Detects Unicode curly quotes (`\u201c` `\u201d` `\u2018` `\u2019`) that should be ASCII quotes.

```bash
# Bad (unicode):
echo \u201chello\u201d

# Good (ASCII):
echo "hello"
```

### SC1098: Quote special characters in eval

**Severity:** Warning

Detects unquoted variables in `eval` statements.

```bash
# Bad:
eval $cmd

# Good:
eval "$cmd"
```

## Spacing & Formatting Rules

### SC1007: Remove spaces around `=`

**Severity:** Error

```bash
# Bad:
VAR = value

# Good:
VAR=value
```

### SC1068: Don't put spaces around `=` in assignments

**Severity:** Error

```bash
# Bad:
let x = 1

# Good:
let x=1
```

### SC1069: Missing space before `[`

**Severity:** Error

```bash
# Bad:
if[ -f file ]; then echo ok; fi

# Good:
if [ -f file ]; then echo ok; fi
```

### SC1101: Trailing spaces after `\` continuation

**Severity:** Warning

Detects invisible trailing whitespace after line continuation backslash.

## Syntax Style Rules

### SC1065: Don't declare function parameters

**Severity:** Error

Shell functions don't take named parameters -- use `$1`, `$2`, etc.

```bash
# Bad:
function myfunc(x, y) {
    echo "$x $y"
}

# Good:
myfunc() {
    echo "$1 $2"
}
```

### SC1066: Don't use `$` on left side of assignments

**Severity:** Error

```bash
# Bad:
$VAR=value

# Good:
VAR=value
```

### SC1075: Use `elif` not `else if`

**Severity:** Warning

```bash
# Bad:
if [ "$x" = 1 ]; then
    echo "one"
else if [ "$x" = 2 ]; then
    echo "two"
fi
fi

# Good:
if [ "$x" = 1 ]; then
    echo "one"
elif [ "$x" = 2 ]; then
    echo "two"
fi
```

### SC1086: Don't use `$` on for loop variable

**Severity:** Error

```bash
# Bad:
for $i in 1 2 3; do echo "$i"; done

# Good:
for i in 1 2 3; do echo "$i"; done
```

### SC1097: Use `=` not `==` in `[ ]`

**Severity:** Warning

POSIX `test` uses `=` for string comparison, not `==`.

## Here-document Rules

### SC1040: With `<<-`, indent with tabs only

**Severity:** Warning

The `<<-` heredoc operator only strips leading tabs, not spaces.

### SC1041: Delimiter on same line as `<<`

**Severity:** Error

The heredoc body starts on the next line after `<<`.

### SC1044: Unterminated here-document

**Severity:** Error

The closing delimiter was not found.

### SC1120: No comments after heredoc token

**Severity:** Warning

```bash
# Bad:
cat <<EOF # this is a comment
hello
EOF

# Good:
# this is a comment
cat <<EOF
hello
EOF
```

## Unicode & Encoding Rules

### SC1017: Literal carriage return

**Severity:** Error

Detects Windows-style `\r\n` line endings in shell scripts.

### SC1018: Unicode non-breaking space

**Severity:** Error

Detects invisible `\u00a0` characters that look like spaces but aren't.

### SC1082: UTF-8 BOM detected

**Severity:** Warning

Detects the UTF-8 Byte Order Mark (`\xef\xbb\xbf`) at the start of a file.

### SC1100: Unicode dash instead of minus

**Severity:** Error

Detects en-dash (`\u2013`) or em-dash (`\u2014`) used where a minus/hyphen is needed.

### SC1109: Unquoted HTML entity

**Severity:** Warning

Detects `&amp;`, `&lt;`, etc. that suggest the script was copy-pasted from a web page.

## Bash-in-sh Portability Rules

These rules only fire on `#!/bin/sh` scripts to catch bashisms.

### SC1037: Braces required for positional parameters beyond `$9`

**Severity:** Error

```bash
# Bad:
echo $10   # Interpreted as $1 followed by literal 0

# Good:
echo ${10}
```

### SC1076: Use `$((...))` for math, not `$[...]`

**Severity:** Warning

The `$[expr]` syntax is deprecated. Use `$((expr))`.

### SC1087: Use braces for array access

**Severity:** Warning

```bash
# Bad:
echo $arr[0]

# Good:
echo ${arr[0]}
```

## Source/Include Warnings

### SC1090: Can't follow non-constant source

**Severity:** Info

When `source` or `.` is used with a variable argument, static analysis cannot determine which file is sourced.

```bash
# Flagged (informational):
source "$config_file"
. "$HOME/.bashrc"

# Not flagged:
source /etc/profile
```

### SC1091: Source file not followed

**Severity:** Info

The linter notes it is not verifying sourced files.

### SC1083: `{` / `}` is literal

**Severity:** Warning

Unquoted braces may be interpreted literally rather than as a group.

## Usage

All SC1xxx rules are automatically applied when linting:

```bash
# Lint a script (SC1xxx rules included automatically)
bashrs lint script.sh

# Filter output to SC1xxx only
bashrs lint script.sh 2>&1 | grep "SC1"
```

## Shell Type Filtering

Most SC1xxx rules are **Universal** (apply to all shell types). A few are shell-specific:

- **SC1095** (function keyword spacing): NotSh -- only applies to bash/zsh
- **SC1037, SC1076, SC1087** (positional params, `$[]`, arrays): ShOnly portability warnings
- **SC1105, SC1106, SC1131, SC1139, SC1140**: Bash-in-sh portability

bashrs automatically applies the correct rules based on [shell type detection](./shell-detection.md).
