#!/bin/bash
# REPL Example 09: Tab Completion Mastery
# Demonstrates using tab completion to speed up your workflow
#
# This example shows:
# - Command completion
# - Mode name completion
# - File path completion
# - Bash construct completion
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 09: Tab Completion Mastery
=================================================================

This example demonstrates how to use tab completion effectively
to speed up your REPL workflow by 50% or more.

WHAT IS TAB COMPLETION?
-----------------------
Tab completion automatically completes:
- REPL commands (:parse, :purify, :lint, etc.)
- Mode names (normal, purify, lint, explain, debug)
- File paths (examples/, scripts/, ~/Documents/)
- Bash constructs (for, if, while, ${var:-default})

Press Tab to complete, press Tab twice to show all options.

=================================================================
STEP 1: Command Completion
=================================================================

Start typing a REPL command and press Tab:

bashrs [normal]> :mo<TAB>
# Completes to: :mode

bashrs [normal]> :p<TAB>
# Shows options:
# :parse  :purify

bashrs [normal]> :par<TAB>
# Completes to: :parse

bashrs [normal]> :h<TAB>
# Shows options:
# :help  :history

bashrs [normal]> :hi<TAB>
# Completes to: :history

# List all commands by pressing : and Tab twice
bashrs [normal]> :<TAB><TAB>
# Shows all commands:
# :clear
# :functions
# :history
# :lint
# :load
# :mode
# :parse
# :purify
# :reload
# :source
# :vars

=================================================================
STEP 2: Mode Name Completion
=================================================================

When using :mode, complete mode names with Tab:

bashrs [normal]> :mode pur<TAB>
# Completes to: :mode purify

bashrs [normal]> :mode li<TAB>
# Completes to: :mode lint

bashrs [normal]> :mode ex<TAB>
# Completes to: :mode explain

bashrs [normal]> :mode de<TAB>
# Completes to: :mode debug

# List all modes by typing :mode and Tab twice
bashrs [normal]> :mode <TAB><TAB>
# Shows all modes:
# debug
# explain
# lint
# normal
# purify

=================================================================
STEP 3: File Path Completion
=================================================================

Complete file paths when using :load or :source:

# Complete directory names
bashrs [normal]> :load ex<TAB>
# Completes to: :load examples/

bashrs [normal]> :load examples/rep<TAB>
# Completes to: :load examples/repl/

# Complete file names
bashrs [normal]> :load examples/sample_<TAB>
# Shows options:
# sample_bashrc.sh
# sample_ci.sh
# sample_deploy.sh
# sample_installer.sh
# sample_zshrc.sh

bashrs [normal]> :load examples/sample_ba<TAB>
# Completes to: :load examples/sample_bashrc.sh

# Absolute paths work too
bashrs [normal]> :load /etc/ba<TAB>
# Completes to: :load /etc/bash

bashrs [normal]> :load /etc/bash.<TAB>
# Completes to: :load /etc/bash.bashrc

# Tilde expansion
bashrs [normal]> :load ~/.<TAB>
# Shows hidden files in home directory:
# .bashrc
# .bash_profile
# .bash_history
# .zshrc
# ...

=================================================================
STEP 4: Bash Construct Completion
=================================================================

In explain mode, complete bash constructs:

bashrs [normal]> :mode explain
Switched to explain mode

# Parameter expansion completion
bashrs [explain]> ${var:<TAB>
# Shows options:
# ${var:-default}   # Use default value
# ${var:=default}   # Assign default value
# ${var:?error}     # Error if unset
# ${var:+alternate} # Use alternate value

bashrs [explain]> ${var:-<TAB>
# Completes to: ${var:-default}

# String operation completion
bashrs [explain]> ${var#<TAB>
# Shows options:
# ${var#prefix}     # Remove shortest prefix
# ${var##prefix}    # Remove longest prefix

bashrs [explain]> ${var%<TAB>
# Shows options:
# ${var%suffix}     # Remove shortest suffix
# ${var%%suffix}    # Remove longest suffix

# Control flow completion
bashrs [explain]> for<TAB>
# Expands to: for i in

bashrs [explain]> if<TAB>
# Expands to: if [

bashrs [explain]> while<TAB>
# Expands to: while [

bashrs [explain]> case<TAB>
# Expands to: case $var in

=================================================================
STEP 5: Case-Insensitive Completion
=================================================================

Tab completion is case-insensitive for convenience:

bashrs [normal]> :MO<TAB>
# Completes to: :mode (lowercase)

bashrs [normal]> :mode PUR<TAB>
# Completes to: :mode purify (lowercase)

bashrs [normal]> :LOAD Examples/<TAB>
# Completes to: :load examples/ (lowercase)

=================================================================
STEP 6: Completion in Context
=================================================================

Completion is context-aware:

# After :mode, only mode names are suggested
bashrs [normal]> :mode <TAB><TAB>
# Shows: debug, explain, lint, normal, purify

# After :load or :source, only files are suggested
bashrs [normal]> :load <TAB><TAB>
# Shows: files and directories

# After :, only commands are suggested
bashrs [normal]> :<TAB><TAB>
# Shows: :clear, :functions, :history, etc.

=================================================================
STEP 7: Speed Workflow with Tab Completion
=================================================================

Example: Switching modes quickly

# Without tab completion (slow)
bashrs [normal]> :mode purify
Switched to purify mode

# With tab completion (fast)
bashrs [normal]> :m<TAB>pur<TAB><ENTER>
# Result: :mode purify
Switched to purify mode

# Save 8 keystrokes!

Example: Loading files quickly

# Without tab completion
bashrs [normal]> :load examples/sample_deploy.sh

# With tab completion
bashrs [normal]> :l<TAB>ex<TAB>sam<TAB>de<TAB><ENTER>
# Result: :load examples/sample_deploy.sh

# Save 15+ keystrokes!

=================================================================
STEP 8: Combining Tab Completion with Workflow
=================================================================

Real-world workflow using tab completion:

# Step 1: Load script (using tab completion)
bashrs [normal]> :l<TAB>~/pr<TAB>de<TAB><ENTER>
# Expands to: :load ~/projects/deploy.sh

# Step 2: Switch to lint mode (using tab completion)
bashrs [normal]> :m<TAB>li<TAB><ENTER>
# Expands to: :mode lint

# Step 3: Reload script (using tab completion)
bashrs [lint]> :rel<TAB><ENTER>
# Expands to: :reload

# Step 4: Switch to purify mode (using tab completion)
bashrs [lint]> :m<TAB>pur<TAB><ENTER>
# Expands to: :mode purify

# Step 5: View history (using tab completion)
bashrs [purify]> :h<TAB><ENTER>
# Expands to: :history (or :help, press Tab again to choose)

=================================================================
STEP 9: Discovery with Tab Completion
=================================================================

Use tab completion to discover features:

# Discover all commands
bashrs [normal]> :<TAB><TAB>
# See all available commands

# Discover all modes
bashrs [normal]> :mode <TAB><TAB>
# See all available modes

# Discover parameter expansions
bashrs [explain]> ${<TAB><TAB>
# See all parameter expansion syntaxes

# Discover available scripts
bashrs [normal]> :load examples/<TAB><TAB>
# See all example scripts

=================================================================
STEP 10: Tab Completion Best Practices
=================================================================

1. Press Tab liberally
   Don't hesitate to press Tab after every few characters

2. Press Tab twice to list options
   When unsure, Tab Tab shows all possibilities

3. Use prefix typing
   Type 2-3 characters, then Tab to narrow options

4. Learn common abbreviations
   :m = :mode
   :l = :load
   :p = :parse or :purify (need one more character)
   :h = :history or :help (need one more character)

5. Navigate with Tab
   Use Tab to explore file systems and discover scripts

6. Combine with other shortcuts
   Tab + ↑/↓ (history) = super fast workflow
   Tab + Ctrl-R (search) = powerful combination

=================================================================
Time Savings with Tab Completion:
=================================================================

Command comparison (keystrokes):

Without Tab:
  :mode purify     → 12 keystrokes
  :load examples/sample_deploy.sh → 35 keystrokes
  :reload          → 7 keystrokes

With Tab:
  :m<TAB>pur<TAB>  → 6 keystrokes (50% reduction)
  :l<TAB>ex<TAB>sa<TAB>de<TAB> → 12 keystrokes (66% reduction)
  :rel<TAB>        → 4 keystrokes (43% reduction)

Estimated time savings:
- Average session: 100+ commands
- Time saved per session: 5-10 minutes
- Time saved per week: 25-50 minutes
- Time saved per year: 20-40 hours!

=================================================================
Practice Exercises:
=================================================================

Try these tab completion exercises to build muscle memory:

Exercise 1: Mode switching
  :m<TAB>pur<TAB>
  :m<TAB>li<TAB>
  :m<TAB>ex<TAB>
  :m<TAB>no<TAB>

Exercise 2: File loading
  :l<TAB>ex<TAB>sam<TAB>ba<TAB>
  :l<TAB>ex<TAB>sam<TAB>zsh<TAB>
  :l<TAB>ex<TAB>sam<TAB>de<TAB>

Exercise 3: Command discovery
  :<TAB><TAB>
  :mode <TAB><TAB>
  :load <TAB><TAB>

Exercise 4: Bash constructs
  (Switch to explain mode first)
  ${var:<TAB><TAB>
  ${var#<TAB><TAB>
  for<TAB>
  if<TAB>

Exercise 5: Speed workflow
  Combine: :m<TAB>pur<TAB> then :l<TAB>ex<TAB> then :rel<TAB>
  Practice until you can do it without thinking!

=================================================================
Key Takeaways:
=================================================================

1. Tab completion saves 40-60% of typing
2. Press Tab liberally after every few characters
3. Press Tab twice to see all options
4. Works for commands, modes, files, and bash constructs
5. Case-insensitive for convenience
6. Context-aware (different suggestions per command)
7. Discover features by pressing Tab Tab
8. Combine with history (↑/↓) for maximum efficiency

Next Steps:
-----------
Try example 10_variables_session.sh to master variable
management and session state!
EOF
