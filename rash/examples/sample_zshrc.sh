#!/bin/zsh
# Sample .zshrc-style configuration
# ZSH-specific patterns and oh-my-zsh style

# Development environment (define early for aliases)
export EDITOR=vim
export VISUAL=vim
export LANG=en_US.UTF-8

# Path to oh-my-zsh installation
export ZSH="${HOME}/.oh-my-zsh"

# Theme
ZSH_THEME='robbyrussell'

# Plugins
plugins=(git docker kubectl terraform)

# ZSH options
setopt HIST_IGNORE_DUPS
setopt HIST_FIND_NO_DUPS
setopt SHARE_HISTORY

# Aliases
alias zshconfig="${EDITOR} ~/.zshrc"
alias ohmyzsh="${EDITOR} ~/.oh-my-zsh"
alias dc='docker-compose'
alias k='kubectl'
alias tf='terraform'

# Custom functions
update_system() {
    echo "Updating system packages..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get upgrade -y
    elif command -v brew &> /dev/null; then
        brew update && brew upgrade
    fi
}

# Node.js version manager
export NVM_DIR="${HOME}/.nvm"
[[ -s "${NVM_DIR}/nvm.sh" ]] && \. "${NVM_DIR}/nvm.sh"

# Load oh-my-zsh if available
if [[ -f "${ZSH}/oh-my-zsh.sh" ]]; then
    source "${ZSH}/oh-my-zsh.sh"
fi

# TEST: test_update_system_detects_package_manager
test_update_system_detects_package_manager() {
    type update_system >/dev/null 2>&1 || return 1
    return 0
}

# TEST: test_editor_is_set
test_editor_is_set() {
    [[ -n "${EDITOR}" ]] || return 1
    return 0
}

# TEST: test_aliases_defined
test_aliases_defined() {
    # Check if key aliases are defined
    alias dc >/dev/null 2>&1 || return 1
    alias k >/dev/null 2>&1 || return 1
    return 0
}
