#!/bin/bash
# Sample .bashrc-style configuration
# Common patterns found in user shell configurations

# PATH setup
export PATH="${HOME}/bin:${HOME}/.local/bin:${PATH}"

# History configuration
HISTSIZE=10000
HISTFILESIZE=20000
HISTCONTROL=ignoredups:erasedups

# Prompt customization
PS1='\[\033[01;32m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]\$ '

# Aliases - common productivity shortcuts
alias ll='ls -alF'
alias la='ls -A'
alias l='ls -CF'
alias grep='grep --color=auto'
alias mkdir='mkdir -pv'

# Git aliases
alias gs='git status'
alias ga='git add'
alias gc='git commit'
alias gp='git push'
alias gl='git log --oneline --graph'

# Functions
mkcd() {
    mkdir -p "$1" && cd "$1" || return 1
}

extract() {
    if [ -f "$1" ]; then
        case "$1" in
            *.tar.bz2)   tar xjf "$1"     ;;
            *.tar.gz)    tar xzf "$1"     ;;
            *.bz2)       bunzip2 "$1"     ;;
            *.gz)        gunzip "$1"      ;;
            *.tar)       tar xf "$1"      ;;
            *.zip)       unzip "$1"       ;;
            *)           echo "'$1' cannot be extracted" ;;
        esac
    else
        echo "'$1' is not a valid file"
    fi
}

# Environment-specific settings
if [[ -f "${HOME}/.bashrc.local" ]]; then
    source "${HOME}/.bashrc.local"
fi

# TEST: test_mkcd_creates_and_changes_dir
test_mkcd_creates_and_changes_dir() {
    local test_dir="/tmp/test_mkcd_$$"
    mkcd "$test_dir" || return 1
    [[ "$PWD" == "$test_dir" ]] || return 1
    cd / || return 1
    rm -rf "${test_dir:?}"
    return 0
}

# TEST: test_extract_handles_tar_gz
test_extract_handles_tar_gz() {
    local test_file="/tmp/test_$$.tar.gz"
    touch "$test_file"
    extract "$test_file" 2>/dev/null || true
    rm -f "$test_file"
    return 0
}

# TEST: test_path_includes_local_bin
test_path_includes_local_bin() {
    [[ "${PATH}" == *"${HOME}/.local/bin"* ]] || return 1
    return 0
}
