#!/usr/bin/env python3
import hashlib
import json
import os
import random
import re
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HOST = "0.0.0.0"
PORT = int(os.environ.get("PORT", "8080"))
ROUND_SECS = int(os.environ.get("KOH_ROUND_SECS", "60"))
TOTAL_ROUNDS = int(os.environ.get("KOH_TOTAL_ROUNDS", "0"))
MAX_PROGRAM_BYTES = 16 * 1024

STARTED_AT = time.time()
LOCK = threading.Lock()
SUBMISSIONS = {}
ASSIGNMENT = re.compile(r"^\s*(attack|gather|defend|scout)\s*=\s*(-?\d+)\s*$", re.I)


def now_round():
  return int((time.time() - STARTED_AT) // max(1, ROUND_SECS)) + 1


def completed_round():
  value = now_round() - 1
  if TOTAL_ROUNDS > 0:
    value = min(value, TOTAL_ROUNDS)
  return max(0, value)


def read_json(handler):
  length = int(handler.headers.get("content-length", "0"))
  if length <= 0 or length > 40000:
    raise ValueError("invalid request size")
  return json.loads(handler.rfile.read(length))


def response(handler, status, body):
  raw = json.dumps(body, separators=(",", ":")).encode()
  handler.send_response(status)
  handler.send_header("content-type", "application/json")
  handler.send_header("content-length", str(len(raw)))
  handler.end_headers()
  handler.wfile.write(raw)


def stable_int(*parts):
  digest = hashlib.sha256("|".join(map(str, parts)).encode()).digest()
  return int.from_bytes(digest[:8], "big")


def parse_strategy(program):
  raw = program.encode()
  if not raw or len(raw) > MAX_PROGRAM_BYTES:
    raise ValueError("strategy must be 1..16384 bytes")
  values = {"attack": None, "gather": None, "defend": None, "scout": None}
  for line in program.splitlines():
    match = ASSIGNMENT.match(line)
    if match:
      values[match.group(1).lower()] = max(0, min(100, int(match.group(2))))
  seed = stable_int(program)
  for index, key in enumerate(("attack", "gather", "defend", "scout")):
    if values[key] is None:
      values[key] = 10 + ((seed >> (index * 8)) & 63)
  total = sum(values.values()) or 1
  return {key: values[key] / total for key in values}


def duel(round_id, ida, stra, idb, strb):
  rng = random.Random(stable_int("duel", round_id, min(ida, idb), max(ida, idb)))
  state = {
    ida: {"hp": 100.0, "ore": 0.0, "kills": 0, "strategy": stra},
    idb: {"hp": 100.0, "ore": 0.0, "kills": 0, "strategy": strb},
  }
  for _ in range(80):
    for attacker, defender in ((ida, idb), (idb, ida)):
      a = state[attacker]
      d = state[defender]
      strategy = a["strategy"]
      a["ore"] += 1.0 + 7.5 * strategy["gather"] + 2.0 * strategy["scout"] * rng.random()
      damage = (1.0 + 12.0 * strategy["attack"]) * (1.0 - 0.55 * d["strategy"]["defend"])
      damage *= 0.65 + rng.random() * 0.7
      if a["ore"] >= 8.0:
        damage *= 1.35
        a["ore"] -= 8.0
      d["hp"] -= max(0.1, damage)
      if d["hp"] <= 0:
        a["kills"] += 1
        d["hp"] += 55.0 + 35.0 * d["strategy"]["defend"]
  score_a = state[ida]["kills"] * 10 + max(0.0, state[ida]["hp"]) + state[ida]["ore"] * 0.2
  score_b = state[idb]["kills"] * 10 + max(0.0, state[idb]["hp"]) + state[idb]["ore"] * 0.2
  return score_a, score_b, state[ida]["kills"], state[idb]["kills"]


def rankings_for(round_id):
  round_submissions = {
    identifier: per_round[str(round_id)]
    for identifier, per_round in SUBMISSIONS.items()
    if str(round_id) in per_round
  }
  strategies = {identifier: parse_strategy(program) for identifier, program in round_submissions.items()}
  totals = {
    identifier: {"points": 0, "kills": 0, "base": 0.0, "matches": 0}
    for identifier in round_submissions
  }
  identifiers = sorted(round_submissions)
  if len(identifiers) == 1:
    only = identifiers[0]
    strategy = strategies[only]
    totals[only]["points"] = int(100 * strategy["attack"] + 80 * strategy["gather"] + 60 * strategy["defend"])
    totals[only]["base"] = 100.0
  for i, ida in enumerate(identifiers):
    for idb in identifiers[i + 1:]:
      score_a, score_b, kills_a, kills_b = duel(round_id, ida, strategies[ida], idb, strategies[idb])
      totals[ida]["kills"] += kills_a
      totals[idb]["kills"] += kills_b
      totals[ida]["base"] += score_a
      totals[idb]["base"] += score_b
      totals[ida]["matches"] += 1
      totals[idb]["matches"] += 1
      if score_a > score_b:
        totals[ida]["points"] += 3
      elif score_b > score_a:
        totals[idb]["points"] += 3
      else:
        totals[ida]["points"] += 1
        totals[idb]["points"] += 1
  ordered = sorted(
    identifiers,
    key=lambda identifier: (
      -totals[identifier]["points"],
      -totals[identifier]["kills"],
      -totals[identifier]["base"],
      identifier,
    ),
  )
  return [
    {
      "identifier": identifier,
      "rank": index + 1,
      "metric": int(totals[identifier]["points"]),
      "message": (
        f"points={totals[identifier]['points']} "
        f"kills={totals[identifier]['kills']} "
        f"base={totals[identifier]['base']:.1f}"
      ),
    }
    for index, identifier in enumerate(ordered)
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
      "name": "Minions in 16k",
      "round": now_round(),
      "completed_round": completed_round(),
      "submit": "POST /submit {identifier,program}",
      "limit": MAX_PROGRAM_BYTES,
    })

  def do_POST(self):
    if self.path != "/submit":
      return response(self, 404, {"success": False, "message": "not found"})
    try:
      data = read_json(self)
      identifier = str(data.get("identifier", "")).strip()
      program = str(data.get("program", ""))
      if not identifier.startswith("koh_"):
        raise ValueError("invalid identifier")
      parse_strategy(program)
      round_id = now_round()
      if TOTAL_ROUNDS > 0 and round_id > TOTAL_ROUNDS:
        return response(self, 409, {"success": False, "message": "all rounds completed"})
      with LOCK:
        SUBMISSIONS.setdefault(identifier, {})[str(round_id)] = program
      return response(self, 200, {"success": True, "round": round_id, "bytes": len(program.encode())})
    except Exception as exc:
      return response(self, 400, {"success": False, "message": str(exc)})


if __name__ == "__main__":
  ThreadingHTTPServer((HOST, PORT), Handler).serve_forever()
