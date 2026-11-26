# AGENTS.md — Game Off Waves

A Vampire Survivors–like game played at sea. The player steers a boat, auto-firing
weapons mounted on its sides and bow fight off waves of enemies, and the sea pushes
the boat around: waves now, storms later.

Built with **Bevy 0.20** and **avian2d 0.8** (2D physics) on a pinned nightly
toolchain. The project began as the
[bevy_new_2d](https://github.com/TheBevyFlock/bevy_new_2d) template; the screen,
menu, theme, audio and asset-tracking scaffolding comes from that template.

## Commands

Run `mise install` once (mise ≥ 2026.11.14). It installs `just`, the `bevy` CLI and
`bevy_lint` at the versions pinned in `mise.toml`.

All checks live in the `Justfile` and match CI (`.github/workflows/ci.yaml`). The
check recipes set `RUSTFLAGS=-Dwarnings`, so a warning fails them.

| | |
|---|---|
| `just run` | run natively through the Bevy CLI (`bevy run`) |
| `just run-web` | build for wasm and serve locally |
| `cargo run` | run natively without the Bevy CLI (default feature `dev_native`) |
| `just test` | `cargo test --locked --workspace` |
| `just clippy` | clippy, all targets, all features |
| `just fmt` | `cargo fmt --check` |
| `just docs` | rustdoc, private items included |
| `just check-web` | `cargo check` for `wasm32-unknown-unknown` |
| `just bevy-lints` | Bevy-specific lints |
| `just all` | everything, in CI order |

`--locked` is used everywhere: after changing `Cargo.toml`, run `cargo update` so the
lockfile follows.

The toolchain is pinned in `rust-toolchain.toml`, and the same pin is written into
`ci.yaml` (`env.toolchain`). Change both together. It has to be nightly because the
Justfile uses `-Zshare-generics` and `-Zthreads`, and Bevy 0.20 needs rustc ≥ 1.97.1.

The project toolchain is deliberately not a mise tool. When mise manages Rust it
exports `RUSTUP_TOOLCHAIN`, which overrides the toolchain `bevy_lint` must be built with.

`bevy_lint` supports one Bevy version at a time and is built and run with its own
nightly. Both are pinned once, under `[vars]` in `mise.toml`, and CI installs from
there. If `just bevy-lints` fails because that nightly is missing, run
`mise run bevy-lint-toolchain`.

Features: `dev` turns on dynamic linking, the Bevy dev tools, the UI debug overlay and
`track_location`, and `dev_native` adds asset hot reloading on top. Release and web
builds turn default features off (see `[package.metadata.bevy_cli]`).

## Layout

```
src/main.rs          AppPlugin: DefaultPlugins, physics, global sets, Pause state, camera
src/asset_tracking   load a Resource as an Asset and insert it once its dependencies load
src/audio.rs         Music / SoundEffect categories, global volume
src/dev_tools.rs     dev-only: state-transition logging, backquote toggles the UI debug overlay
src/screens/         top-level `Screen` state: Splash → Title → Loading → Gameplay → YouDied
src/menus/           `Menu` state on top of screens: Main, Settings, Credits, Pause, Shop
src/theme/           UI widgets (buttons, labels, roots), palette, hover/press colours
src/demo/            the game itself (the name comes from the template)
  mod.rs             collision layers, Health, Daytime, forward/angle helpers
  player.rs          boat entity, input, follow camera, health HUD, PlayerStats
  movement.rs        MovementController → avian forces; wave forces on every body
  enemy.rs           spawners, enemy boats, chasing, contact damage, kills → gold
  weapons.rs         WeaponSlots (3 left, 3 right, 1 front): cannon, archer, fire mage
  upgrade.rs         shop UI: stat upgrades and weapon-slot purchases
  haven.rs           havens (ports): material only, gameplay not implemented yet
  level.rs           spawns the level root with the player
  terrain/           height-map chunk, land colliders, spawn points, wave directions
assets/shaders/      one WESL fragment shader per material
```

### Gameplay loop

`Daytime` runs a 120 s timer from entering `Screen::Gameplay`. Night is meant to end
in `Menu::Shop`, where kill gold and stat points are spent. Leaving the shop starts
the next day, and enemy health and damage grow with `Daytime::day_counter`.

### Terrain and waves

One 4096 px chunk made of a 16×16 grid (`SQUARE`) of noise heights. A cell above
`WATER_LEVEL` is land: it gets a static collider. Every water cell gets an enemy
`Spawner`. `Waves::init` gives each water cell a direction pointing away from nearby
land. `Waves::wave_height` turns that direction into a travelling sine wave, and
`movement::apply_waves` pushes every physics body with it. The heights and the wave
directions are also uploaded as textures (`R32Float`, `Rg32Float`) so that
`terrain.wesl` draws the same waves.

`terrain/height.rs::world_2_chunk` maps world positions to grid cells. Its test pins
the coordinate convention: world (0, 0) is the centre of the chunk.

## Conventions

- **One plugin per feature.** Each module exposes a `plugin` fn or an `XPlugin`
  struct, and `demo::plugin` / `AppPlugin` add them.
- **System ordering is centralised.** `AppSystems` (`TickTimers → RecordInput →
  Update`) is chained once in `main.rs`. Plugins only `.in_set(...)`. Systems that
  must stop while paused go in `PausableSystems`.
- **Gate gameplay systems** with `.run_if(in_state(Screen::Gameplay))`.
- **Lifetime by state.** Spawned gameplay entities carry `DespawnOnExit(Screen::Gameplay)`.
  Anything that must not survive into the shop also carries `DespawnOnEnter(Menu::Shop)`.
- **Errors warn, they do not panic.** `app.set_error_handler(warn)` is global, so a
  system returning `Err` logs a warning every frame. When it is normal for a system's
  target to be missing (no player yet, a HUD despawned while the shop is open), take
  it as `Single<..>`, which skips the system silently, or gate the system to the state
  where the target exists. Keep `query.single()?` for cases where a missing target
  is a real bug.
- **Collision layers** come from `GameCollisionLayer` (Terrain, Player, Enemy). Gravity
  is zero: this is a top-down game.
- **Forward is +Y.** `forward_vec` / `angle_between` in `demo/mod.rs` define it, and
  tests pin it. Use these helpers rather than recomputing angles.
- **Custom materials** are `Material2d` + `AsBindGroup`, with a `time: Vec4` uniform at
  binding 0 that an `update_time` system refreshes every frame. The WESL shaders bind
  at `@group(2)`, take the default 2D vertex shader, and read `uv` from `@location(2)`.
  `WeaponMaterial` uses `time.y` to choose between cannonball, flame and arrow.
- **UI buttons** come from `theme::widget::button`/`button_small`. They take an observer
  (usually `On<PointerClick>`) and a marker component that is copied onto the button
  and its text, so one handler can tell buttons apart. Hover and press colours come
  from `InteractionPalette`, driven by `Hovered` / `Pressed`.
- **`children![]` uses square brackets**: `clippy.toml` enforces it.

## Bevy 0.20 notes

Common traps when writing code against this version:

- Picking events are flat types: `On<PointerClick>`, `On<PointerOver>`, not
  `On<Pointer<Click>>`.
- Lifecycle observers take the component as a type parameter: `On<Insert<T>>`, not
  `On<Insert, T>`.
- Run-condition combinators are `.and_then(...)` / `.or_else(...)`.
- `Interaction` and `bevy_ui`'s `Button` are deprecated. Use `bevy::ui_widgets::Button`
  with `bevy::picking::hover::Hovered` and `bevy::ui::Pressed`.
- `BorderRadius` is a `Node` field, not a component.
- `Assets::get_mut` returns `AssetMut`: bind it `mut`.
- Loading with settings: `asset_server.load_builder().with_settings(..).load(path)`.
- WESL is always enabled, and there is no `shader_format_wesl` feature.
- The UI debug overlay toggles through the `GlobalUiDebugOptions` resource.
