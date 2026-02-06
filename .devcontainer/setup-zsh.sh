#!/bin/bash

sudo mkdir -p /usr/share/

if [ ! -d "/usr/share/zsh-syntax-highlighting" ]; then
    sudo git clone https://github.com/zsh-users/zsh-syntax-highlighting.git /usr/share/zsh-syntax-highlighting
    echo "source /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh" >> ${ZDOTDIR:-$HOME}/.zshrc
fi

if [ ! -d "/usr/share/zsh-autosuggestions" ]; then
    sudo git clone https://github.com/zsh-users/zsh-autosuggestions.git /usr/share/zsh-autosuggestions
    echo "source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh" >> ${ZDOTDIR:-$HOME}/.zshrc
fi

function add_zsh_plugin() {
    local plugin_name="$1"
    if ! grep -q "plugins=(.*${plugin_name}.*)" ${ZDOTDIR:-$HOME}/.zshrc; then
        sed -i "s/plugins=(/plugins=(${plugin_name} /g" ${ZDOTDIR:-$HOME}/.zshrc
    fi
}

add_zsh_plugin "git"
add_zsh_plugin "docker"

echo "To apply for application, please run:"
echo "  source ${ZDOTDIR:-$HOME}/.zshrc"

# source ${ZDOTDIR:-$HOME}/.zshrc
