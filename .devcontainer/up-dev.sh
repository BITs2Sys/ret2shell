#!/bin/bash

function update_env {
  local env_file="$HOME/.ret2shell/.env.development"
  mkdir -p "$(dirname "$env_file")"
  key="$1"
  value="$2"
  if grep -q "^$key=" "$env_file" 2>/dev/null; then
    sed -i "s/^$key=.*/$key=$value/" "$env_file"
  else
    if [ -s "$env_file" ] && [ "$(tail -c1 "$env_file")" != "" ]; then
      echo "" >> "$env_file"
    fi
    echo "$key=$value" >> "$env_file"
  fi
}

ctx_dir="$(dirname "$(realpath "$0")")"
repo_dir="$(dirname "$ctx_dir")"
cd "$repo_dir"

cd deploy/compose
update_env "DOCKER_SVC_NAME" "$(grep -oP '^name:\s*\K\S+' docker-compose.dev.yml)"

source .env
docker compose -f docker-compose.dev.yml up -d --no-deps
