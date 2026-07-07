# ISW (Internal Security Warfare) — Design & Implementation Spec

> BITs2CTF fork of ret2shell. A distributed VMware-Workstation internal-pentest **range mode**:
> teams pentest an isolated multi-VM range and submit dynamic flags injected into the guests.

## 0. Locked decisions & chosen defaults

| # | Decision | Choice |
|---|----------|--------|
| 1 | Game model | **Attack-only flag hunt** — reuse existing submission + first-blood + dynamic-decay scoring |
| 2 | Flag scope | **Per-range, shared by the 5-team group**; distinct across the 4 groups |
| 3 | Team access | **Platform-brokered VPN** (WireGuard/OpenVPN) into each range's isolated vmnet |
| 4 | Host OS | **Mixed Windows + Linux**; agent cross-compiled, `vmrun`-path autodetect |
| D1 | Flag rotation | **Static per game** by default; rotation scheduler built but opt-in |
| D2 | Reset policy | **Manual admin reset + optional scheduled reset**; no auto-on-compromise (capture is unwired) |
| D3 | Flag→challenge | **One platform challenge per flag/target**; a *range template* groups challenges + topology |

**Topology (example):** 20 teams → 4 groups of 5 → 4 ranges; each range = 2 Linux + 3 Windows VMs on an isolated vmnet; the 4 ranges live on 4 separate physical hosts (RAM limit).

---

## 1. Why a new subsystem (not a k8s-style challenge kind)

ISW differs fundamentally from Fix/KoH:
- The backend is **VMware Workstation on distributed physical hosts**, not the single k8s cluster. `crates/cluster` is hard-wired to `kube::Client`; there's no backend trait, and its `ChallengeEnvSnapshot{pod, service}` currency doesn't fit VMs. So ISW gets its **own provisioning path**, not a `cluster` extension.
- Ranges are **long-lived shared infra** keyed to a *group of teams*, not ephemeral per-user pods.
- But **flags and scoring reuse the existing machinery**: a range's flags are minted by ordinary challenges' `environ()` and verified by their `check()`; submissions/first-blood/decay are untouched.

So ISW = **new fork-owned crate(s) + a per-host agent binary + a thin manifest that binds a challenge's flag to a guest file** — mirroring the koh seams, with minimal core edits.

---

## 2. Architecture

```
                         ret2shell platform (server)
   ┌───────────────────────────────────────────────────────────────┐
   │ crates/isw (new)                                               │
   │  • IswManager: host registry, HTTP/mTLS clients, scheduler     │
   │  • FlagService: environ()->mint per-range flags, inject, verify│
   │  • DB entities: isw_host / isw_range_template / isw_range /     │
   │    isw_vm / isw_flag / isw_assignment / isw_vpn_peer           │
   │  routes/range (admin, DevOps-gated) + routes/game/.../range    │
   │  worker/isw: arm/reset/health background jobs (NATS + ticker)  │
   └───────────────┬───────────────────────────────────────────────┘
                   │ HTTP/JSON + mTLS  (typed ops; guest creds NEVER on the wire)
   ┌───────────────┴───────────────┐   ┌───────────────┐   ┌───────────────┐
   │ Host A (Windows)              │   │ Host B (Linux)│   │ Host C/D ...  │
   │ r2s-isw-agent (new binary)    │   │ r2s-isw-agent │   │ r2s-isw-agent │
   │  • vmrun autodetect + queue   │   │               │   │               │
   │  • guest creds (local secret) │   │               │   │               │
   │  • WireGuard endpoint (opt.)  │   │               │   │               │
   │  Range 1: dc01,web01,... (vmnet, isolated)                        ... │
   └───────────────────────────────┘   └───────────────┘   └───────────────┘
                   │ WireGuard/OpenVPN per group
             team clients (group 1 → range 1, group 2 → range 2, ...)
```

**`vmrun` is the control plane; `vmrest` is a localhost-only helper.** `vmrun` (VIX) is the only
interface that does snapshots and guest file/command injection; `vmrest` can't do either (only
power/NIC/IP). The agent shells out to `vmrun` locally so guest credentials stay on the host.

---

## 3. Host-agent: `r2s-isw-agent`

A single small Rust binary (its own crate `crates/isw-agent`, or a `[[bin]]` in `crates/isw`),
cross-compiled for `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`.

> **Transport: HTTP/JSON over mTLS** using `axum` (agent server) + `reqwest` (platform client) —
> both already in the workspace, so **zero new dependencies** (honors the AGENTS.md no-new-deps
> rule). No gRPC/`tonic`/`protoc`. mTLS via rustls with a fork-managed CA; the platform pins each
> host's client-cert fingerprint (`isw_host.fingerprint`); a shared bearer token per request for
> RBAC.

**Responsibilities**
- Expose an **authenticated HTTP/JSON API over mTLS** to the platform; per-request bearer token for RBAC; whitelisted operations only.
- **Autodetect `vmrun`**: Windows `C:\Program Files (x86)\VMware\VMware Workstation\vmrun.exe` (+ registry lookup); Linux `/usr/bin/vmrun`. Configurable override.
- Own a **VM registry**: logical `range/vm` → absolute `.vmx` path + guest OS + guest creds, loaded from a local secret file (creds never sent by the platform per-call).
- **Serialize ops per VM** (VIX dislikes concurrent ops on one VM) with a global concurrency cap (RAM-constrained hosts).
- **Heartbeat** free-RAM / vmrun-up / per-VM power+tools state → feeds the platform scheduler.

**HTTP/JSON API (sketch)** — all under mTLS, `Authorization: Bearer <token>`:
```
GET  /v1/health                 -> { free_mem, vmrun_ok, vms:[{name,power,tools}] }   // heartbeat
GET  /v1/vms                     -> [{ logical_name, vmx, power_state, tools_state }]
POST /v1/vms/{vm}/power          { op: start|stop_soft|stop_hard|reset|suspend }
POST /v1/vms/{vm}/snapshot       { name }
POST /v1/vms/{vm}/revert         { name }
POST /v1/vms/{vm}/inject         { guest_path, content_b64, owner, mode }  -> { ok, sha256, injected_at }
POST /v1/vms/{vm}/run            { interpreter, script | program+args }    -> { exit_code, stdout_b64 }
GET  /v1/vms/{vm}/ip             -> { ip }
POST /v1/vms/{vm}/verify         { guest_path }                           -> { exists, sha256 }
```
Modeled as an axum `Router` on the agent; the platform calls them with a `reqwest::Client` built
from the fork CA + client cert. `inject`/`run` capture guest output via file read-back (VIX gives no
stdout). Guest creds are resolved **inside the agent** from its local secret store, never sent by
the platform.

**Injection primitive (`InjectFile`)** — the load-bearing op:
1. Stage: write the flag string to a host temp file (unique per range/vm/target).
2. Push: `vmrun -T ws -gu <u> -gp <p> copyFileFromHostToGuest "<vmx>" <hostTemp> <guestAbsPath>` (requires VMware Tools running + valid guest login).
3. Permissions:
   - Linux: `runScriptInGuest "<vmx>" "/bin/bash" "chown <owner> '<path>' && chmod <mode> '<path>'"`.
   - Windows: prefer a **non-system dir** the service user owns (e.g. `C:\flags\`) to dodge UAC token filtering; set ACLs via `runProgramInGuest cmd /c "icacls <path> /inheritance:r /grant <user>:(R)"`. Only write to `C:\Windows\...` if the gold image runs as built-in Administrator with UAC lowered.
4. **Verify (mandatory):** `runScriptInGuest` returns no stdout, so verify by `fileExistsInGuest` + `copyFileFromGuestToHost` read-back → agent hashes and compares to the intended flag. Return `InjectResult{ok, sha256, injected_at}`. Branch on the `vmrun` process exit code AND parse the `Guest program exited with non-zero exit code` string.

**Preconditions the arm step asserts before injecting:** VM powered on; `checkToolsState`=running (poll/backoff — Tools come up seconds after boot); guest login valid. Fail loudly rather than leave a stale flag.

---

## 4. Data model (new SeaORM entities)

New fork-owned entities under `crates/database/src/entities/isw/` (append-only registration, per
[fixkoh-optimization.md](fixkoh-optimization.md) A2). One migration
`m_20260706_000001_create_isw` appended at the tail of the migrator (KoH migration is the template).

| Entity | Key fields | Purpose |
|--------|-----------|---------|
| `isw_host` | id, name, address, grpc_port, fingerprint (mTLS pin), os (`win`/`linux`), enabled, last_heartbeat, free_mem, status | A physical VMware host running the agent |
| `isw_range_template` | id, game_id, name, brief, topology (JSONB: VMs + guest OS + creds-ref + vmnet + injection targets), vpn (JSONB) | Reusable range blueprint authored by admin |
| `isw_range` | id, template_id, host_id, group_index, name, status (`provisioning`/`armed`/`error`/`down`), armed_at, snapshot_name | A concrete range instance placed on a host |
| `isw_vm` | id, range_id, logical_name, vmx_path, guest_os, ip, power_state, tools_state | One guest VM in a range |
| `isw_flag` | id, range_id, challenge_id, vm_id, guest_path, value_hash, injected_at, verified, round | A minted+injected flag bound to a challenge |
| `isw_assignment` | id, range_id, team_id (or group_index), game_id | Maps a team/group to its range (the 5→1 mapping) |
| `isw_vpn_peer` | id, range_id, team_id, public_key, address, config_ref, revoked | Per-team VPN credential into the range |

Notes:
- `isw_range_template.game_id`, `isw_flag.challenge_id`, `isw_assignment.team_id` FK to existing
  core tables (Cascade/SetNull like `koh_*`). No core table is ALTERed.
- **No new `TeamScoreHistoryKind`** — ISW flags score as **ordinary submissions** (see §7), so the
  scoreboard path is 100% reused and there's zero new coupling to core scoring.

---

## 5. Config manifests

Two levels, both TOML, both fork-owned types in `crates/config/src/cluster.rs` (parallel to
`KohConfig`), with `set_isw/isw/delete_isw` bucket accessors in `crates/bucket/src/challenge.rs`.

### 5a. `isw.toml` — per-challenge flag-target binding (the manifest "kind")
Lives in a challenge bucket, exactly like `koh.toml`. Binds this challenge's dynamic flag to a
guest injection target:
```toml
enabled = true
range_template = "corp-lan"     # which range template this challenge belongs to
vm = "web01"                    # logical VM name within the range
guest_path = "/var/www/html/flag.txt"   # absolute path inside the guest
owner = "www-data:www-data"     # Linux owner (or Windows user for icacls)
mode = "0640"                   # perms
rotate = false                  # per-round rotation (default off, D1)
# The flag VALUE + verification come from this challenge's checker/main.rx:
#   environ(bucket, user, team) -> { "FLAG": <per-range flag> }   (seed by range/group, not user)
#   check(...) verifies the submitted flag as usual.
```
The challenge's `environ`/`check` scripts are unchanged in spirit — `environ` mints the per-range
flag (seeded by the range/group identifier so all 5 teams in a group get the same value, distinct
across groups — decision #2), and `check` verifies submissions normally.

### 5b. Range template — topology (admin-authored, stored in `isw_range_template.topology`)
Not a bucket file (topology is infra, not per-challenge). Authored via the admin UI / a
game-level `range.toml`, e.g.:
```toml
name = "corp-lan"
vmnet = "vmnet12"               # isolated host-only network / LAN segment
[[vm]]
logical_name = "web01"
guest_os = "linux"
vmx = "web01/web01.vmx"          # relative to the host's range root; agent resolves absolute
creds_ref = "web01"             # key into the host agent's local secret store
[[vm]]
logical_name = "dc01"
guest_os = "windows"
vmx = "dc01/dc01.vmx"
creds_ref = "dc01"
[vpn]
kind = "wireguard"
server_endpoint = "hostA:51820"
subnet = "10.50.<group>.0/24"
```

---

## 5c. Flag verification: hash lookup, not `environ` (refined during implementation)

The Phase-1 design assumed per-team flags minted by the rune `environ()` seam. Because
decision #2 makes flags **per-range (shared by the group)**, verification can't use the normal
per-team `check()` (it would need the submitting team's range). The simpler, implemented approach:

- **Mint** a random flag per `(range, challenge)` at arm time (`flag{<nanoid>}`); store **only its
  sha256** in `isw_flag.value_hash` (plaintext never hits the DB); inject the plaintext into the guest.
- **Verify** a submission by resolving the submitting team → `isw_assignment` → range →
  `isw_flag(range, challenge)` and comparing `sha256(submission) == value_hash`. No rune checker,
  no `environ`. Distinct-per-group and anti-cross-group-share fall out for free.

Implemented in `crates/server/src/utility/isw.rs` (`arm_range`, `verify_submission`); the submission
worker branches to it when `isw.toml` is enabled (`crates/server/src/worker/game.rs`).

## 6. Flag pipeline (mint → inject → verify → score)

1. **Assignment:** admin defines the range template + assigns groups→ranges→hosts (`isw_assignment`, `isw_range.host_id`, `group_index`). Scheduler places each range on a host with free RAM (heartbeat).
2. **Mint:** at arm time, `FlagService` enumerates the game's challenges that have `isw.toml`, and for each range calls the challenge's `checker.environ(bucket, user, team)` — **seeded by the range/group identifier** (a synthetic "team" = the group), so the value is per-range/per-group and reproducible.
3. **Inject:** for each `isw.toml` target, `InjectFile` to `{vm, guest_path, owner, mode}` on the range's host; store `isw_flag{value_hash, guest_path, injected_at, verified}`.
4. **Verify:** read-back hash compare; mark `isw_flag.verified`; surface per-flag status in the admin UI.
5. **Score:** teams submit found flags through the **normal** `POST /submit` flow. The challenge's `check()` verifies against the same `environ`-derived value. First-blood/decay/scoreboard are entirely reused → **no new scoring code**. (Optional anti-share: since flags are per-group, a team submitting another group's flag simply fails `check` — built-in.)
6. **Rotate (opt-in, D1):** if `rotate=true`, the round scheduler re-seeds `environ` with `(group, round)`, re-injects, re-verifies, and (optionally) invalidates prior-round submissions server-side.

---

## 7. Scoring integration (attack-only)

- Each injected flag = one ordinary challenge (D3). Its `checker/main.rx` `check()` already verifies
  the flag; submissions go through `submission_worker` unchanged.
- The ISW layer only needs to ensure the **same value** is what `environ` injected and what `check`
  expects — both derive from the same `(challenge, group[, round])` seed. This is the single
  invariant to keep.
- Result: the scoreboard, first-blood, dynamic-decay, audit/cheat-detection all work with **zero**
  changes to `crates/server/src/worker/game.rs` scoring.

---

## 8. VPN & network isolation (decision #3)

- Each range's VMs sit on a **dedicated isolated host-only vmnet / LAN Segment** (no bridge to the
  physical LAN or Internet). Distinct vmnet/subnet per range → groups can't see each other even on
  the same physical host; cross-host isolation is inherent (separate machines).
- **Team access:** platform brokers **WireGuard** (recommended) or OpenVPN. The host-agent runs a
  VPN endpoint whose peer network routes only into that range's vmnet. Per team, the platform
  generates a peer key (`isw_vpn_peer`), pushes it to the agent, and serves the team a downloadable
  config (player UI). Group→range mapping enforced by which range's VPN a team is issued.
- No wsrx/k8s-portforward needed (that's for the container backend); VPN gives full L3 access for
  real exploitation tooling.

---

## 9. Lifecycle

```
provision → arm → (in-game: health / reset / rotate) → teardown
```
- **Provision:** admin registers hosts (agent installed), authors range template, uploads/places gold-image VMs on hosts, assigns groups→ranges. Agent takes a `clean-armed` snapshot per VM after the baseline is ready.
- **Arm:** for each range: power on VMs (`start nogui`) → wait Tools → mint + inject + verify flags → mark `isw_range.status=armed` → issue VPN peers. (If `rotate`, this is also the per-round op.)
- **In-game:**
  - *Health:* `worker/isw` polls heartbeats + `GetGuestIP`/tools-state; surfaces down VMs.
  - *Reset (D2):* admin button `POST /range/{id}/reset` → `revertToSnapshot clean-armed` → boot → wait Tools → **re-inject** (revert restores the OLD flag) → re-verify. Optional scheduled per-round reset. No auto-on-compromise.
  - *Rotate (opt-in):* re-seed → re-inject → re-verify.
- **Teardown:** power off, optionally delete snapshots, revoke VPN peers.

---

## 10. Platform integration points (kept modular)

**New (fork-owned):**
- `crates/isw/` — manager, flag service, gRPC client, scheduler. (+ `crates/isw-agent/` or a `[[bin]]`.)
- `crates/database/src/entities/isw/` — the 7 entities (registered via one EOF `pub mod isw;`).
- `crates/migrator/src/migrations/m_20260706_000001_create_isw.rs` — appended at the vec tail.
- `crates/server/src/routes/range/` — admin router (DevOps-gated) + a player sub-router nested under `/game/{game}`.
- `crates/server/src/worker/isw.rs` — arm/reset/health/rotate background jobs.
- `crates/server/src/routes/game/challenge/isw.rs` — `isw.toml` get/update/delete (mirror `koh.rs`).
- `crates/config/src/cluster.rs` — add `IswConfig` (+ defaults/desensitize).
- `crates/bucket/src/challenge.rs` — `set_isw/isw/delete_isw` triad.
- `web/src/lib/blocks/challenge/isw.tsx`, `web/src/lib/api/isw.ts`, `web/src/lib/models/isw.ts`, admin range-management route, i18n overlay (`challenge.isw.*`, `range.*`).

**Core edits (minimal — these are the only upstream files touched):**
- `crates/server/src/traits.rs` — one `GlobalState.isw: IswManager` field.
- `crates/server/src/lib.rs` — load `isw` in `up()` + spawn `worker::isw`.
- `crates/server/src/routes/mod.rs` — one `.nest("/range", range::router(state))` + `data-range-*` span field.
- `crates/database/src/entities/user.rs` — one appended `Permission::Range` variant (or reuse `DevOps`).
- `crates/server/src/routes/game/challenge/{mod.rs, submission.rs, repo_sync.rs}` — register the `isw` manifest routes + the `isw().enabled` submit guard + the `isw.toml` repo-sync classification (don't repeat the fix/koh omission, C1).
- Frontend `web/src/lib/blocks/challenge/index.tsx` — via the `fork-tabs.tsx` registry from A7 (so this file stays a one-line spread).

Everything else is new files upstream never sees.

---

## 11. Admin & player UX

**Admin (DevOps/Range permission):**
- Hosts page: register/enroll agents (paste address + mTLS fingerprint), see heartbeat/free-RAM/VM states.
- Range templates: author topology (VMs, guest OS, creds-ref, vmnet, VPN, injection targets) — a form mirroring the koh/fix editor style (Card+Input+Select).
- Ranges: create from template, place on host (scheduler suggests), assign groups→ranges, **Arm / Reset / Rotate / Teardown** buttons, per-flag injection status (verified ✓ / failed ✗), per-VM power/tools/IP.
- Per-challenge `isw.tsx` tab: edit `isw.toml` (range_template, vm, guest_path, owner, mode, rotate).

**Player (team):**
- "My range" panel in `intro.tsx` (gated `isw.data?.config?.enabled`, using the declarative
  `providesSharedTarget` helper from A7): shows the range's entry point, **downloadable VPN config**,
  target list (VM names/IPs the team may attack), and the challenges (flags) to hunt. Flag submission
  uses the normal submit box.

---

## 12. Distributed deployment (4 hosts, mixed OS)

- Install `r2s-isw-agent` on each of the 4 hosts (Windows service / systemd unit). Each gets an mTLS
  server cert (signed by the fork CA) + a local secret file with that host's guest creds and
  vmnet/VPN config, and a shared bearer token.
- Platform holds the CA; pins each host's cert fingerprint (`isw_host.fingerprint`).
- Scheduler places a range on a host by free-RAM heartbeat; with 4 ranges / 4 hosts it's 1:1, but the
  design supports N:1 and rebalancing.
- Gold images: VMware Tools running, known injection account (built-in Administrator on Windows or a
  service user + non-system flag dir), UAC handled, `clean-armed` snapshot taken.

---

## 13. Security considerations

- Guest credentials **never** leave the host (resolved inside the agent). Flags injected as file
  copies, never on a command line (avoids leaking into guest process/audit logs).
- HTTP/JSON over mTLS + per-request bearer token + operation whitelist; bind `vmrest` (if used) to localhost only.
- Per-group VPN isolation; ranges on distinct isolated vmnets; no bridge to platform LAN.
- Anti-share is intrinsic: per-group flags mean a stolen cross-group flag fails `check`.
- Note: **packet capture is unimplemented upstream** (`enable_capture` is dead config) — don't build
  anti-cheat that assumes it. If needed, capture must be added separately (e.g. tcpdump on the vmnet
  or a mirror VM), out of scope here.

---

## 14. Build roadmap (phased)

> **Status (2026-07-06): COMPLETE + verified** (full `cargo clippy --workspace` clean + full
> `pnpm build` exit 0). Phase 0 ✅ (bugs B1–B7, fixes C1/C4/C5/C6, refactors A1–A8). ISW Phases
> 1 ✅ (entities/config/skeleton), 2 ✅ (mTLS: agent tokio-rustls + platform reqwest-rustls Identity),
> 3 ✅ (flag pipeline: mint→inject→hash-verify + admin CRUD + arm), 4 ✅ (WireGuard peer provisioning
> + player VPN download), 5 ✅ (snapshot/reset lifecycle). Frontend ✅ (per-challenge isw.tsx panel +
> admin range page + models/api + i18n).
>
> **Only remaining work is physical-infra integration testing** (not code): a cert-based mTLS
> handshake, a real WireGuard tunnel, real VMs for end-to-end vmrun inject/verify, and the
> auto-on-boot migration apply against the live Postgres. Round rotation + a host heartbeat/scheduler
> worker are optional future enhancements.


- **Phase 0 — Fix/KoH upstream-proofing + bug fixes** (see [fixkoh-optimization.md](fixkoh-optimization.md)): safe, high-value, no ISW dependency. Good warm-up that also establishes the fork-owned module/overlay patterns ISW reuses (`entities/koh/`, `worker/fix.rs`, `fork-tabs.tsx`, i18n overlays, migrator chain).
- **Phase 1 — ISW data + manifest skeleton:** entities + migration; `IswConfig` + bucket triad; `crates/isw` crate skeleton + `GlobalState` field; DevOps `/range` router stub; frontend model/api/tab stubs. Compiles + migrates, no VMware yet.
- **Phase 2 — Host-agent:** `r2s-isw-agent` with gRPC/mTLS, `vmrun` autodetect + per-VM queue, `PowerOp`/`Snapshot`/`Revert`/`InjectFile`/`RunCommand`/`GetGuestIP`/`VerifyFile`. Test against one real VM per OS.
- **Phase 3 — Flag pipeline + arm/reset:** `FlagService` (environ→mint→inject→verify), `worker/isw` arm/reset jobs, admin range CRUD + Arm/Reset buttons, per-flag status.
- **Phase 4 — VPN + player UX:** WireGuard peer provisioning, "my range" panel, VPN config download.
- **Phase 5 — Rotation, scheduler polish, health, docs, hardening.**

---

## 15. Open items (defaults chosen — override if needed)

- Flag rotation cadence (default static; opt-in per-round) — confirm if you want rounds.
- Reset policy (default manual + optional scheduled) — confirm auto-reset appetite.
- VPN tech (default WireGuard) vs OpenVPN — confirm which you already run.
- Whether each flag is a separate visible challenge (D3) or grouped under one "range" challenge with
  multiple sub-flags — default separate challenges (fits the scoreboard model best).
- Gold-image ownership: who builds the 5 VMs per range (Tools + injection account + snapshot)?
