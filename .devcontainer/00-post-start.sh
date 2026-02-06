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
exec $ctx_dir/install-deps.sh || eprintln "Failed to install dependencies."

# install devtools
echo "Installing devtools..."
find "$ctx_dir/devtools" -type f -executable -exec cp --no-clobber {} -t /usr/local/bin/ \;

# copy config
if [ ! -f 'config/config.toml']; then
  cp config/config.sample.toml config/config.toml
fi

# get kubeconfig.yaml configuration
if command -v kubectl &> /dev/null; then
  if [ ! -f config/kubeconfig.yaml ]; then
    echo "Setting up kubeconfig..."
    {
      kubectl config view --raw > config/kubeconfig.yaml && \
      sed -i 's@kube_config_path\s*=\s*".*"@kube_config_path = "config/kubeconfig.yaml"@' config/config.toml
    } || eprintln "Failed to get kubeconfig."
  fi
else
  echo "kubectl not found, skipping kubeconfig setup."
fi

# pre-generate license
exec $ctx_dir/gen-license.sh || eprintln "Failed to generate license."
