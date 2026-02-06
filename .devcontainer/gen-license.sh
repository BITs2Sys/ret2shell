#!/bin/bash

if [ -f ./config/license ]; then
    echo "License already exists, skipping generation."
    exit 0
fi

if [ ! -f ./config/priv.bin ]; then
  echo "Initializing license CA..."
  cargo run --bin r2s-license -- init -p config || { echo "Failed to initialize license CA." >&2; exit 1; }
else
  echo "Private key already exists, skipping CA initialization."
fi

echo "Generating license..."
cargo run --bin r2s-license -- new --ca ./config/priv.bin --path ./config/ --issuer Developer --website localhost --level enterprise --date "$(date -d "+36500 days" +"%Y-%m-%d")"
