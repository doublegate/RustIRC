<!-- Managed by Master-Claude. Universal rules come from the imported/inlined core.
     Edit only inside the MC-PROJECT block; mc-sync overwrites everything else. -->
# AGENTS.md — RustIRC

@/home/parobek/.claude/master-core/AGENTS.base.md
@/home/parobek/.claude/master-core/lang/rust.md
@/home/parobek/.claude/master-core/modules/10-commits-and-versioning.md
@/home/parobek/.claude/master-core/modules/20-testing-and-accuracy.md
@/home/parobek/.claude/master-core/modules/30-quality-gates.md
@/home/parobek/.claude/master-core/modules/40-docs-and-adrs.md
@/home/parobek/.claude/master-core/modules/50-architecture-patterns.md
@/home/parobek/.claude/master-core/modules/60-security.md
@/home/parobek/.claude/master-core/modules/70-release-ceremony.md
@/home/parobek/.claude/master-core/modules/80-phase-sprint-workflow.md
@/home/parobek/.claude/master-core/modules/90-multi-language-integration.md
@/home/parobek/.claude/master-core/modules/95-named-pattern-library.md

<<< MC-PROJECT-START >>>
## Project: RustIRC

> Project-specific guidance. Universal rules (read-before-write, conventional commits,
> quality gates, Rust build/test/lint commands, security baseline) come from the shared
> core imported above and are not repeated here.

## Overview

Modern Rust IRC client combining mIRC (scripting/customization), HexChat (friendly GUI +
plugins), and WeeChat (efficiency, buffer management). Targets full IRC-standard compliance —
RFC 1459/2812, IRCv3 extensions, DCC, SASL (PLAIN/EXTERNAL/SCRAM-SHA-256) — on Linux, macOS,
and Windows 10+. Three interface modes share one core: GUI (default), TUI (`--tui`), CLI (`--cli`).

## Current status

- **v0.5.0** on the `dioxus` branch: GUI migrated from iced 0.14.0 to **Dioxus 0.7.3 + Axum**.
- 254 tests passing across the workspace; zero clippy warnings.
- Phases 1-6 complete: research, core engine, UI, scripting/plugins/config, advanced features
  (DCC, IRCv3, proxy, notifications), testing/integration.
- `main` holds the stable v0.4.2 (iced) release; `dioxus` is active development.

## Architecture

Event-driven, 6-crate workspace: `rustirc-core`, `rustirc-protocol`, `rustirc-gui`,
`rustirc-tui`, `rustirc-scripting`, `rustirc-plugins`.

- **Connection manager** — async (Tokio) tasks, separate state per server; multi-server via
  `HashMap<String, ServerData>`, checked for connection state before IRC ops.
- **Protocol parser** — RFC 1459/2812 + IRCv3 (CAP LS/REQ, message tags, server-time, batch,
  away/account tracking).
- **DCC handler** — file transfers and direct chats, with IP-masking and security warnings.
- **SASL** — PLAIN, EXTERNAL, SCRAM-SHA-256.
- **Scripting/plugins** — mlua (Lua) scripts + process/thread-isolated Rust plugins; event API
  (`on_message`, `on_join`, ...).
- **TLS** — rustls; all network traffic over TLS by default.

## Commands (project-specific)

```bash
cargo run                        # GUI mode (Dioxus desktop)
cargo run -- --tui               # TUI mode (ratatui)
cargo run -- --cli               # CLI prototype mode
cargo run -- --config path.toml  # custom config
cargo build --target x86_64-pc-windows-gnu   # cross builds
cargo build --target x86_64-apple-darwin
```

Build/test/fmt/clippy gates and `cargo doc` come from the shared Rust overlay.

## Design constraints

- TLS by default; system-keychain credential storage (planned); sandboxed scripting environment.
- Validate against malformed IRC messages at the parse boundary.
- Performance target: 100+ simultaneous channels without UI lag; background logging and message
  processing; responsive UI under heavy message load.
- Extensibility: event-driven script/plugin API, isolated plugins, config-file themes, built-in
  script/plugin manager for discovery and installation.

## Dioxus 0.7.3 GUI patterns (current — supersedes all prior iced/Material Design 3 patterns)

- **State**: `Signal<AppState>` for reactive state with automatic re-render. The EventBus bridges
  core events to signals via `use_coroutine()`. `IrcActions` is a Copy-type dispatcher
  (connect/send/join/leave) backed by an `OnceLock<Arc<IrcClient>>` global; use `spawn_forever()`
  for network ops that must survive component unmount.
- **Components**: 18 RSX components in `crates/rustirc-gui/src/components/`.
- **Theming**: 22 themes as CSS custom properties via `[data-theme="..."]` selectors; Tailwind
  utility classes in RSX `class` attributes; IRC color codes as `.irc-color-N` classes.
- **Assets**: load CSS with `include_str!()` + `document::Style` (not the `asset!()` macro);
  `Dioxus.toml` configures the dx CLI (hot-patching, asset dir).
- **RSX gotchas**: Signal is Copy — `let mut s = signal; s.write()` to mutate. No `<` before
  `{var}` in format strings, and no custom attributes in RSX — pre-compute strings with `format!()`
  before the `rsx!` block (e.g. `env!("CARGO_PKG_VERSION")` needs a `{format!(...)}` code block).
  `impl FromStr` for enums (not an inherent `from_str`) to satisfy clippy.

## Implementation rules & gotchas

- **Zero placeholders**: implement functionality fully; never leave "in a real implementation"
  stubs. Fix compile errors by implementing, not by removing/disabling features. Platform code
  uses `#[cfg]` with complete Windows (PowerShell), macOS (osascript), Linux (notify-send) paths.
- **IRC field names**: verify against the protocol crate before use (e.g. WHOIS uses `targets`,
  not `target`/`nickmasks`).
- **Message filtering**: handle both `"System"` and `"system"` sender casing.
- Tests cover protocol edge cases against mock IRC servers.

## CI/CD gotchas (GitHub Actions)

- **Order**: run clippy only after a successful build — clippy before/parallel-with build fails
  with "can't find crate for <dep>".
- **Function persistence**: steps run in separate shells; define helpers in
  `$RUNNER_TEMP/ci_helpers.sh`, point `BASH_ENV` at it, and `export -f` them.
- **Cross-platform timeout**: macOS lacks `timeout` (exit 127); fall back to
  `perl -e "alarm $d; exec @ARGV"`.
- **sccache resilience**: probe with `sccache --start-server`; on HTTP 400 (GHA cache outage)
  `unset RUSTC_WRAPPER` and fall back to local disk (`SCCACHE_GHA_ENABLED=false`).
- **rust-cache@v2**: no `restore-keys` input (only `key`/`shared-key`/`save-if`).
- Validate workflow YAML before pushing: `python3 -c "import yaml; yaml.safe_load(open(f))"`.

## Docs & todos

Project docs in `docs/` (overview, architecture-guide, technology-stack, project-status,
`phases/`, `specs/`); phase todo lists in `to-dos/`; reference plans in `ref_docs/`. Volatile
session state belongs in `CLAUDE.local.md`, not this block.

<<< MC-PROJECT-END >>>
