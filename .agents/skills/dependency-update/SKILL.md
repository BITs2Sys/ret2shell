---
name: dependency-update
description: Non-destructively update project dependencies, identify semver-breaking upgrades, summarize breaking/API changes, and apply approved major-version bumps for the Ret2Shell monorepo.
---

# Dependency Update Skill for Ret2Shell

This skill describes the complete dependency update workflow for the Ret2Shell monorepo, covering both the Rust workspace (`crates/`) and the SolidJS frontend (`web/`).

The workflow is designed to be executed in a temporary directory (e.g. `/tmp` or the OS equivalent) so that the original project is not mutated until the upgrade set has been reviewed and validated.

## When to Use

Use this skill when the user asks to:

- Update dependencies
- Run `cargo update` / `pnpm up`
- Check for outdated dependencies
- Upgrade major versions of Rust or JS packages

## Workflow

### 1. Prepare a Temporary Workspace

To avoid modifying the original repository during exploration, copy or clone the project into a temporary directory first.

```bash
# Example on Linux/macOS
TMP_DIR=$(mktemp -d)
cp -r /path/to/ret2shell "$TMP_DIR/ret2shell-update"
cd "$TMP_DIR/ret2shell-update"
```

All subsequent commands should run inside this temporary copy. After the final upgrade set is approved and verified, apply the same changes to the real repository.

### 2. Apply Non-Breaking Updates

Run the standard non-destructive update commands.

```bash
# Frontend
pnpm -C web up

# Rust
cargo update
```

These commands only update versions that are compatible with the ranges declared in `web/package.json` and `Cargo.toml`.

### 3. Identify Major-Version Candidates

#### Rust

Use the helper script shipped with this skill:

```bash
python3 .agents/skills/dependency-update/check_rust_outdated.py --project-dir .
```

This script reads `Cargo.toml` workspace dependencies, queries crates.io, and writes `outdated_rust_major.json` listing crates whose latest version is a semver-breaking upgrade from the current requirement.

> **Important:** For pre-1.0 crates (e.g. `0.x.y`), a minor bump is treated as breaking.

#### Frontend

Use pnpm's built-in outdated command:

```bash
pnpm -C web outdated --format json
```

This reports packages where the `latest` version exceeds the `wanted` (range-resolved) version, i.e. major-version candidates.

### 4. Research Breaking/API Changes

For each major-version candidate, gather the breaking changes from official sources:

- **Rust crates:** crates.io page → repository → `CHANGELOG.md` or release notes
- **JS packages:** npm package page → repository → releases / migration guide

Pay special attention to:

- Peer-dependency constraints on the frontend
- Transitive dependency alignment on the backend (e.g. `kube` and `k8s-openapi` must move together)
- Pre-1.0 Rust crates where minor bumps are breaking
- Release-candidate versions — generally wait for stable unless explicitly requested

### 5. Present a Summary and Wait for Approval

Summarize the candidates in a table and ask the user which upgrades to apply. Each row should include:

- Package name
- Current version
- Latest version
- Risk level (Low / Medium / High)
- Key breaking changes or blockers

Mark packages as:

- **Approved:** proceed with upgrade
- **Deferred:** wait for stable release or further investigation
- **Rejected:** blocked by peer dependency or known usage issues

### 6. Apply Approved Upgrades

#### Frontend

Edit `web/package.json` to bump the approved package version, then run:

```bash
pnpm -C web install
pnpm -C web check
```

#### Rust

Edit `Cargo.toml` workspace dependency versions, then run:

```bash
cargo update -p <crate>
cargo check --workspace
cargo clippy --workspace --all-targets --all-features
```

For coupled upgrades (e.g. `kube` + `k8s-openapi`), update both in `Cargo.toml` first, then run:

```bash
cargo update -p kube -p k8s-openapi
cargo check --workspace
```

### 7. Check `Cargo.lock` for Duplicate Transitive Versions

After upgrading Rust crates, inspect `Cargo.lock` to see whether the upgrade introduced multiple versions of the same crate.

```bash
grep -A2 'name = "<crate>"' Cargo.lock
```

If multiple semver-incompatible versions appear, determine whether this is acceptable:

- **Acceptable:** upstream crates still depend on the old version; Cargo resolves this by keeping both. Functionality is fine, but compile time/binary size increases slightly.
- **Not acceptable:** wait until upstream crates update, or do not perform the upgrade.

### 8. Promote Changes to the Real Repository

Once the temporary copy builds and passes checks cleanly, apply the same changes to the real repository. Run the same verification commands in the real project before committing.

## Common Pitfalls

1. **Pre-1.0 Rust crates:** Remember that `0.6 → 0.7` is a breaking change, just like `1 → 2`.
2. **Peer dependencies:** A frontend package may be blocked from upgrading because a peer dependency does not support the new major version yet.
3. **Coupled Rust upgrades:** Some crates must be upgraded together (e.g. `kube` and `k8s-openapi`).
4. **Release candidates:** Do not upgrade production code to RC versions unless explicitly requested.
5. **Duplicate transitive versions:** A clean `cargo check` does not guarantee only one version of a crate exists in `Cargo.lock`. Always inspect whether the duplicate matters.

## Helper Files

- `.agents/skills/dependency-update/check_rust_outdated.py` — scans `Cargo.toml` workspace deps and reports breaking-version candidates.

## Verification Checklist

- [ ] `pnpm -C web up` ran without errors
- [ ] `cargo update` ran without errors
- [ ] `pnpm -C web check` passes
- [ ] `cargo check --workspace` passes
- [ ] `cargo clippy --workspace --all-targets --all-features` passes (or at least has no new warnings)
- [ ] Major-version candidates documented with breaking changes
- [ ] User approved each applied major-version upgrade
- [ ] `Cargo.lock` inspected for duplicate transitive versions
