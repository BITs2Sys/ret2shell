#!/bin/bash

function eprintln {
  echo "$@" 1>&2
}

if ! command -v kubectl &> /dev/null; then
  eprintln "kubectl not found"
  exit 1
fi

if [ ! -f config/kubeconfig.yaml ]; then
  echo "Generating kubeconfig to config/kubeconfig.yaml"
  # get kubeconfig
  kubectl config view --raw > config/kubeconfig.yaml
  if [ $? -ne 0 ]; then
    eprintln "Failed to get kubeconfig from kubectl"
    exit 1
  fi
  # modify config
  if [ -f config/config.toml ]; then
    echo "Updating config/config.toml"
    sed -i 's@kube_config_path\s*=\s*".*"@kube_config_path = "config/kubeconfig.yaml"@' config/config.toml
    sed -i '/^\[cluster\]/,/^\[/{s/^\(enabled\s*=\s*\).*/\1true/}' config/config.toml
  fi
else
  echo "kubeconfig already exists at config/kubeconfig.yaml"
fi
