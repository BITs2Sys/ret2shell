#!/bin/bash

function do_web {
    echo "[+] Downloading web dependencies..."
    pnpm --prefix=web install
}

function do_rust {
    echo "[+] Downloading rust dependencies..."
    cargo fetch --locked
}

if [ "$1" == "web" ]; then
    do_web
elif [ "$1" == "rust" ]; then
    do_rust
else
    do_web &
    do_rust &
    wait
fi