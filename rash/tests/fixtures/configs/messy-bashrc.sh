# ~/.bashrc - Accumulated over 5 years
# This is a realistic example of a messy config

# PATH entries - lots of duplicates!
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="/usr/local/bin:$PATH"

# Environment variables
export EDITOR=vim
export VISUAL=$EDITOR
export PAGER=less

# Aliases
alias ls='ls --color=auto'
alias ll='ls -lah'
alias la='ls -A'
alias l='ls -CF'

# Version managers - expensive evals!
eval "$(rbenv init -)"
eval "$(pyenv init -)"
eval "$(nodenv init -)"

# Non-deterministic constructs
export SESSION_ID=$RANDOM
export BUILD_TAG="build-$(date +%s)"
export TEMP_DIR="/tmp/work-$$"

# Unquoted variables (dangerous!)
export PROJECT_DIR=$HOME/my projects
export BACKUP_DIR=$HOME/backups

# Old Java version that no longer exists
export JAVA_6_HOME=/Library/Java/JavaVirtualMachines/1.6.0.jdk

# Python 2.7 (deprecated)
export PYTHONPATH="/opt/python2.7/lib"

# Old project alias
alias oldproject="cd ~/projects/2015/oldproject"

# Prompt
PS1='\u@\h:\w\$ '

# History settings
export HISTSIZE=10000
export HISTFILESIZE=20000
export HISTCONTROL=ignoredups:ignorespace

# Shell options
shopt -s histappend
shopt -s checkwinsize

# Load additional configs
[ -f ~/.bash_aliases ] && . ~/.bash_aliases
[ -f ~/.bash_functions ] && . ~/.bash_functions

echo "Welcome to $(hostname)!"
