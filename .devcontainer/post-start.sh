#!/bin/bash

ctx_dir="$(dirname "$(realpath "$0")")"

echo "Setting up zsh..."
. $ctx_dir/setup-zsh.sh

echo "Init environment..."
exec $ctx_dir/init.sh web || echo "Failed to init environment, aborting..."
