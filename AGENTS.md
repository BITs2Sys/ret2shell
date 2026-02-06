# Ret2Shell AGENTS Instructions

## Overview

Follow these instructions when contributing to this repository. Keep changes minimal and consistent with existing conventions.

## Repository Structure

- **Backend**: Rust services and libraries under `crates/`.
- **Frontend**: SolidJS application under `web/`.

## Frontend (SolidJS + TypeScript)

- Use SolidJS idioms with functional components.
- Prefer existing helpers in `web/src/lib` and keep imports organized.
- Formatting and linting are handled by Biome:
  - `pnpm -C web format`
  - `pnpm -C web lint`
- Follow the existing file structure: routes in `web/src/routes`, reusable widgets in `web/src/lib/widgets`, and API modules in `web/src/lib/api`.
- Prefer `async/await` grammar than old Promise (then, catch, finally, etc.) one.
- Keep translation file sync, the translations is presented in `web/src/lib/i18n`, take a look at `index.ts`.

## Backend (Rust)

- Follow the formatting conventions in `rustfmt.toml` (2-space indentation, same-line braces).
- Favor existing helpers in `crates/*/src` and keep modules aligned with current structure.
- Never add new dependencies in you work, if you actually need it to implement some functions, abort the session and told user to add dependency manually.
- Run formatting with `cargo +nightly fmt` when touching Rust files.
- You'd better use `cargo clippy` when you finished your work, and fix all the warnings/errors shown in clippy.

## Git & Pull Request Conventions

- **Commit format**: Use gitmoji (emoji shortcodes, see https://gitmoji.dev) for every commit, followed by a concise message.
  - Format: `<gitmoji> <summary>`
  - Example: `:sparkles: add new challenge export`
- **Common gitmoji**:
  - `:sparkles:` New feature
  - `:bug:` Bug fix
  - `:memo:` Documentation
  - `:recycle:` Refactor
  - `:art:` Formatting/style changes
  - `:zap:` Performance improvement
  - `:white_check_mark:` Tests
  - `:construction:` Work in progress
  - `:fire:` Remove code or files
  - `:package:` Dependency updates
- **Scope**: One logical change per commit; keep commits small and focused.
- **PR Title**: Prefix with the same gitmoji shortcode as the primary change.
  - Example: `:memo: update AGENTS instructions`
- **PR Description**: Include a brief summary plus any testing performed.
- **Branch naming**: Use short, kebab-case names (e.g., `docs/git-guidelines`).

## Testing

- Only run tests relevant to your changes.
- Frontend: `pnpm -C web lint` (and `format` if needed).
- Backend: `cargo +nightly fmt` and targeted test commands if applicable.
