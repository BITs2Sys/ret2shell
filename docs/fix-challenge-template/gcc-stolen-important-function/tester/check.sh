#!/bin/sh

R2S_FIX_RESULT=${R2S_FIX_RESULT:-failed}
R2S_FIX_RESULT_ENV=${R2S_FIX_RESULT_ENV:-R2S_FIX_RESULT}

if python3 - <<'PY'
import os
import socket
import sys

host = os.environ.get("R2S_FIX_TARGET_HOST")
port = int(os.environ.get("R2S_FIX_TARGET_PORT", "10001"))
if not host:
    print("missing R2S_FIX_TARGET_HOST", file=sys.stderr)
    sys.exit(2)

def request(payload: str) -> str:
    with socket.create_connection((host, port), timeout=5) as sock:
        sock.settimeout(8)
        sock.sendall(payload.encode())
        try:
            sock.shutdown(socket.SHUT_WR)
        except OSError:
            pass
        chunks = []
        while True:
            try:
                chunk = sock.recv(4096)
            except socket.timeout:
                break
            if not chunk:
                break
            chunks.append(chunk)
    return b"".join(chunks).decode("utf-8", "replace")

probe = request("int x = 0;\n")
if not probe:
    print("target service returned no probe output", file=sys.stderr)
    sys.exit(3)

exploit = r'''__attribute__((section(".text"))) const unsigned char main[] =
"\xb8\x01\x00\x00\x00"
"\xbf\x01\x00\x00\x00"
"\x48\x8d\x35\x0a\x00\x00\x00"
"\xba\x0c\x00\x00\x00"
"\x0f\x05"
"\x31\xc0"
"\xc3"
"Hello World\n";
'''

result = request(exploit)
markers = ("flag{", "Congratulations")
if any(marker in result for marker in markers):
    print("exploit still works", file=sys.stderr)
    print(result[-1000:], file=sys.stderr)
    sys.exit(4)
PY
then
  export R2S_FIX_RESULT=success
else
  export R2S_FIX_RESULT=failed
fi

return 0 2>/dev/null || exit 0
