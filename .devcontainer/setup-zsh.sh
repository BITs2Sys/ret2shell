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

echo "To apply for application, please run:"
echo "  source ${ZDOTDIR:-$HOME}/.zshrc"

source ${ZDOTDIR:-$HOME}/.zshrc
