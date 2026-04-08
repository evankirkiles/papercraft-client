# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Paperarium is a 3D papercraft editor with a Rust/WASM core and React frontend. The Rust engine handles state, rendering (via wgpu), and editing logic; it compiles to WASM and is consumed by a React-based editor UI.

## Architecture

The repo is a hybrid monorepo managed by **pnpm workspaces + Turborepo** (JS/TS) and **Cargo workspaces** (Rust).

### Rust crates (`crates/`)

- **pp_core** — Core state model: meshes, materials, selections, settings. The source of truth for document state.
- **pp_draw** — GPU rendering via wgpu. Contains render engines, shaders, selection picking, and draw caching.
- **pp_editor** — Client-side editor state: viewports, splits, tool configs, scene tree. Not synced to server.
- **pp_client** — The WASM entry point. Owns the `App` struct (controller) which ties together pp_core (model) and pp_draw (view). Handles events, keyboard input, command history, and tool dispatch. Also has a `ts/` directory with TypeScript wrappers around the WASM bindings.
- **pp_protocol** — Wire protocol for client-server communication.
- **pp_save** — Save/load logic for document files (`.pp` format).
- **pp_server** — Axum-based server for multiplayer collaboration. Optional S3 support via `s3` feature flag.

### JS/TS packages (`packages/`)

- **@paperarium/editor** (`packages/editor`) — React UI: viewport component, editor controls, contexts (Engine, Editor, Theme). Built with Vite. Uses Tailwind CSS v4, shadcn/ui components, react-resizable-panels, TanStack Query.
- **@paperarium/client** (`crates/pp_client`) — The WASM module packaged for JS. Built via `wasm-pack` + Rollup. This is both a Cargo crate and an npm package.
- **@repo/eslint-config** (`packages/eslint-config`) — Shared ESLint flat config.

### Data flow

`pp_client::App` is the central controller. React (via `EngineContext`) instantiates and holds the WASM `App`, forwarding DOM events. The App processes events through tools, applies commands to `pp_core::State`, and triggers re-renders via `pp_draw::Renderer`. Commands are tracked in a `MultiplayerCommandStack` for undo/redo and server sync.

## Common Commands

```bash
# Install JS dependencies
pnpm install

# Development (starts Vite dev server + server, builds WASM first)
pnpm dev

# Build everything (WASM + JS)
pnpm build

# Build WASM only (from crates/pp_client)
cd crates/pp_client && pnpm compile-wasm        # release
cd crates/pp_client && pnpm compile-wasm --dev   # debug

# Lint / typecheck
pnpm lint                    # ESLint across all packages
pnpm typecheck               # TypeScript across all packages
pnpm typecheck:rs            # cargo check (excludes pp_server)
cargo check --workspace      # all Rust crates including pp_server

# Format
pnpm format                  # Prettier for TS/CSS/MD
cargo fmt                    # rustfmt for Rust

# Dead code analysis
pnpm knip
pnpm knip:production

# Storybook (editor package)
cd packages/editor && pnpm storybook

# Run server example locally
cargo run --example local -p pp_server
```

## Rust Conventions

- Rust edition 2021, MSRV 1.90
- rustfmt config: 100 char line width, crate-level import granularity, wrapped/normalized comments
- The WASM target is `wasm32-unknown-unknown`. Use `cfg-if` / `cfg(target_arch = "wasm32")` for platform-specific code.
- Types exposed to JS use `#[wasm_bindgen]` and `tsify` for TypeScript type generation.
- State uses `slotmap` for entity storage and `BTreeMap` (not `HashMap`) for deterministic ordering.

## JS/TS Conventions

- Node >= 24, pnpm 8.9
- TypeScript ~6.0
- Tailwind CSS v4 (not v3 — uses `@tailwindcss/vite` plugin, CSS-based config)
- Pre-commit hooks via husky + lint-staged (Prettier + ESLint)
