# Licence notice — Mekanism

This port carries the original mod's assets and data, and rewrites its behaviour
in Rust (or, for a data-tier port, declares its blocks from the same data). The
original licence and copyright notices are reproduced verbatim in
[LICENSE](LICENSE).

## Where the licence was read

- The pack jar `Mekanism-1.21.1-10.7.19.85.jar` (kept at
  `_shared/carry-work/mekanism/`) declares `license="MIT"` in
  `META-INF/neoforge.mods.toml` and carries no licence file.
- The MIT text and `Copyright (c) 2017-2025 Aidan C. Brady` come from the
  upstream repository's `LICENSE` (<https://github.com/mekanism/Mekanism>).

## The port

- Original: Mekanism 10.7.19.85 by Aidan C. Brady and contributors — MIT.
- Licence: **MIT**
- Port to Horizon by AdLf0208: the Rust runtime, the Horizon declarations and the
  build/verification tooling in this repository.
- No Mojang file is included. The carried files are the mod's own assets and
  data; `data/minecraft/tags/…` files only add this mod's ids to vanilla tags.
