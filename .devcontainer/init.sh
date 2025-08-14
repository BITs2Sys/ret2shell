#!/bin/bash

function init_web {
    pnpm --prefix=web install
}

function init_rust {
    cargo update
}

# test arg 1
if [ "$1" == "web" ]; then
    init_web
elif [ "$1" == "rust" ]; then
    init_rust
else
    echo "[+] Init web" && \
    init_web && \
    echo "[+] Init rust" && \
    init_rust
fi