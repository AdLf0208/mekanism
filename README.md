# Mekanism — Horizon port

A port of [Mekanism](https://github.com/mekanism/Mekanism) by Aidan C. Brady and
the Mekanism contributors for Horizon. These first slices bring four pieces of
the ore-processing chain:

- **Energized Smelter** — 50 FE/t, 200 ticks per smelt, a 20,000 FE buffer.
- **Enrichment Chamber** — the jar's recipes: one ore becomes two dusts, raw ore
  3 becomes 4, a raw block 1 becomes 12.
- **Basic Energy Cube** — a 4,000,000 FE buffer with the mod's side
  configuration.
- **Basic Universal Cable** — moves 8,000 FE/t between them.

Place them, feed items and FE through their sides, and right-click to open their
menus. The rest of Mekanism — the other machines, the multiblocks, the tools and
the deep-tier content — is a planned gap list. MIT — original copyright and
licence notices are kept in the port repository; see LICENSE and
LICENSE-NOTICE.md there.

**Original:** [Mekanism](https://github.com/mekanism/Mekanism) 10.7.19.85 by
Aidancbrady, Thommy101, Thiakil, pupnewfster, dizzyd, **MIT**. The original
copyright and licence notices are kept in [LICENSE](LICENSE), and
[LICENSE-NOTICE.md](LICENSE-NOTICE.md) records where each was read from.

**Horizon:** a port for [Horizon](https://github.com/horizon-rust/horizon), Minecraft:
Java Edition 26.3 rewritten in Rust. The exact Horizon and horizon-mixins
revisions this port builds against are pinned in `compatibility.json`.

## Install

1. Take `mekanism.hnmod` from this repository (or build it below).
2. Put it in the game folder's `horizon-native-mods` folder — the game folder is the one
   holding `options.txt`.
3. Native mods are always on once installed; the first install shows a
   trust notice, and the launcher builds the mod for your game.

To try it without installing, start the game with
`--dev-mod mekanism.hnmod`.

## Build

The port is a native mod (`.hnmod`): carried content plus a Rust runtime and
source mixins.

```sh
cargo test --locked --offline                        # the pure runtime rules
python3 tools/horizon.py stage   --horizon <horizon-checkout>
python3 tools/horizon.py check   --horizon <horizon-checkout>
python3 tools/horizon.py test    --horizon <horizon-checkout>
python3 tools/horizon.py build   --horizon <horizon-checkout>   # rebuild the .hnmod
```

`tools/e2e.sh` is the in-game harness. It expects a campaign checkout
(`$CAMPAIGN_ROOT`), the shared pins under `ports/_shared/pins/`, and the
prepared 26.3 resources (`$HORIZON_RES`, default `.local/horizon-client/26.3`).

## Layout

`content/` is the shipped content; `mixins/mekanism/` holds the declarations and the runtime; `tools/` builds and verifies the port.

This is an unofficial port: Horizon is not affiliated with the original author.
The licence above is the original's and travels with every copy; see
[LICENSE](LICENSE) and [LICENSE-NOTICE.md](LICENSE-NOTICE.md).
