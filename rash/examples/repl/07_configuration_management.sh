#!/bin/bash
# REPL Example 07: Configuration File Management
# Demonstrates managing .bashrc, .zshrc, and config files
#
# This example shows:
# - Analyzing shell config files
# - Cleaning up PATH deduplication
# - Organizing aliases
# - Safe config file editing
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 07: Configuration File Management
=================================================================

This example demonstrates how to use the REPL to analyze and
clean up shell configuration files like .bashrc and .zshrc.

SCENARIO: Cleaning up a messy .bashrc
--------------------------------------

STEP 1: Load your configuration file
-------------------------------------
$ bashrs repl

bashrs [normal]> :load ~/.bashrc
✓ Loaded: /home/user/.bashrc (15 functions, 250 lines)

bashrs [normal]> :functions
Available functions (15 total):
  1 parse_git_branch
  2 update_prompt
  3 docker_cleanup
  4 git_commit_all
  5 deploy_staging
  ... (truncated)

STEP 2: Analyze for issues
---------------------------
bashrs [normal]> :mode lint
Switched to lint mode

bashrs [lint]> :load ~/.bashrc
Found 35 issue(s):
  ⚠ 25 warning(s)
  ℹ 10 info

Common issues found:
  [1] CONFIG-001: PATH has duplicate entries
  [2] CONFIG-002: Unquoted variables in exports
  [3] CONFIG-003: Redundant alias definitions
  [4] ℹ Consider using functions instead of complex aliases
  [5] ⚠ Some exports could be consolidated

STEP 3: Analyze specific issues
--------------------------------

# Issue 1: PATH deduplication
bashrs [lint]> export PATH="/usr/local/bin:$PATH"
bashrs [lint]> export PATH="/opt/homebrew/bin:$PATH"
bashrs [lint]> export PATH="$HOME/.cargo/bin:$PATH"

Found issue:
  CONFIG-001: PATH may contain duplicates after sourcing multiple times

# Issue 2: Unquoted variables
bashrs [lint]> export EDITOR=$HOME/.local/bin/nvim
Found issue:
  CONFIG-002: Variable should be quoted

# Issue 3: Redundant aliases
bashrs [lint]> alias ll='ls -la'
bashrs [lint]> alias ll='ls -lah'  # Duplicate!
Found issue:
  CONFIG-003: Alias 'll' redefined

STEP 4: Get purified versions
------------------------------
bashrs [lint]> :mode purify
Switched to purify mode

# Fix PATH deduplication
bashrs [purify]> export PATH="/usr/local/bin:$PATH"
✓ Purified:
# Deduplicated PATH (prevent duplicates)
PATH="/usr/local/bin${PATH:+:$PATH}"
export PATH="$(echo "$PATH" | tr ':' '\n' | awk '!seen[$0]++' | tr '\n' ':')"

# Fix unquoted variables
bashrs [purify]> export EDITOR=$HOME/.local/bin/nvim
✓ Purified:
export EDITOR="$HOME/.local/bin/nvim"

# Fix redundant aliases
bashrs [purify]> # Remove duplicate alias definitions
✓ Suggestion: Keep only the last definition of 'll'

STEP 5: Organize PATH additions
--------------------------------
bashrs [normal]> :mode normal

# Create a clean PATH setup
bashrs [normal]> cat << 'PATHSETUP' > /tmp/path_setup.sh
# PATH Setup (idempotent)
add_to_path() {
  case ":$PATH:" in
    *":$1:"*) ;;  # Already in PATH
    *) PATH="$1:$PATH" ;;
  esac
}

# Add directories to PATH (newest first)
add_to_path "$HOME/.local/bin"
add_to_path "$HOME/.cargo/bin"
add_to_path "/opt/homebrew/bin"
add_to_path "/usr/local/bin"

export PATH

# Remove duplicates
PATH="$(echo "$PATH" | tr ':' '\n' | awk '!seen[$0]++' | tr '\n' ':' | sed 's/:$//')"
PATHSETUP

# Verify it in REPL
bashrs [normal]> :load /tmp/path_setup.sh
✓ Loaded: /tmp/path_setup.sh (1 function, 20 lines)

bashrs [normal]> :mode lint
bashrs [lint]> :load /tmp/path_setup.sh
✓ No issues found!

STEP 6: Organize aliases
-------------------------
bashrs [normal]> cat << 'ALIASES' > /tmp/aliases.sh
# File Operations
alias ll='ls -lah'
alias la='ls -A'
alias l='ls -CF'

# Git Shortcuts
alias gs='git status'
alias gc='git commit'
alias gp='git push'
alias gl='git log --oneline --graph --decorate'

# Docker Shortcuts
alias dps='docker ps'
alias di='docker images'
alias dc='docker-compose'

# Kubernetes Shortcuts
alias k='kubectl'
alias kgp='kubectl get pods'
alias kgs='kubectl get svc'
alias kgd='kubectl get deployments'

# Safety Aliases (with confirmation)
alias rm='rm -i'
alias cp='cp -i'
alias mv='mv -i'
ALIASES

# Verify aliases
bashrs [normal]> :load /tmp/aliases.sh
✓ Loaded: /tmp/aliases.sh (no functions, 25 lines)

bashrs [normal]> :mode lint
bashrs [lint]> :load /tmp/aliases.sh
✓ No issues found!

STEP 7: Organize functions
---------------------------
bashrs [normal]> cat << 'FUNCTIONS' > /tmp/functions.sh
# Useful bash functions

# Git: Show current branch
parse_git_branch() {
  git branch 2>/dev/null | sed -e '/^[^*]/d' -e 's/* \(.*\)/(\1)/'
}

# Docker: Clean up unused containers and images
docker_cleanup() {
  echo "Removing stopped containers..."
  docker container prune -f
  echo "Removing unused images..."
  docker image prune -f
  echo "Cleanup complete!"
}

# Directory: Create and change to directory
mkcd() {
  mkdir -p "$1" && cd "$1"
}

# Extract: Universal extraction function
extract() {
  if [ -f "$1" ]; then
    case "$1" in
      *.tar.bz2)   tar xjf "$1"     ;;
      *.tar.gz)    tar xzf "$1"     ;;
      *.bz2)       bunzip2 "$1"     ;;
      *.rar)       unrar e "$1"     ;;
      *.gz)        gunzip "$1"      ;;
      *.tar)       tar xf "$1"      ;;
      *.tbz2)      tar xjf "$1"     ;;
      *.tgz)       tar xzf "$1"     ;;
      *.zip)       unzip "$1"       ;;
      *.Z)         uncompress "$1"  ;;
      *.7z)        7z x "$1"        ;;
      *)           echo "'$1' cannot be extracted" ;;
    esac
  else
    echo "'$1' is not a valid file"
  fi
}
FUNCTIONS

# Verify functions
bashrs [normal]> :load /tmp/functions.sh
✓ Loaded: /tmp/functions.sh (4 functions, 45 lines)

bashrs [normal]> :functions
Available functions (4 total):
  1 parse_git_branch
  2 docker_cleanup
  3 mkcd
  4 extract

bashrs [normal]> :mode lint
bashrs [lint]> :load /tmp/functions.sh
✓ No issues found!

=================================================================
Complete Reorganized .bashrc:
=================================================================

After testing components in REPL, create clean .bashrc:

$ cat > ~/.bashrc.new << 'BASHRC'
#!/bin/bash
# ~/.bashrc - Clean and organized configuration

# If not running interactively, don't do anything
[[ $- != *i* ]] && return

# History settings
HISTSIZE=10000
HISTFILESIZE=20000
HISTCONTROL=ignoreboth
shopt -s histappend

# Shell options
shopt -s checkwinsize
shopt -s globstar

# PATH Setup
add_to_path() {
  case ":$PATH:" in
    *":$1:"*) ;;
    *) PATH="$1:$PATH" ;;
  esac
}

add_to_path "$HOME/.local/bin"
add_to_path "$HOME/.cargo/bin"
add_to_path "/opt/homebrew/bin"
export PATH

# Environment Variables
export EDITOR="nvim"
export VISUAL="$EDITOR"
export PAGER="less"
export LESS="-R"

# Prompt Configuration
parse_git_branch() {
  git branch 2>/dev/null | sed -e '/^[^*]/d' -e 's/* \(.*\)/(\1)/'
}

PS1='\[\033[01;32m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]$(parse_git_branch)\$ '

# Source additional configurations
[ -f ~/.bash_aliases ] && source ~/.bash_aliases
[ -f ~/.bash_functions ] && source ~/.bash_functions
[ -f ~/.bash_local ] && source ~/.bash_local

# Enable bash completion
if [ -f /etc/bash_completion ]; then
  source /etc/bash_completion
fi
BASHRC

=================================================================
Configuration File Best Practices:
=================================================================

1. Deduplicate PATH
   ✅ Use add_to_path function
   ✅ Remove duplicates with awk

2. Quote all variables
   ✅ export VAR="$VALUE"
   ❌ export VAR=$VALUE

3. Organize by category
   ✅ History settings
   ✅ PATH setup
   ✅ Environment variables
   ✅ Prompt configuration
   ✅ Aliases
   ✅ Functions

4. Modularize large configs
   ✅ ~/.bash_aliases
   ✅ ~/.bash_functions
   ✅ ~/.bash_local (machine-specific)

5. Make operations idempotent
   ✅ PATH additions don't duplicate
   ✅ Sourcing multiple times is safe
   ✅ Aliases don't conflict

6. Test before applying
   ✅ Load in REPL first
   ✅ Lint for issues
   ✅ Purify for safety
   ✅ Then update actual config

=================================================================
Workflow for Config File Cleanup:
=================================================================

1. Backup current config
   $ cp ~/.bashrc ~/.bashrc.backup

2. Load in REPL
   bashrs [normal]> :load ~/.bashrc

3. Analyze issues
   bashrs [normal]> :mode lint
   bashrs [lint]> :load ~/.bashrc

4. Create clean sections
   bashrs [lint]> # Create /tmp/path_setup.sh
   bashrs [lint]> # Create /tmp/aliases.sh
   bashrs [lint]> # Create /tmp/functions.sh

5. Verify each section
   bashrs [lint]> :load /tmp/path_setup.sh
   bashrs [lint]> :load /tmp/aliases.sh
   bashrs [lint]> :load /tmp/functions.sh

6. Combine into new config
   $ cat /tmp/path_setup.sh /tmp/aliases.sh /tmp/functions.sh > ~/.bashrc.new

7. Test new config
   $ bash --rcfile ~/.bashrc.new
   $ # Verify everything works

8. Apply new config
   $ mv ~/.bashrc.new ~/.bashrc

9. Reload
   $ source ~/.bashrc

=================================================================
Key Takeaways:
=================================================================

1. Use REPL to analyze config files before editing
2. Lint mode finds common config file issues
3. Purify mode suggests idempotent alternatives
4. Organize configs by category (PATH, aliases, functions)
5. Test sections individually before combining
6. Make PATH additions idempotent
7. Quote all variable expansions
8. Modularize large configs into separate files

Next Steps:
-----------
Try example 08_multiline_editing.sh to learn advanced
multi-line editing techniques!
EOF
