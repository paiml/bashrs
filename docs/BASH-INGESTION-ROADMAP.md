# Bash Ingestion Roadmap - EXTREME TDD

**Goal**: Document every Bash construct transformation to Rust and Purified Bash
**Methodology**: EXTREME TDD (Test-First, RED-GREEN-REFACTOR)
**Reference**: GNU Bash Manual (bash.pdf)
**Status**: IN PROGRESS

---

## Roadmap Overview

This roadmap documents the transformation of every Bash construct from the GNU Bash manual into:
1. **Rust code** (using Rash transpiler)
2. **Purified Bash** (deterministic, idempotent, safe)

Each item follows EXTREME TDD:
- ✅ Write test FIRST (RED)
- ✅ Implement transformation (GREEN)
- ✅ Refactor for quality (REFACTOR)
- ✅ Document with examples

---

## Chapter 1: Introduction

### 1.1 What is Bash?
- [ ] **Task**: Document bash shebang transformation
  - Input: `#!/bin/bash`
  - Rust: `fn main() {}`
  - Purified: `#!/bin/sh` (POSIX)
  - Test: `test_shebang_transformation`

### 1.2 What is a shell?
- [ ] **Task**: Document interactive vs script mode
  - Input: Interactive bash session
  - Rust: Not applicable (compile-time only)
  - Purified: Script mode only (deterministic)
  - Test: `test_script_mode_only`

---

## Chapter 2: Definitions

### 2.1 POSIX Compliance
- [ ] **Task**: Document POSIX-only constructs
  - Input: Bash-specific syntax
  - Rust: Standard Rust constructs
  - Purified: POSIX sh syntax only
  - Test: `test_posix_compliance_check`

---

## Chapter 3: Basic Shell Features

### 3.1 Shell Syntax

#### 3.1.1 Shell Operation
- [ ] **Task**: Document command execution
  - Input: `echo "hello"`
  - Rust: `fn main() { echo("hello"); }`
  - Purified: `printf '%s\n' "hello"` (POSIX printf)
  - Test: `test_echo_to_printf_transformation`

#### 3.1.2 Quoting

##### 3.1.2.1 Escape Character
- [ ] **Task**: Document backslash escaping
  - Input: `echo "Hello \"World\""`
  - Rust: `println!("Hello \"World\"")`
  - Purified: `printf '%s\n' "Hello \"World\""`
  - Test: `test_escape_character_preservation`

##### 3.1.2.2 Single Quotes
- [ ] **Task**: Document single quote literals
  - Input: `echo 'It'\''s working'`
  - Rust: `println!("It's working")`
  - Purified: `printf '%s\n' "It's working"` (use double quotes)
  - Test: `test_single_quote_transformation`

##### 3.1.2.3 Double Quotes
- [ ] **Task**: Document double quote preservation
  - Input: `echo "Value: $VAR"`
  - Rust: `println!("Value: {}", var)`
  - Purified: `printf '%s\n' "Value: $VAR"` (quoted)
  - Test: `test_double_quote_variable_expansion`

##### 3.1.2.4 ANSI-C Quoting
- [ ] **Task**: Document $'...' transformation
  - Input: `echo $'Hello\nWorld'`
  - Rust: `println!("Hello\nWorld")`
  - Purified: `printf '%s\n%s\n' "Hello" "World"` (explicit)
  - Test: `test_ansi_c_quoting_to_explicit`

### 3.2 Shell Commands

#### 3.2.1 Simple Commands
- [ ] **Task**: Document command with arguments
  - Input: `mkdir -p /tmp/data`
  - Rust: `fn main() { mkdir_p("/tmp/data"); }`
  - Purified: `mkdir -p "/tmp/data"` (quoted, idempotent)
  - Test: `test_simple_command_transformation`

#### 3.2.2 Pipelines
- [ ] **Task**: Document pipe transformation
  - Input: `cat file.txt | grep "pattern"`
  - Rust: `grep(cat("file.txt"), "pattern")`
  - Purified: `cat "file.txt" | grep "pattern"` (quoted paths)
  - Test: `test_pipeline_transformation`

#### 3.2.3 Lists
- [ ] **Task**: Document command lists (&&, ||, ;)
  - Input: `cmd1 && cmd2 || cmd3`
  - Rust: `if cmd1() { cmd2() } else { cmd3() }`
  - Purified: `cmd1 && cmd2 || cmd3` (same, but quoted)
  - Test: `test_command_list_transformation`

#### 3.2.4 Compound Commands

##### 3.2.4.1 Looping Constructs

**Until Loop**
- [ ] **Task**: Document until loop transformation
  - Input: `until [ $i -gt 5 ]; do echo $i; i=$((i+1)); done`
  - Rust: `while !(i > 5) { println!("{}", i); i += 1; }`
  - Purified: `while [ ! "$i" -gt 5 ]; do printf '%s\n' "$i"; i=$((i+1)); done`
  - Test: `test_until_to_while_transformation`

**While Loop**
- [x] **Task**: Document while loop (COMPLETE - v0.8.0)
  - Input: `while [ $i -lt 5 ]; do echo $i; i=$((i+1)); done`
  - Rust: `while i < 5 { println!("{}", i); i += 1; }`
  - Purified: `while [ "$i" -lt 5 ]; do printf '%s\n' "$i"; i=$((i+1)); done`
  - Test: `test_while_loop_transformation` ✅

**For Loop**
- [x] **Task**: Document for loop (COMPLETE - v0.5.0)
  - Input: `for i in {1..5}; do echo $i; done`
  - Rust: `for i in 1..=5 { println!("{}", i); }`
  - Purified: `for i in $(seq 1 5); do printf '%s\n' "$i"; done`
  - Test: `test_for_loop_transformation` ✅

##### 3.2.4.2 Conditional Constructs

**If Statement**
- [x] **Task**: Document if/else (COMPLETE - v0.4.0)
  - Input: `if [ "$VAR" = "value" ]; then echo "yes"; else echo "no"; fi`
  - Rust: `if var == "value" { println!("yes") } else { println!("no") }`
  - Purified: `if [ "$VAR" = "value" ]; then printf '%s\n' "yes"; else printf '%s\n' "no"; fi`
  - Test: `test_if_statement_transformation` ✅

**Case Statement**
- [x] **Task**: Document case/match (COMPLETE - v0.6.0)
  - Input: `case $VAR in 1) echo "one";; 2) echo "two";; esac`
  - Rust: `match var { 1 => println!("one"), 2 => println!("two"), _ => {} }`
  - Purified: `case "$VAR" in 1) printf '%s\n' "one";; 2) printf '%s\n' "two";; esac`
  - Test: `test_case_statement_transformation` ✅

**Select Statement**
- [ ] **Task**: Document select menu transformation
  - Input: `select opt in "A" "B"; do echo $opt; break; done`
  - Rust: Not supported (interactive only)
  - Purified: Not supported (non-deterministic)
  - Test: `test_select_not_supported`

### 3.3 Shell Functions

#### 3.3.1 Function Definition
- [x] **Task**: Document function syntax (COMPLETE - v0.4.0)
  - Input: `function greet() { echo "Hello $1"; }`
  - Rust: `fn greet(name: &str) { println!("Hello {}", name); }`
  - Purified: `greet() { printf '%s %s\n' "Hello" "$1"; }`
  - Test: `test_function_definition` ✅

#### 3.3.2 Function Return Values
- [x] **Task**: Document return values (COMPLETE - v0.4.0)
  - Input: `add() { echo $(($1 + $2)); }; result=$(add 3 5)`
  - Rust: `fn add(a: i32, b: i32) -> i32 { a + b }; let result = add(3, 5);`
  - Purified: `add() { echo "$(($1 + $2))"; }; result="$(add 3 5)"`
  - Test: `test_function_return_values` ✅

### 3.4 Shell Parameters

#### 3.4.1 Positional Parameters
- [ ] **Task**: Document $1, $2, etc.
  - Input: `echo "First: $1, Second: $2"`
  - Rust: `fn main(args: Vec<String>) { println!("First: {}, Second: {}", args[0], args[1]); }`
  - Purified: `printf '%s %s, %s %s\n' "First:" "$1" "Second:" "$2"`
  - Test: `test_positional_parameters`

#### 3.4.2 Special Parameters

**$#** (Argument Count)
- [ ] **Task**: Document $# transformation
  - Input: `echo "Args: $#"`
  - Rust: `println!("Args: {}", args.len())`
  - Purified: `printf '%s %s\n' "Args:" "$#"`
  - Test: `test_arg_count_parameter`

**$?** (Exit Status)
- [ ] **Task**: Document $? transformation
  - Input: `cmd; echo "Exit: $?"`
  - Rust: `let exit = cmd(); println!("Exit: {}", exit);`
  - Purified: `cmd; _exit="$?"; printf '%s %s\n' "Exit:" "$_exit"`
  - Test: `test_exit_status_parameter`

**$$** (Process ID)
- [ ] **Task**: Document $$ purification
  - Input: `echo "PID: $$"`
  - Rust: Not supported (non-deterministic)
  - Purified: Remove (use fixed identifier)
  - Test: `test_pid_removed_for_determinism`

**$!** (Background PID)
- [ ] **Task**: Document $! purification
  - Input: `cmd & echo "BG: $!"`
  - Rust: Not supported (background jobs non-deterministic)
  - Purified: Remove (use synchronous execution)
  - Test: `test_background_pid_removed`

**$0** (Script Name)
- [ ] **Task**: Document $0 transformation
  - Input: `echo "Script: $0"`
  - Rust: `println!("Script: {}", std::env::args().nth(0).unwrap())`
  - Purified: `printf '%s %s\n' "Script:" "$0"`
  - Test: `test_script_name_parameter`

**$-** (Shell Options)
- [ ] **Task**: Document $- purification
  - Input: `echo "Options: $-"`
  - Rust: Not supported (runtime-specific)
  - Purified: Remove (not needed in purified scripts)
  - Test: `test_shell_options_removed`

### 3.5 Shell Expansions

#### 3.5.1 Brace Expansion
- [ ] **Task**: Document brace expansion
  - Input: `echo {1..5}`
  - Rust: `for i in 1..=5 { print!("{} ", i); }`
  - Purified: `seq 1 5 | tr '\n' ' '` (explicit)
  - Test: `test_brace_expansion_to_seq`

#### 3.5.2 Tilde Expansion
- [ ] **Task**: Document ~ expansion
  - Input: `cd ~/docs`
  - Rust: `std::env::home_dir()`
  - Purified: `cd "$HOME/docs"` (explicit $HOME)
  - Test: `test_tilde_to_home_expansion`

#### 3.5.3 Shell Parameter Expansion

**${parameter:-word}** (Default Value)
- [ ] **Task**: Document default value expansion
  - Input: `echo "${VAR:-default}"`
  - Rust: `let val = var.unwrap_or("default");`
  - Purified: `printf '%s\n' "${VAR:-default}"` (same)
  - Test: `test_default_value_expansion`

**${parameter:=word}** (Assign Default)
- [ ] **Task**: Document assign default
  - Input: `echo "${VAR:=default}"`
  - Rust: `let val = var.get_or_insert("default");`
  - Purified: `VAR="${VAR:=default}"; printf '%s\n' "$VAR"`
  - Test: `test_assign_default_expansion`

**${parameter:?word}** (Error if Unset)
- [ ] **Task**: Document error expansion
  - Input: `echo "${VAR:?error message}"`
  - Rust: `let val = var.expect("error message");`
  - Purified: `: "${VAR:?error message}"; printf '%s\n' "$VAR"`
  - Test: `test_error_if_unset_expansion`

**${parameter:+word}** (Alternative Value)
- [ ] **Task**: Document alternative value
  - Input: `echo "${VAR:+set}"`
  - Rust: `let val = if var.is_some() { "set" } else { "" };`
  - Purified: `printf '%s\n' "${VAR:+set}"`
  - Test: `test_alternative_value_expansion`

**${#parameter}** (String Length)
- [ ] **Task**: Document length expansion
  - Input: `echo "${#VAR}"`
  - Rust: `println!("{}", var.len())`
  - Purified: `printf '%s\n' "${#VAR}"` (POSIX compliant)
  - Test: `test_string_length_expansion`

**${parameter%word}** (Remove Suffix)
- [ ] **Task**: Document suffix removal
  - Input: `file="test.txt"; echo "${file%.txt}"`
  - Rust: `file.strip_suffix(".txt").unwrap_or(file)`
  - Purified: `printf '%s\n' "${file%.txt}"`
  - Test: `test_remove_suffix_expansion`

**${parameter#word}** (Remove Prefix)
- [ ] **Task**: Document prefix removal
  - Input: `path="/tmp/file"; echo "${path#/tmp/}"`
  - Rust: `path.strip_prefix("/tmp/").unwrap_or(path)`
  - Purified: `printf '%s\n' "${path#/tmp/}"`
  - Test: `test_remove_prefix_expansion`

**${parameter/pattern/string}** (Substitution)
- [ ] **Task**: Document pattern substitution
  - Input: `text="hello"; echo "${text/l/L}"`
  - Rust: `text.replacen("l", "L", 1)`
  - Purified: Use `sed` or `awk` (POSIX doesn't support ${//})
  - Test: `test_pattern_substitution_to_sed`

#### 3.5.4 Command Substitution
- [x] **Task**: Document $() and backticks (COMPLETE - v0.4.0)
  - Input: `result=$(date); result=\`date\``
  - Rust: `let result = date();`
  - Purified: `result="$(date)"` (prefer $() over backticks)
  - Test: `test_command_substitution` ✅

#### 3.5.5 Arithmetic Expansion
- [x] **Task**: Document $((...)) (COMPLETE - v0.4.0)
  - Input: `result=$((3 + 5 * 2))`
  - Rust: `let result = 3 + 5 * 2;`
  - Purified: `result="$((3 + 5 * 2))"` (POSIX arithmetic)
  - Test: `test_arithmetic_expansion` ✅

#### 3.5.6 Process Substitution
- [ ] **Task**: Document <(...) and >(...)
  - Input: `diff <(cmd1) <(cmd2)`
  - Rust: Not supported (bash-specific)
  - Purified: Use temporary files instead
  - Test: `test_process_substitution_to_temp_files`

#### 3.5.7 Word Splitting
- [ ] **Task**: Document IFS-based splitting
  - Input: `IFS=':'; read -ra PARTS <<< "$PATH"`
  - Rust: `let parts: Vec<_> = path.split(':').collect();`
  - Purified: Use explicit `tr` or `cut` (avoid IFS manipulation)
  - Test: `test_word_splitting_purification`

#### 3.5.8 Filename Expansion (Globbing)
- [ ] **Task**: Document glob patterns
  - Input: `for f in *.txt; do echo $f; done`
  - Rust: `for f in glob("*.txt") { println!("{}", f); }`
  - Purified: `for f in *.txt; do printf '%s\n' "$f"; done` (quoted)
  - Test: `test_glob_pattern_transformation`

### 3.6 Redirections

#### 3.6.1 Redirecting Input
- [ ] **Task**: Document < redirection
  - Input: `cmd < input.txt`
  - Rust: `cmd(File::open("input.txt"))`
  - Purified: `cmd < "input.txt"` (quoted path)
  - Test: `test_input_redirection`

#### 3.6.2 Redirecting Output
- [ ] **Task**: Document > and >> redirection
  - Input: `echo "text" > file.txt; echo "more" >> file.txt`
  - Rust: `write_file("file.txt", "text"); append_file("file.txt", "more");`
  - Purified: `printf '%s\n' "text" > "file.txt"; printf '%s\n' "more" >> "file.txt"`
  - Test: `test_output_redirection`

#### 3.6.3 Appending Redirected Output
- [ ] **Task**: Document >> append (covered above)

#### 3.6.4 Redirecting Standard Output and Standard Error
- [ ] **Task**: Document &> redirection
  - Input: `cmd &> output.txt`
  - Rust: `cmd_redirect("output.txt")`
  - Purified: `cmd > "output.txt" 2>&1` (POSIX equivalent)
  - Test: `test_stderr_stdout_redirection`

#### 3.6.5 Here Documents
- [ ] **Task**: Document << heredoc
  - Input: `cat << EOF\nHello\nWorld\nEOF`
  - Rust: `println!("Hello\nWorld")`
  - Purified: `cat << 'EOF'\nHello\nWorld\nEOF` (quoted delimiter)
  - Test: `test_heredoc_transformation`

#### 3.6.6 Here Strings
- [ ] **Task**: Document <<< herestring
  - Input: `cmd <<< "input string"`
  - Rust: `cmd("input string")`
  - Purified: `printf '%s' "input string" | cmd` (POSIX pipe)
  - Test: `test_herestring_to_pipe`

---

## Chapter 4: Shell Builtin Commands

### 4.1 Bourne Shell Builtins

- [ ] **Task**: Document `:` (no-op)
  - Input: `: # comment`
  - Rust: `// comment`
  - Purified: `: # comment` (POSIX no-op)
  - Test: `test_noop_colon`

- [ ] **Task**: Document `.` (source)
  - Input: `. ./config.sh`
  - Rust: `include!("config.rs")`
  - Purified: `. "./config.sh"` (quoted path)
  - Test: `test_source_command`

- [x] **Task**: Document `break` (COMPLETE - v0.8.0)
  - Input: `while true; do break; done`
  - Rust: `while true { break; }`
  - Purified: `while true; do break; done`
  - Test: `test_break_statement` ✅

- [x] **Task**: Document `continue` (COMPLETE - v0.8.0)
  - Input: `for i in 1 2 3; do continue; done`
  - Rust: `for i in 1..=3 { continue; }`
  - Purified: `for i in 1 2 3; do continue; done`
  - Test: `test_continue_statement` ✅

- [ ] **Task**: Document `cd`
  - Input: `cd /tmp`
  - Rust: `std::env::set_current_dir("/tmp")`
  - Purified: `cd "/tmp"` (quoted path)
  - Test: `test_cd_command`

- [x] **Task**: Document `echo` (COMPLETE - v0.4.0)
  - Input: `echo "Hello World"`
  - Rust: `println!("Hello World")`
  - Purified: `printf '%s\n' "Hello World"` (prefer printf)
  - Test: `test_echo_to_printf` ✅

- [ ] **Task**: Document `eval`
  - Input: `cmd="echo hello"; eval $cmd`
  - Rust: Not supported (dynamic execution unsafe)
  - Purified: Remove (security risk, not deterministic)
  - Test: `test_eval_not_supported`

- [ ] **Task**: Document `exec`
  - Input: `exec ./new-script.sh`
  - Rust: `std::process::Command::new("./new-script.sh").exec()`
  - Purified: Remove (replaces process, not idempotent)
  - Test: `test_exec_not_supported`

- [ ] **Task**: Document `exit`
  - Input: `exit 0`
  - Rust: `std::process::exit(0)`
  - Purified: `exit 0` (but avoid in functions, use return)
  - Test: `test_exit_command`

- [ ] **Task**: Document `export`
  - Input: `export VAR="value"`
  - Rust: `std::env::set_var("VAR", "value")`
  - Purified: `VAR="value"; export VAR` (two-step for clarity)
  - Test: `test_export_command`

- [ ] **Task**: Document `pwd`
  - Input: `current=$(pwd)`
  - Rust: `let current = std::env::current_dir()?;`
  - Purified: `current="$(pwd)"`
  - Test: `test_pwd_command`

- [ ] **Task**: Document `read`
  - Input: `read -r var`
  - Rust: Not supported (interactive input non-deterministic)
  - Purified: Remove (use command-line args instead)
  - Test: `test_read_not_supported`

- [x] **Task**: Document `return` (COMPLETE - v0.4.0)
  - Input: `func() { return 1; }`
  - Rust: `fn func() -> Result<(), String> { Err("error".into()) }`
  - Purified: `func() { return 1; }`
  - Test: `test_return_statement` ✅

- [ ] **Task**: Document `set`
  - Input: `set -e` (exit on error)
  - Rust: Not applicable (compile-time checking)
  - Purified: `set -e` (preserve safety flags)
  - Test: `test_set_flags`

- [ ] **Task**: Document `shift`
  - Input: `shift; echo $1`
  - Rust: `args.remove(0); println!("{}", args[0])`
  - Purified: `shift; printf '%s\n' "$1"`
  - Test: `test_shift_command`

- [ ] **Task**: Document `test` / `[`
  - Input: `if [ -f "file.txt" ]; then echo "exists"; fi`
  - Rust: `if std::path::Path::new("file.txt").exists() { println!("exists"); }`
  - Purified: `if [ -f "file.txt" ]; then printf '%s\n' "exists"; fi`
  - Test: `test_test_command`

- [ ] **Task**: Document `times`
  - Input: `times`
  - Rust: Not supported (profiling, non-deterministic)
  - Purified: Remove (use external profiling tools)
  - Test: `test_times_not_supported`

- [ ] **Task**: Document `trap`
  - Input: `trap 'cleanup' EXIT`
  - Rust: Use Drop trait
  - Purified: `trap 'cleanup' EXIT` (preserve for safety)
  - Test: `test_trap_signal_handling`

- [ ] **Task**: Document `umask`
  - Input: `umask 022`
  - Rust: `std::fs::set_permissions()`
  - Purified: `umask 022` (preserve security settings)
  - Test: `test_umask_command`

- [ ] **Task**: Document `unset`
  - Input: `unset VAR`
  - Rust: Variables go out of scope
  - Purified: `unset VAR` (explicit cleanup)
  - Test: `test_unset_command`

### 4.2 Bash Builtin Commands

- [ ] **Task**: Document `alias`
  - Input: `alias ll='ls -la'`
  - Rust: Not supported (interactive feature)
  - Purified: Remove (use functions instead)
  - Test: `test_alias_to_function`

- [ ] **Task**: Document `declare`/`typeset`
  - Input: `declare -i num=5`
  - Rust: `let num: i32 = 5;`
  - Purified: `num=5` (sh doesn't have declare)
  - Test: `test_declare_to_assignment`

- [ ] **Task**: Document `let`
  - Input: `let "x = 5 + 3"`
  - Rust: `let x = 5 + 3;`
  - Purified: `x=$((5 + 3))` (use arithmetic expansion)
  - Test: `test_let_to_arithmetic`

- [ ] **Task**: Document `local`
  - Input: `func() { local var=5; }`
  - Rust: `fn func() { let var = 5; }`
  - Purified: `func() { _var=5; }` (use naming convention)
  - Test: `test_local_to_scoped_var`

- [ ] **Task**: Document `printf`
  - Input: `printf '%s %d\n' "Number:" 42`
  - Rust: `println!("Number: {}", 42)`
  - Purified: `printf '%s %d\n' "Number:" 42` (preferred!)
  - Test: `test_printf_preservation`

- [ ] **Task**: Document `readarray`/`mapfile`
  - Input: `readarray -t lines < file.txt`
  - Rust: `let lines: Vec<_> = read_lines("file.txt")?;`
  - Purified: Use `while read` loop (POSIX)
  - Test: `test_readarray_to_while_read`

---

## Chapter 5: Shell Variables

### 5.1 Bourne Shell Variables

- [ ] **Task**: Document `HOME`
  - Input: `cd $HOME`
  - Rust: `std::env::home_dir()`
  - Purified: `cd "$HOME"` (always quoted)
  - Test: `test_home_variable`

- [ ] **Task**: Document `PATH`
  - Input: `PATH="/usr/local/bin:$PATH"`
  - Rust: `std::env::set_var("PATH", format!("/usr/local/bin:{}", env::var("PATH")))`
  - Purified: `PATH="/usr/local/bin:$PATH"; export PATH`
  - Test: `test_path_variable`

- [ ] **Task**: Document `IFS`
  - Input: `IFS=':'; read -ra parts <<< "$PATH"`
  - Rust: `let parts: Vec<_> = path.split(':').collect();`
  - Purified: Avoid IFS manipulation (use `tr`/`cut`)
  - Test: `test_ifs_purification`

- [ ] **Task**: Document `PS1`, `PS2`, etc.
  - Input: `PS1='$ '`
  - Rust: Not applicable (no interactive mode)
  - Purified: Remove (not needed in scripts)
  - Test: `test_prompt_vars_removed`

### 5.2 Bash Variables

- [ ] **Task**: Document `BASH_VERSION`
  - Input: `echo $BASH_VERSION`
  - Rust: `const VERSION: &str = "1.0.0";`
  - Purified: Remove (not available in sh)
  - Test: `test_bash_version_removed`

- [ ] **Task**: Document `RANDOM`
  - Input: `num=$RANDOM`
  - Rust: Not supported (non-deterministic)
  - Purified: Remove (use deterministic seed)
  - Test: `test_random_removed_for_determinism`

- [ ] **Task**: Document `SECONDS`
  - Input: `echo $SECONDS`
  - Rust: Use `std::time::Instant`
  - Purified: Remove (non-deterministic timing)
  - Test: `test_seconds_removed`

---

## Chapter 6: Bash Features

### 6.1 Arrays

- [ ] **Task**: Document indexed arrays
  - Input: `arr=(1 2 3); echo ${arr[0]}`
  - Rust: `let arr = vec![1, 2, 3]; println!("{}", arr[0]);`
  - Purified: Use whitespace-separated strings (POSIX sh has no arrays)
  - Test: `test_array_to_whitespace_list`

- [ ] **Task**: Document associative arrays
  - Input: `declare -A map; map[key]=value`
  - Rust: `let mut map = HashMap::new(); map.insert("key", "value");`
  - Purified: Not supported (use multiple variables)
  - Test: `test_associative_array_not_supported`

### 6.2 The Directory Stack (pushd/popd)

- [ ] **Task**: Document pushd/popd
  - Input: `pushd /tmp; popd`
  - Rust: Not supported (use explicit cd tracking)
  - Purified: `_prev="$(pwd)"; cd "/tmp"; cd "$_prev"`
  - Test: `test_pushd_popd_to_explicit_cd`

### 6.3 Controlling the Prompt

- [ ] **Task**: Document PROMPT_COMMAND
  - Input: `PROMPT_COMMAND='date'`
  - Rust: Not applicable (no interactive mode)
  - Purified: Remove (not needed)
  - Test: `test_prompt_command_removed`

### 6.4 The Restricted Shell

- [ ] **Task**: Document restricted mode
  - Input: `bash -r script.sh`
  - Rust: Not applicable (compile-time restrictions)
  - Purified: Not applicable
  - Test: `test_restricted_mode_not_applicable`

### 6.5 Bash POSIX Mode

- [ ] **Task**: Document --posix flag
  - Input: `bash --posix script.sh`
  - Rust: Always POSIX-compliant output
  - Purified: Always POSIX sh (default)
  - Test: `test_always_posix_compliant`

---

## Chapter 7: Job Control

- [ ] **Task**: Document background jobs (&)
  - Input: `cmd &`
  - Rust: Not supported (non-deterministic)
  - Purified: Remove (use synchronous execution)
  - Test: `test_background_jobs_removed`

- [ ] **Task**: Document `jobs` command
  - Input: `jobs`
  - Rust: Not supported
  - Purified: Remove
  - Test: `test_jobs_command_removed`

- [ ] **Task**: Document `fg`/`bg` commands
  - Input: `fg %1`
  - Rust: Not supported
  - Purified: Remove
  - Test: `test_fg_bg_removed`

---

## Chapter 8: Command Line Editing

- [ ] **Task**: Document readline features
  - Input: Interactive editing (Ctrl+A, Ctrl+E, etc.)
  - Rust: Not applicable (no interactive mode)
  - Purified: Not applicable
  - Test: `test_no_interactive_editing`

---

## Chapter 9: Using History Interactively

- [ ] **Task**: Document history expansion
  - Input: `!!` (repeat last command), `!$` (last arg)
  - Rust: Not applicable (no interactive mode)
  - Purified: Remove (not deterministic)
  - Test: `test_history_expansion_removed`

---

## Chapter 10: Installing Bash

- [ ] **Task**: Document installation
  - Not applicable to transformation

---

## Summary Statistics

### Completion Status

```
Total Tasks:        ~120 items
Completed (v0.x):   ~15 items ✅
In Progress:        ~105 items 🔄
Coverage:           ~13% complete
```

### Completed Features (v0.x)
- [x] While loops (v0.8.0)
- [x] For loops (v0.5.0)
- [x] If/else statements (v0.4.0)
- [x] Case/match statements (v0.6.0)
- [x] Functions (v0.4.0)
- [x] Return values (v0.4.0)
- [x] Command substitution (v0.4.0)
- [x] Arithmetic expansion (v0.4.0)
- [x] Break/continue (v0.8.0)
- [x] Echo transformation (v0.4.0)

### High Priority (Next 20 Tasks)
1. Positional parameters ($1, $2)
2. Special parameters ($#, $?)
3. String parameter expansion (${var:-default})
4. Redirections (<, >, >>)
5. Here documents (<<)
6. cd, pwd, exit commands
7. export, unset commands
8. test/[ command
9. printf preservation
10. PATH, HOME variables
11. Brace expansion purification
12. Tilde expansion
13. Glob patterns
14. Word splitting purification
15. Process substitution to temp files
16. alias to function transformation
17. declare to assignment
18. local to scoped var
19. Array to whitespace list
20. Remove RANDOM, $$, $!, $SECONDS

### EXTREME TDD Workflow

For each task:
1. **RED**: Write failing test first
   ```rust
   #[test]
   fn test_<feature>_transformation() {
       let bash = "<bash input>";
       let rust = "<expected rust>";
       let purified = "<expected purified bash>";

       assert_eq!(bash_to_rust(bash), rust);
       assert_eq!(bash_to_purified(bash), purified);
   }
   ```

2. **GREEN**: Implement transformation
   - Update parser for bash construct
   - Update Rust AST generation
   - Update purified bash emission

3. **REFACTOR**: Clean up implementation
   - Extract common patterns
   - Add helper functions
   - Document edge cases

4. **DOCUMENT**: Add to examples
   - Update PURIFICATION_WORKFLOW.md
   - Add to test fixtures
   - Update CHANGELOG

---

## Next Steps

1. **Start with High Priority tasks** (positional parameters, special vars)
2. **Follow EXTREME TDD** religiously (test-first, always)
3. **Document each transformation** with examples
4. **Track progress** in this roadmap (update checkboxes)
5. **Release incrementally** (v1.3.0, v1.4.0, etc.)

---

**Status**: 🔄 IN PROGRESS (13% complete)
**Methodology**: EXTREME TDD (RED-GREEN-REFACTOR)
**Goal**: 100% Bash manual coverage

🤖 Generated with [Claude Code](https://claude.com/claude-code)
