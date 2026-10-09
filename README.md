# Licht Launcher

An unofficial Minecraft: Java Edition launcher targeting Windows and Linux,
built with Tauri v2, React, TypeScript, and Rust.

**Unofficial project, not affiliated with Mojang Studios or Microsoft.**

## Project status

Early development. The foundation phase is complete: an empty desktop window, a
standalone Rust core, structured logging, typed core errors, the design system,
TanStack Router with file routes under `src/routes`, frontend tests, and
Windows/Ubuntu CI. The style guide is at `/#/styleguide` when
`VITE_SHOW_STYLEGUIDE=true` (see `.env.example`). Game installation, game
launching, and account authentication are planned in the [roadmap](docs/ROADMAP.md).

## Prerequisites

- Node.js 24, version 24.15.0 or newer within that major version.
- pnpm 10.33.0, as specified in `package.json`.
- Rust installed through rustup, with `cargo`, `rustc`, and `rustup` on PATH.
  The repository selects Rust 1.99.0, rustfmt, and Clippy through
  `rust-toolchain.toml`.
- [Tauri v2 system prerequisites](https://v2.tauri.app/start/prerequisites/):
  C++ build tools, Windows SDK, and WebView2 on Windows; the required development
  libraries, including WebKitGTK 4.1, on Linux.

## Development

From the repository root:

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

The frontend development server is started automatically by the Tauri command.

To build the release executable without an installer:

```sh
pnpm tauri build --ci --no-bundle -- --locked
```

## Quality checks

```sh
pnpm lint
pnpm typecheck
pnpm test
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The same checks and the Tauri release build run in GitHub Actions on Windows and
Ubuntu. `pnpm lint` checks formatting and imports as well as lint rules.

## Documentation

- [Architecture](docs/ARCHITECTURE.md): layer responsibilities and project boundaries.
- [Roadmap](docs/ROADMAP.md): implementation stages and decisions requiring approval.
- [Compatibility](docs/COMPATIBILITY.md): version differences the launcher has to cover.
- [Project rules](AGENTS.md): development conventions.

## License

Licensed under the [MIT License](LICENSE).

Copyright (c) 2026 Mateus Neiva.
