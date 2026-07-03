#!/usr/bin/env python3
import json
import sys
import urllib.request

if len(sys.argv) != 4:
  print(f"usage: {sys.argv[0]} URL IDENTIFIER STRATEGY_FILE", file=sys.stderr)
  raise SystemExit(2)

url, identifier, path = sys.argv[1:]
with open(path, "r", encoding="utf-8") as handle:
  program = handle.read()
body = json.dumps({"identifier": identifier, "program": program}).encode()
request = urllib.request.Request(
  url.rstrip("/") + "/submit",
  data=body,
  headers={"content-type": "application/json"},
  method="POST",
)
with urllib.request.urlopen(request, timeout=5) as response:
  print(response.read().decode())
