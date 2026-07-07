# Fix/KoH Optimization & Upstream-Merge-Safety Plan

> BITs2CTF fork of ret2shell. Goal #1: make the Fix and KoH features **modular / minimally
> invasive** so we can keep pulling upstream without painful merge conflicts. Goal #2:
> correctness bugs. Goal #3: consistency with platform conventions.

**Core finding:** The fork is architecturally sound — ~95% additive (new files), and it ALTERs
**zero** upstream DB tables. Merge pain is concentrated in ~12 shared "registry" seams where the
fork splices tokens into lists/enums/unions/route bodies that upstream actively edits. Converting
those splices into **append-only registration points the fork owns** (or isolating them behind a
facade) gets us painless `git pull`.

---

## (A) Modularity / upstream-merge-safety refactors

### A1. [HIGHEST ROI] i18n overlay files instead of splicing upstream locale JSONs
- **Touch-points:** `web/src/lib/i18n/{en-us,ja-jp,zh-cn,zh-tw}.json` (`challenge.fix`, `challenge.koh` subtrees); loader `web/src/lib/i18n/index.ts:6-17`.
- **Why merge-risk:** 4 large JSON insertions at fixed offsets in the highest-churn upstream surface; JSON has no merge-friendly structure → up to 4 conflicts per pull.
- **Refactor:** Pull the fork keys OUT of upstream JSONs into 4 fork-owned overlays (`fix-koh.en-us.json`, …) holding only `challenge.fix`/`challenge.koh`. Deep-merge them into the base dict in `index.ts` after flatten (~10-line loader change). → upstream locale JSONs return pristine, **zero i18n conflicts**.
- **Safe now:** yes.

### A2. Move `koh_*` entities into a fork submodule (append-only registration)
- **Touch-points:** `crates/database/src/entities/mod.rs` (4 `pub mod koh_*`), `crates/database/src/lib.rs` (mid-line `pub use entities::{koh_*}`).
- **Refactor:** Create `crates/database/src/entities/koh/mod.rs` declaring the 4 entities; add ONE end-of-file `pub mod koh;` to the generated `entities/mod.rs`; in `lib.rs` add a separate trailing `pub use entities::koh::*;` instead of editing the shared `pub use` braces. Move the 4 `koh_*.rs` under `entities/koh/`.
- **Safe now:** yes (mechanical).

### A3. Migrator: append via a fork helper, not a spliced vec entry
- **Touch-points:** `crates/migrator/src/migrations/mod.rs`, `crates/migrator/src/lib.rs` (`migrations()` vec).
- **Refactor:** Keep the `pub mod m_2026…_create_koh` (already last by timestamp). Extract fork boxes into `crates/migrator/src/koh_migrations.rs` (`fn koh_migrations() -> Vec<Box<dyn MigrationTrait>>`) and append with `.chain(koh_migrations())` at the tail of the vec. KoH migration is purely additive (CREATE only), so tail-ordering is safe.
- **Caveat:** always keep fork migrations AFTER upstream (the `koh_award.extra_id → extra.id` FK needs `extra` to exist first).
- **Safe now:** yes.

### A4. Centralize KoH scoring behind one `award_points` seam
- **Touch-points:** `crates/database/src/entities/team.rs` (`TeamScoreHistoryKind::Koh`), the `koh_award.extra_id → extra.id` FK, and award sites in `worker/koh.rs` (`process_identifier`, `process_rankings`).
- **Why deepest coupling:** `TeamScoreHistoryKind` is a shared scoring enum; the `Koh` variant is load-bearing (scoreboard integrity) and is the one place KoH can't be cleanly severed from core.
- **Refactor (two-tier):**
  - *Now:* keep the variant but funnel every KoH award through one fork-owned `koh::award_points(txn, team, challenge, tick, score, meta)` encapsulating `extra::create` + `koh_award::create` + `calc_score` + history-push (currently duplicated across two functions). One place to fix if upstream changes the scoring signature.
  - *Later (user/upstream decision):* propose upstream add a generic `TeamScoreHistoryKind::Plugin(String)` (or `source: Option<String>`) so fork score-sources don't each need a core enum variant.
- **Safe now:** the centralization. The `Plugin` variant is an upstream PR.

### A5. Fold KoH/Fix routes into fork-owned nested routers
- **Touch-points:** `crates/server/src/routes/game/challenge/mod.rs` (`mod fix; mod koh;` + ~10 routes woven into `router()`, straddling the `game_admin_required` layer).
- **Refactor:** Have `fix.rs`/`koh.rs` each expose `pub fn admin_router()` and `pub fn player_router()`; in `mod.rs` replace the ~14 interleaved `.route(...)` calls with adjacent `.merge(fix::admin_router())` / `.merge(koh::player_router())` on the correct sides of the auth layer.
- **Caveat needing a decision:** KoH routes straddle the admin-layer boundary — the two-router split (a) is safe now; making auth uniform-in-handlers (b) changes the auth model (see C2).

### A6. Extract `fix_worker` into `worker/fix.rs` + dedup the scoring tail
- **Touch-points:** `crates/server/src/worker/game.rs` (`fix_worker`/`fix_worker_exec`/`run_fix_check`, ~430 inline lines; spawn list).
- **Refactor:** Move Fix worker bodies into a new `crates/server/src/worker/fix.rs` (KoH already has `worker/koh.rs`); shrink `game.rs` spawn list to two adjacent `tokio::spawn(super::fix::spawn(...))`/`super::koh::spawn(...)`. **Also** extract the shared solve→`maintain_score`→blood-extra→`calc_score`→history→event→scoreboard tail (duplicated in `submission_worker_exec` and `fix_worker_exec`) into one `finalize_solve(...)` helper — reduces fork footprint AND kills a copy-paste that will drift when upstream fixes scoring.
- **Safe now:** the move + spawn change. `finalize_solve` touches core `submission_worker_exec` — do deliberately, test the flag path.

### A7. Frontend fork tab-registry
- **Touch-points:** `web/src/lib/blocks/challenge/index.tsx` (`pages` map `fix/koh`, conditional tab buttons, `useChallengeFix/Koh`), `intro.tsx` (`!koh.data?.config?.enabled` guard), `web/src/lib/models/team.ts` (`"koh"` in kind union).
- **Refactor:** Extract fork wiring into `web/src/lib/blocks/challenge/fork-tabs.tsx` exporting `forkPages = { fix, koh }` + `<ForkTabButtons>` + the `useChallengeFix/Koh` calls; in `index.tsx` do `const pages = { ...upstreamPages, ...forkPages }` (one spread) and render `<ForkTabButtons/>` in one slot. Replace `intro.tsx`'s hardcoded `!koh…enabled` with a declarative `providesSharedTarget(challenge)` helper (so ISW can opt in too, see ISW design).
- **Safe now:** yes (the union widening is optional/user decision).

### A8. Relocate frontend model/api blocks
- **Touch-points:** `web/src/lib/models/challenge.ts` (Fix/KoH types), `web/src/lib/api/challenge.ts` (~18 hooks).
- **Refactor:** Move into `web/src/lib/models/fix-koh.ts` and `web/src/lib/api/fix-koh.ts`, re-exported via one `export * from "./fix-koh"` line each.
- **Safe now:** yes.

---

## (B) Correctness bugs

### B1. [HIGH] AgentHttp award idempotency is per-tick-global, not per-team
- `crates/server/src/worker/koh.rs` `process_identifier` uses `koh_award::get_by_tick` (ignores team). If the hill changes owner within one `interval_secs` tick, the new owner earns **nothing** and the event label is wrong. RoundRankHttp correctly uses `get_by_tick_team`.
- **Fix (DECIDED 2026-07-06):** semantics = **the holder at tick-end gets that tick's score**, exactly one award per tick. Each tick evaluation credits whoever currently holds the hill (the end-of-tick owner); if ownership changed mid-tick, the final holder is credited, not the first. Implement idempotency per (challenge, tick) but let a later same-tick check re-point the award to the current holder, and fix the `captured`/`held` event label accordingly.

### B2. [HIGH] Fix attempt cap bypassable + racy
- `routes/game/challenge/fix.rs`: attempts count against the **team** row when in-progress but the **user** row otherwise (two buckets → cap sidesteppable by straddling game states). Plus TOCTOU: count→check→`submission::create` with no transaction → concurrent uploads all pass.
- **Fix:** count attempts over the union of team-or-user rows consistently; wrap count-check-create in a txn with a row lock (or a UNIQUE guard). Confirm accounting unit (team when a team exists).

### B3. [HIGH] `fix_worker` acks unconditionally on crash
- `worker/game.rs` `double_ack`s on all paths; a mid-`run_fix_check` crash loses the message → submission stuck pending forever but still counts against `count_fix_attempts` (player loses an attempt for a platform failure), and the `ret2shell-fix-target-*` pod leaks.
- **Fix:** mirror `submission_worker` at-least-once — only ack after the result is durably written; nack-for-redelivery (with a cap) or write `solved=false,"internal error"` before ack; add a startup sweep for orphaned `ret.sh.cn/fix-*` pods.

### B4. [MED] `GameElo` KoH mode is dead
- `KohEloConfig` defined but never read; `worker/koh.rs` records `unsupported_mode` and awards nothing; `validate_koh_config` doesn't reject it. Silent 0-scoring foot-gun.
- **Fix:** make `validate_koh_config` **reject** `GameElo` with "not implemented" (small, do now); or implement the ELO worker (feature). Frontend option is already disabled.

### B5. [MED] Mid-game interval reconfig corrupts tick idempotency
- `worker/koh.rs` `tick = now / interval_secs`; editing `interval_secs`/`round_secs` mid-game shifts the tick namespace → duplicate/reset awards against the unique `(challenge,tick,team)` index.
- **Fix:** forbid changing `interval_secs`/`round_secs` in `validate_koh_config` once any `koh_award` exists (do now); or store a monotonic round counter in `koh_state` (redesign).

### B6. [MED] Rank score silently rounds to 0 and drops with no event
- `worker/koh.rs` `(reward*percent+50)/100` → e.g. reward=1, percent=40 → 0 → `continue` with no `koh_event`.
- **Fix:** emit a `rank_skipped`/`zero_score` event when `score<=0`; optionally warn in validation when `reward × min(percent)/100 < 0.5`.

### B7. [MED] `koh_state::put` non-atomic upsert races the force-check endpoint
- `entities/koh_state.rs` SELECT-then-write; the admin `POST /koh/check` runs `check_once` concurrently with the 5s worker loop → interleaved awards/state.
- **Fix:** `SELECT … FOR UPDATE` on `koh_state` (PK=challenge_id) at the top of `check_once`, or a per-challenge in-process mutex.

---

## (C) Consistency / convention fixes

- **C1 [MED]** `fix.toml`/`koh.toml` absent from `repo_sync::classify_challenge_change` (`routes/game/repo_sync.rs`) → git-pushed manifest edits are silently ignored (no validation/invalidation, unlike `env.toml`). Add the arms + `validate_*_config`. **Do this and don't repeat it for `isw.toml`.**
- **C2 [MED]** `/koh` PATCH/DELETE sit below the `game_admin_required` layer relying on inner `is_game_admin!`, while Fix's are above — inconsistent + fragile. Move above the layer (coordinate with A5). Keep `GET /koh` player-accessible.
- **C3 [LOW-MED]** `GET /koh` mutates (creates identifier on read) — non-idempotent. Move to `POST /koh/identifier` or make idempotent + documented (touches frontend contract → decision).
- **C4 [LOW]** `FixConfig::desensitize` leaks `result_env`/`success_value` to players (default `R2S_FIX_RESULT=success`) — a potential bypass oracle. Blank them.
- **C5 [LOW]** `KohConfig::desensitize` only nulls `api_key`, exposing `status_url`/ports/elo. Also blank those.
- **C6 [LOW]** `delete_fix`/`delete_koh` (`bucket/src/challenge.rs`) not idempotent (`remove_file` errors if absent) — treat NotFound as success like the getters.
- **C7 [LOW]** KoH `api_key` never injected into the hill pod, but the platform sends `x-api-key` on polls → auth is out-of-band/unenforced. Inject as env var or document (security-posture decision).

---

## Sequencing

- **Do now, zero decisions:** A1, A2, A3, A6 (worker move only), A7 (fork-tabs), A8, C1, C6.
- **Do now, tiny scope decisions:** B3 (redelivery cap), B4a (reject GameElo), B5 (validation guard), B6, B7, C4, C5.
- **Needs a product decision first:** A4 (`Plugin` variant → upstream PR), A5 caveat, A6 `finalize_solve` (touches core), B1 (per-tick semantics), B2 (accounting unit), C2, C3, C7.
- **The one structural call:** whether to invest in a real `ChallengeKind` plugin abstraction (config triad + router + worker + score-source via a trait table). There is none today. **Recommendation:** since ISW is being built as a **separate crate/subsystem** (not a 4th manifest crammed into the same seams), the append-only refactors A1–A8 get ~90% of the benefit at a fraction of the cost — do those, defer the trait registry unless a 5th in-challenge kind appears.
