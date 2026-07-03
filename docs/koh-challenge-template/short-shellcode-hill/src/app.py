#!/usr/bin/env python3
import json
import os
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HOST = "0.0.0.0"
PORT = int(os.environ.get("PORT", "8080"))
ROUND_SECS = int(os.environ.get("KOH_ROUND_SECS", "60"))
TOTAL_ROUNDS = int(os.environ.get("KOH_TOTAL_ROUNDS", "0"))
MAX_BYTES = int(os.environ.get("MAX_SHELLCODE_BYTES", "512"))
TARGET_OUTPUT = b"KOH\n"

STARTED_AT = time.time()
LOCK = threading.Lock()
SUBMISSIONS = {}


def now_round():
  return int((time.time() - STARTED_AT) // max(1, ROUND_SECS)) + 1


def completed_round():
  value = now_round() - 1
  if TOTAL_ROUNDS > 0:
    value = min(value, TOTAL_ROUNDS)
  return max(0, value)


def read_json(handler):
  length = int(handler.headers.get("content-length", "0"))
  if length <= 0 or length > 20000:
    raise ValueError("invalid request size")
  return json.loads(handler.rfile.read(length))


def response(handler, status, body):
  raw = json.dumps(body, separators=(",", ":")).encode()
  handler.send_response(status)
  handler.send_header("content-type", "application/json")
  handler.send_header("content-length", str(len(raw)))
  handler.end_headers()
  handler.wfile.write(raw)


def validate_shellcode(shellcode_hex):
  clean = "".join(shellcode_hex.split())
  if len(clean) % 2 != 0:
    return False, "hex shellcode must have even length", None
  try:
    payload = bytes.fromhex(clean)
  except ValueError:
    return False, "shellcode must be hex", None
  if not payload or len(payload) > MAX_BYTES:
    return False, f"shellcode must be 1..{MAX_BYTES} bytes", None
  try:
    run = subprocess.run(
      ["/app/runner", clean],
      stdout=subprocess.PIPE,
      stderr=subprocess.PIPE,
      timeout=2,
      check=False,
    )
  except subprocess.TimeoutExpired:
    return False, "shellcode timed out", None
  if run.stdout != TARGET_OUTPUT:
    return False, f"expected stdout {TARGET_OUTPUT!r}, got {run.stdout!r}", None
  return True, "accepted", len(payload)


def rankings_for(round_id):
  items = []
  for identifier, per_round in SUBMISSIONS.items():
    item = per_round.get(str(round_id))
    if item:
      items.append((identifier, item))
  items.sort(key=lambda row: (row[1]["length"], row[1]["submitted_at"], row[0]))
  return [
    {
      "identifier": identifier,
      "rank": index + 1,
      "metric": item["length"],
      "message": f"{item['length']} bytes",
    }
    for index, (identifier, item) in enumerate(items)
  ]


class Handler(BaseHTTPRequestHandler):
  def log_message(self, fmt, *args):
    return

  def do_GET(self):
    if self.path == "/healthz":
      return response(self, 200, {"ok": True})
    if self.path == "/status":
      round_id = completed_round()
      with LOCK:
        rankings = rankings_for(round_id) if round_id > 0 else []
      return response(self, 200, {"success": True, "data": {"round": round_id, "rankings": rankings}})
    return response(self, 200, {
      "name": "Short Shellcode Hill",
      "round": now_round(),
      "completed_round": completed_round(),
      "submit": "POST /submit {identifier,shellcode}",
    })

  def do_POST(self):
    if self.path != "/submit":
      return response(self, 404, {"success": False, "message": "not found"})
    try:
      data = read_json(self)
      identifier = str(data.get("identifier", "")).strip()
      shellcode = str(data.get("shellcode", "")).strip()
      if not identifier.startswith("koh_"):
        raise ValueError("invalid identifier")
      ok, message, length = validate_shellcode(shellcode)
      if not ok:
        return response(self, 400, {"success": False, "message": message})
      round_id = now_round()
      if TOTAL_ROUNDS > 0 and round_id > TOTAL_ROUNDS:
        return response(self, 409, {"success": False, "message": "all rounds completed"})
      submitted_at = time.time()
      with LOCK:
        per_round = SUBMISSIONS.setdefault(identifier, {})
        previous = per_round.get(str(round_id))
        if previous is None or length < previous["length"]:
          per_round[str(round_id)] = {"length": length, "submitted_at": submitted_at}
      return response(self, 200, {"success": True, "round": round_id, "length": length})
    except Exception as exc:
      return response(self, 400, {"success": False, "message": str(exc)})


if __name__ == "__main__":
  ThreadingHTTPServer((HOST, PORT), Handler).serve_forever()
