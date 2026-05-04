#!/bin/bash

function eprintln {
  echo "$@" 1>&2
}

ctx_dir="$(dirname "$(realpath "$0")")"
repo_dir="$(dirname "$ctx_dir")"
cd "$repo_dir"

# setup zsh
echo "Setting up zsh..."
. $ctx_dir/setup-zsh.sh || eprintln "Failed to setup zsh."

# install dependencies
echo "Installing project dependencies..."
$ctx_dir/install-deps.sh || eprintln "Failed to install dependencies."

# install devtools
echo "Installing devtools..."
{
  sudo mkdir -p /usr/local/bin
  find "$ctx_dir/devtools" -type f -executable -exec sudo cp --no-clobber {} -t /usr/local/bin/ \;
} || eprintln "Failed to install devtools."

# copy config
if [ ! -f 'config/config.toml' ]; then
  cp config/config.sample.toml config/config.toml || eprintln "Failed to create config/config.toml."
fi
