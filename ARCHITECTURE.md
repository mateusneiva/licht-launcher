# Architecture

Licht Launcher is an unofficial Minecraft: Java Edition launcher for Windows and
Linux. It uses Tauri v2, a React + TypeScript + Vite frontend, and a Rust Cargo
workspace. [AGENTS.md](AGENTS.md) defines the project rules;
[ROADMAP.md](ROADMAP.md) tracks implementation.

## Layers

| Location | Responsibility | Boundary |
| --- | --- | --- |
| `crates/licht-core` | Minecraft manifests, rules, downloads, Java runtimes, game execution, and authentication | A standalone Rust library with no Tauri dependency |
| `src-tauri` | Application startup, logging configuration, thin commands, and frontend events | Delegates game behavior to the core |
| `src` | Screens, UI state, and calls to the Tauri adapter | Contains no Minecraft domain logic |

The Cargo workspace contains `licht-core` and the `licht-launcher` application
package. The application depends on the core through a local path dependency;
its executable is `licht`.

The current foundation opens an empty React frontend. The core exposes its base
error type, and the application initializes logging. Domain operations and their
Tauri commands are not implemented yet.

## Communication

The intended flow is:

```text
Frontend -> typed Tauri commands -> licht-core
Frontend <- Tauri events <- adapter handling core progress and status
```

The frontend will use TanStack Query for command loading, caching, and errors.
The adapter will emit progress events with approximately 100 ms throttling.
Shared Rust/TypeScript types must be generated rather than maintained by hand;
the choice between `ts-rs` and `specta`, and the command/event contracts, remains
pending.

## Frontend layout

- `src/components/ui`: shared design-system components using theme tokens.
- `src/features`: screens and UI behavior grouped by feature.
- `src/lib`: UI utilities and typed invocation wrappers.

These folders are currently placeholders. Tailwind, shadcn/ui, Lucide, TanStack
Query, Zustand, and TanStack Virtual belong to the planned frontend stack and
have not been installed yet. Long lists will be virtualized, and screens will
reuse the design system. The frontend runs in WebView2 on Windows and WebKitGTK
on Linux.

## Errors and logging

The core exports `CoreError`, derived with `thiserror`, and a `Result<T>` alias.
Its initial `Io` variant preserves the original I/O error as its source.
Application startup uses `anyhow` to add context at the boundary; errors are
propagated with `?` rather than unwrapped or silently discarded.

Logging uses `tracing`. The executable configures a `tracing-subscriber` before
starting Tauri, writes structured text to stderr, and uses INFO when `RUST_LOG`
is absent or empty. Invalid filters return an error. Logs must not contain
tokens, secrets, or personal data.

## Testing

The core is testable independently with `cargo test -p licht-core`. Domain tests
will use Mojang JSON fixtures and must not depend on network access. Frontend
tests use Vitest, Testing Library, and jsdom.

CI runs Biome checks, TypeScript typechecking, frontend tests, Rust formatting,
Clippy, Rust tests, and a Tauri release build on Windows and Ubuntu. Automated
builds and tests are separate from manual graphical validation.
