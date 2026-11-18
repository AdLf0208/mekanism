# Mekanism — a Horizon port

**Mekanism** is a Minecraft mod. A mod for Minecraft. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [mekanism/Mekanism](https://github.com/mekanism/Mekanism) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Mekanism ([source](https://github.com/mekanism/Mekanism)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `mekanism.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Reviewer:** [P1] the four machines cannot be crafted: recipes need unregistered items/tags, and the cube used the unimplemented `mekanism:mek_data` type
- **Reviewer:** [P2] the cube's menu and side configuration are advertised but not implemented
- **Reviewer:** licence notice
- **Reviewer:** registry artifact delivery
- **The cable's moves are measured, not planned.** 0.1.0 asked `native_energy` for each neighbour's
- **The machines' slots are sized from the container** rather than resized defensively on every tick:
- **`/mekanism` filters by block id.** The old read was this mod's entities only; the new one asks the

## Not yet ported

- **The rest of Mekanism.** Ore processing beyond smelting and enriching
- **The enrichment table is the jar's recipes over the items this port
- **No pipes or hoppers in the check**: the chamber's two dust are read out of
- **The machines' GUIs** are the vanilla furnace menu (3 slots, flame and
- **The cube and the cable models** are plain replacements for upstream's
- **Recipes are not craftable**: the carried recipes need
- **One cable call moves at most 4,000 FE per side for the cube** (the tier
- **No energy item**: the machines' energy slot is unused and there is no

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `mekanism.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Mekanism was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/mekanism/Mekanism

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run mekanism` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/mekanism/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
