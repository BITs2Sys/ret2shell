#!/bin/sh
set -eu

UPLOAD=${R2S_FIX_UPLOAD:?missing R2S_FIX_UPLOAD}
ORIGINAL_NAME=${R2S_FIX_ORIGINAL_NAME:-$UPLOAD}
WORKDIR=${R2S_FIX_WORKDIR:-/tmp/ret2shell-fix}
EXTRACT_DIR="$WORKDIR/extracted"
TARGET=/home/ctf/src/main.py

rm -rf "$EXTRACT_DIR"
mkdir -p "$EXTRACT_DIR"

case "$ORIGINAL_NAME" in
  *.tar.gz | *.tgz)
    tar -xzf "$UPLOAD" -C "$EXTRACT_DIR"
    ;;
  *.tar)
    tar -xf "$UPLOAD" -C "$EXTRACT_DIR"
    ;;
  *)
    cp "$UPLOAD" "$EXTRACT_DIR/main.py"
    ;;
esac

PATCHED=$(find "$EXTRACT_DIR" -type f -name main.py | head -n 1)
if [ -z "$PATCHED" ]; then
  echo "main.py not found in uploaded fix artifact" >&2
  exit 1
fi

python3 -m py_compile "$PATCHED"
install -m 0644 -o ctf -g ctf "$PATCHED" "$TARGET"

# This challenge starts /run.sh for each TCP connection through socat, so future
# connections automatically use the replaced source file.
if command -v timeout >/dev/null 2>&1; then
  printf 'int x = 0;\n' | timeout 5 /run.sh >/tmp/ret2shell-fix-smoke.log 2>&1 || true
fi
