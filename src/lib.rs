//! Mekanism for Horizon — slices 1–3, one flat SDK package (mod API 0.2).
//!
//! The carried blocks, items, models, textures, recipes, loot, tag and lang are
//! the `assets/` and `data/` at the package root; the behaviour is the one
//! `impl Hooks` in this file and its glue in [`support`]. The rules themselves
//! live in [`runtime`], byte-identical to the retired `.hnmod` bundle's
//! `runtime.rs`.
//!
//! What the retired bundle expressed as eight injected helpers is here three
//! shapes: each block's block entity is a **Part** (attached to the type
//! declared for the block), its container and its energy buffer are
//! **declarations** keyed by the block, and the harness commands are the typed
//! `register_commands`/`run_mod_command` pair. Its embedded-content helper has
//! no successor: one `.hmod` carries the content and the code, which is what
//! that helper existed to work around.
use mod_api::{
    self, Command, CommandArgumentValue, CommandSource, Flow, Hooks, LevelRef,
    RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent, ServerHost, Text,
};

mod support;

pub mod runtime;

/// Whether this command source may read the machine at `pos`: an operator
/// (permission ≥ 2), or a player in the same level within
/// [`runtime::INSPECT_RANGE`] of the block.
fn may_read(host: &dyn ServerHost, source: &CommandSource, level: LevelRef, pos: [i32; 3]) -> bool {
    let op = source.permission().level() >= runtime::LEVEL_ADMINS;
    match source {
        CommandSource::Player { player, .. } => {
            let same_level = player.level(host) == Some(level);
            let distance = player
                .entity()
                .position(host)
                .map(|at| {
                    let block = [
                        f64::from(pos[0]) + 0.5,
                        f64::from(pos[1]) + 0.5,
                        f64::from(pos[2]) + 0.5,
                    ];
                    (0..3)
                        .map(|axis| (at[axis] - block[axis]).powi(2))
                        .sum::<f64>()
                        .sqrt()
                })
                .unwrap_or(f64::MAX);
            runtime::may_inspect(op, same_level, distance)
        }
        _ => op,
    }
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the five blocks: the three machines' containers and buffers,
    /// the cube's and the cable's buffers, and a block entity type each.
    ///
    /// The retired bundle wrote each of these into a hook's shared output
    /// (`block_entity`, `block_entity_ticks`, `block_entity_container`,
    /// `block_entity_energy`) and had to check the block id first. A
    /// declaration is keyed by the block, so a foreign block is another key and
    /// cannot be touched.
    fn register(&self, registrar: &mut dyn Registrar, context: &RegistrationContext) {
        support::declare(registrar, context);
    }

    /// `/mekanism info|energy|progress`: the harness controls the in-game check
    /// uses, since the slice draws no GUI of its own.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event
            .commands
            .push(Command::new("mekanism info <pos:block_pos>"));
        event.commands.push(Command::op(
            "mekanism energy <amount:integer> <pos:block_pos>",
            2,
        ));
        event
            .commands
            .push(Command::new("mekanism progress <pos:block_pos>"));
    }

    /// The command bodies. The arguments are parsed and resolved by the
    /// server's dispatcher, so a malformed command never reaches here; the
    /// answers are the 0.1.0 text, byte for byte, so its e2e assertions still
    /// match.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let literals: Vec<&str> = event.command.literals.iter().map(String::as_str).collect();
        let Some(CommandArgumentValue::BlockPos(pos)) = event.command.argument("pos") else {
            return Flow::Continue;
        };
        let line = match literals.as_slice() {
            ["mekanism", "info"] => {
                let Some(level) = host.level(host.dimension()) else {
                    return Flow::Continue;
                };
                if !may_read(&*host, event.source, level, *pos) {
                    let line = format!(
                        "You must be an operator or within {:.0} blocks of the machine.",
                        runtime::INSPECT_RANGE
                    );
                    let _ = host.commands().feedback(Text::literal(line));
                    return Flow::Cancel;
                }
                support::info_line(host, *pos)
                    .unwrap_or_else(|| "No block entity there.".to_owned())
            }
            ["mekanism", "energy"] => {
                let Some(CommandArgumentValue::Integer(amount)) = event.command.argument("amount")
                else {
                    return Flow::Continue;
                };
                match support::set_energy(host, *pos, *amount) {
                    Some(after) => format!("Energy set to {after} FE."),
                    None => "No block entity there.".to_owned(),
                }
            }
            ["mekanism", "progress"] => support::progress_line(host, *pos)
                .unwrap_or_else(|| "No block entity there.".to_owned()),
            _ => return Flow::Continue,
        };
        let _ = host.commands().feedback(Text::literal(line));
        Flow::Cancel
    }
}
