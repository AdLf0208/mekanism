// Game-side glue for the Mekanism slices 1-2 (energized smelter, energy cube,
// universal cable, enrichment chamber).
//
// Injected into `crate::game` as `mekanism_state`; it needs the game, so it is
// not part of the standalone library (only `runtime.rs` is).
//
// Energy lives in each block entity's saved compound as the int `energy`
// (`native_block_entity_energy`, API 142) and the smelter's progress as
// `progress`, so both survive a save and the block's own tick reads them. The
// cable moves energy between its neighbours on its own tick with the energy
// helpers; the engine takes a ticking block entity out of the world, so the
// cable's own buffer is read and written through the entity it is handed.

use horizon_client::containers::{ModBlockEntity, ModContainer, ModEnergyBuffer};
use horizon_client::items::ItemStack;
use horizon_core::geometry::Direction;
use horizon_world::nbt::{Compound, JavaString, Tag};

use crate::game::mekanism::{
    CABLE_CAPACITY, CHAMBER_CAPACITY, CHAMBER_OPERATION, CHAMBER_TICKS, CUBE_CAPACITY, CUBE_RATE,
    Endpoint, IdleReason, Machine, Move, Operation, Outcome, Recipe, SMELTER_CAPACITY,
    SMELTER_OPERATION, SMELTER_TICKS, Stack, cable_move_limit, cable_plan, clamp_energy, enriching,
    process_tick, smelting,
};
use crate::game::native_server::{NativeCommand, NativeServerPlayer};

/// The four blocks of slices 1-2.
const SMELTER: &str = "mekanism:energized_smelter";
const CUBE: &str = "mekanism:basic_energy_cube";
const CABLE: &str = "mekanism:basic_universal_cable";
const CHAMBER: &str = "mekanism:enrichment_chamber";

/// A machine's menu slots: input, unused (the furnace menu's fuel slot) and
/// output, as upstream's input/output pair and a free middle slot.
const INPUT_SLOT: usize = 0;
const OUTPUT_SLOT: usize = 2;
/// How many item slots the container declares (the furnace menu's three).
const MENU_SLOTS: usize = 3;

/// The vanilla furnace menu's four data slots: flame (stored / capacity) then
/// arrow (progress / required ticks).
const MENU_DATA: usize = 4;

/// A machine's saved energy (`native_block_entity_energy`'s int).
fn be_energy(entity: &ModBlockEntity) -> i32 {
    match entity.nbt.get(&JavaString::from("energy")) {
        Some(Tag::Int(energy)) => *energy,
        _ => 0,
    }
}

fn set_be_energy(entity: &mut ModBlockEntity, energy: i32) {
    entity
        .nbt
        .insert(JavaString::from("energy"), Tag::Int(energy));
}

/// A machine's saved progress, in ticks.
fn be_progress(entity: &ModBlockEntity) -> i32 {
    match entity.nbt.get(&JavaString::from("progress")) {
        Some(Tag::Int(progress)) => *progress,
        _ => 0,
    }
}

fn set_be_progress(entity: &mut ModBlockEntity, progress: i32) {
    entity
        .nbt
        .insert(JavaString::from("progress"), Tag::Int(progress));
}

/// A slot as the pure core's stack, empty when the slot is empty.
fn slot_stack(slot: Option<&ItemStack>) -> Stack {
    match slot {
        Some(stack) if stack.count > 0 => Stack::new(&stack.item, stack.count),
        _ => Stack::default(),
    }
}

/// Write the pure core's stack back into `slot`, keeping the previous stack's
/// components when it is the same item.
fn set_slot(items: &mut [Option<ItemStack>], slot: usize, value: &Stack) {
    if value.is_empty() {
        items[slot] = None;
        return;
    }
    let same = items[slot]
        .as_ref()
        .is_some_and(|current| &*current.item == value.item);
    if same {
        if let Some(current) = items[slot].as_mut() {
            current.count = value.count;
        }
    } else {
        items[slot] = Some(ItemStack::new(&value.item, value.count));
    }
}

fn mark_modified(game: &mut crate::game::Game<'_>, pos: [i32; 3]) {
    game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
}

fn neighbour(pos: [i32; 3], face: Direction) -> [i32; 3] {
    let [dx, dy, dz] = face.normal();
    [pos[0] + dx, pos[1] + dy, pos[2] + dz]
}

/// Declare the four block entities (`native_block_entity`, API 15/129).
#[allow(private_interfaces)]
pub fn on_block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if matches!(block, SMELTER | CUBE | CABLE | CHAMBER) {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

/// The two machines and the cable tick; the cube is storage only
/// (`native_block_entity_ticks`, API 15).
#[allow(private_interfaces)]
pub fn on_block_entity_ticks(block: &str, ticks: &mut bool) {
    if matches!(block, SMELTER | CHAMBER | CABLE) {
        *ticks = true;
    }
}

/// The energy buffer of each block (`native_block_entity_energy`, API 142):
/// both machines take energy and give none, the cube stores 4 M FE at its
/// tier rate, the cable holds one throughput in transit.
#[allow(private_interfaces)]
pub fn on_block_entity_energy(block: &str, energy: &mut Option<ModEnergyBuffer>) {
    let all = [true; 6];
    let none = [false; 6];
    *energy = match block {
        SMELTER => Some(ModEnergyBuffer {
            capacity: SMELTER_CAPACITY,
            max_insert: SMELTER_CAPACITY,
            max_extract: 0,
            insert_faces: all,
            extract_faces: none,
        }),
        CHAMBER => Some(ModEnergyBuffer {
            capacity: CHAMBER_CAPACITY,
            max_insert: CHAMBER_CAPACITY,
            max_extract: 0,
            insert_faces: all,
            extract_faces: none,
        }),
        CUBE => Some(ModEnergyBuffer {
            capacity: CUBE_CAPACITY,
            max_insert: CUBE_RATE,
            max_extract: CUBE_RATE,
            insert_faces: all,
            extract_faces: all,
        }),
        CABLE => Some(ModEnergyBuffer {
            capacity: CABLE_CAPACITY,
            max_insert: CABLE_CAPACITY,
            max_extract: CABLE_CAPACITY,
            insert_faces: all,
            extract_faces: all,
        }),
        _ => None,
    };
}

/// A machine's three slots and the vanilla furnace menu
/// (`native_block_entity_container`, API 16/17). Hoppers may fill the input
/// and take the output; the middle slot is not reachable.
#[allow(private_interfaces)]
pub fn on_block_entity_container(block: &str, container: &mut Option<ModContainer>) {
    if !matches!(block, SMELTER | CHAMBER) {
        return;
    }
    let all = vec![INPUT_SLOT, 1, OUTPUT_SLOT];
    *container = Some(ModContainer {
        slots: MENU_SLOTS,
        faces: [
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all,
        ],
        inputs: vec![INPUT_SLOT],
        outputs: vec![OUTPUT_SLOT],
        menu: Some("minecraft:furnace".to_owned()),
    });
}

/// One tick of a machine or a cable (`native_block_entity_tick`, API 17).
#[allow(private_interfaces)]
pub fn on_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
) {
    match entity.id.as_str() {
        SMELTER => machine_tick(
            game,
            pos,
            entity,
            SMELTER_OPERATION,
            SMELTER_CAPACITY,
            &smelting::lookup,
        ),
        CHAMBER => machine_tick(
            game,
            pos,
            entity,
            CHAMBER_OPERATION,
            CHAMBER_CAPACITY,
            &enriching::lookup,
        ),
        CABLE => cable_tick(game, pos, entity),
        _ => {}
    }
}

/// One tick of a recipe machine: spend energy, advance the operation and
/// produce the recipe's output (upstream `CachedRecipe.process` through the
/// pure core). The smelter and the enrichment chamber differ only in their
/// operation and their recipe table.
fn machine_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    operation: Operation,
    capacity: i32,
    lookup: &dyn Fn(&str) -> Option<Recipe>,
) {
    // The container declares three slots; the engine sizes a mod entity's
    // slots when a container view is built, so a fresh machine does not yet
    // have them. Giving them their size here makes the slots real from the
    // first tick (a hopper, a menu or `ctl container` then finds them).
    if entity.items.len() < MENU_SLOTS {
        entity.items.resize(MENU_SLOTS, None);
    }
    let mut machine = Machine {
        energy: clamp_energy(be_energy(entity), capacity),
        progress: be_progress(entity).clamp(0, operation.ticks - 1),
    };
    let mut input = slot_stack(entity.items.get(INPUT_SLOT).and_then(Option::as_ref));
    let mut output = slot_stack(entity.items.get(OUTPUT_SLOT).and_then(Option::as_ref));
    // Upstream checks the output's room before it spends energy; the pure core
    // needs the result's stack size for that, and the result comes from the
    // recipe. Vanilla-style results stack to 64, so an empty slot takes that
    // and a filled one its own item's size.
    let max_stack = if output.is_empty() {
        64
    } else {
        game.native_max_stack_size(&output.item).unwrap_or(64).max(1)
    };
    let outcome = process_tick(
        &mut machine,
        operation,
        &mut input,
        &mut output,
        max_stack,
        lookup,
    );
    set_be_energy(entity, machine.energy);
    set_be_progress(entity, machine.progress);
    set_slot(&mut entity.items, INPUT_SLOT, &input);
    set_slot(&mut entity.items, OUTPUT_SLOT, &output);
    // The vanilla furnace menu's flame bar is the stored energy fraction and
    // its arrow the operation's progress.
    let flame = machine.energy.saturating_mul(operation.ticks) / capacity.max(1);
    if let Some(data) = entity.data.get_mut(..MENU_DATA) {
        data[0] = flame;
        data[1] = operation.ticks;
        data[2] = machine.progress;
        data[3] = operation.ticks;
    } else {
        entity.data = vec![flame, operation.ticks, machine.progress, operation.ticks];
    }
    // An idle tick with no energy or no room changes nothing; every other
    // outcome wrote energy, progress or items (an empty, unsmeltable or too
    // small input resets the progress).
    if !matches!(
        outcome,
        Outcome::Idle(IdleReason::NoEnergy | IdleReason::NoOutputSpace)
    ) {
        mark_modified(game, pos);
    }
}

/// The Basic Universal Cable: pull from the providers around it into its
/// buffer, then push into the acceptors, one throughput per tick.
fn cable_tick(game: &mut crate::game::Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let energy = clamp_energy(be_energy(entity), CABLE_CAPACITY);
    let dimension = game.native_dimension().to_owned();
    let mut endpoints = Vec::with_capacity(Direction::ALL.len());
    for face in Direction::ALL {
        let at = neighbour(pos, face);
        if let Some(found) = game.native_energy(&dimension, at, face.opposite()) {
            endpoints.push(Endpoint {
                face: face as u8,
                can_insert: found.can_insert,
                can_extract: found.can_extract,
                space: found.capacity.saturating_sub(found.amount),
                stored: found.amount,
            });
        }
    }
    let plan = cable_plan(energy, CABLE_CAPACITY, &endpoints);
    if plan.pulled() == 0 && plan.pushed() == 0 {
        return;
    }
    // Execute the plan and account with what the neighbours really moved.
    let mut current = energy;
    for change in &plan.moves {
        let (face, amount, pull) = match *change {
            Move::Pull { face, amount } => (face, amount, true),
            Move::Push { face, amount } => (face, amount, false),
        };
        let direction = Direction::ALL[face as usize];
        let at = neighbour(pos, direction);
        // The plan assumed every move lands; a neighbour may give less than it
        // said (its buffer was drained, its limit is lower), so each move is
        // capped by what the cable really holds or has room for.
        let limit = cable_move_limit(*change, current, CABLE_CAPACITY).min(amount);
        let moved = if pull {
            game.native_energy_extract(&dimension, at, direction.opposite(), limit, false)
        } else {
            game.native_energy_insert(at, direction.opposite(), limit, false)
        };
        current = if pull {
            current.saturating_add(moved)
        } else {
            current.saturating_sub(moved)
        };
    }
    let after = clamp_energy(current, CABLE_CAPACITY);
    // The planner's figure is what the neighbours should have allowed; the
    // stored one is what they really moved (a neighbour that ticked in between
    // answers less), so the measured amount is written and the plan is only
    // the invariant check.
    debug_assert_eq!(
        plan.energy_after,
        clamp_energy(energy + plan.pulled() - plan.pushed(), CABLE_CAPACITY),
        "the plan balances its own moves"
    );
    if after != energy {
        set_be_energy(entity, after);
        mark_modified(game, pos);
    }
}

/// `/mekanism info|energy|progress` (`register_commands`, API 8).
#[allow(private_interfaces)]
pub fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::new("mekanism info <pos:block_pos>"));
    commands.push(NativeCommand::op("mekanism energy <amount:integer> <pos:block_pos>", 2));
    commands.push(NativeCommand::new("mekanism progress <pos:block_pos>"));
}

/// A machine's energy buffer, 0 for a block this port gives none.
fn machine_capacity(id: &str) -> i32 {
    match id {
        SMELTER => SMELTER_CAPACITY,
        CHAMBER => CHAMBER_CAPACITY,
        CUBE => CUBE_CAPACITY,
        CABLE => CABLE_CAPACITY,
        _ => 0,
    }
}

/// Ticks one operation of a machine, 0 for a block that runs none.
fn machine_ticks(id: &str) -> i32 {
    match id {
        SMELTER => SMELTER_TICKS,
        CHAMBER => CHAMBER_TICKS,
        _ => 0,
    }
}

fn parse_pos(words: &[&str]) -> Option<[i32; 3]> {
    Some([
        words.first()?.parse().ok()?,
        words.get(1)?.parse().ok()?,
        words.get(2)?.parse().ok()?,
    ])
}

/// The `/mekanism` commands (`perform_command`, API 8). `info` prints a machine
/// and `energy` sets its stored FE, the harness controls the e2e uses;
/// `progress` prints a smelter's operation progress.
#[allow(private_interfaces)]
pub fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    _player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    match words.as_slice() {
        ["mekanism", "info", x, y, z] => {
            *ran = true;
            let Some(pos) = parse_pos(&[x, y, z]) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "Usage: /mekanism info <x> <y> <z>",
                ));
                return;
            };
            let Some(report) = game.native_with_mod_block_entity(pos, |entity| {
                let id = entity.id.clone();
                let energy = be_energy(entity);
                let progress = be_progress(entity);
                let input = slot_stack(entity.items.get(INPUT_SLOT).and_then(Option::as_ref));
                let output = slot_stack(entity.items.get(OUTPUT_SLOT).and_then(Option::as_ref));
                (id, energy, progress, input, output)
            }) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "No block entity there.",
                ));
                return;
            };
            let (id, energy, progress, input, output) = report;
            let capacity = machine_capacity(&id);
            let ticks = machine_ticks(&id);
            let text = format!(
                "{id}: {energy}/{capacity} FE, progress {progress}/{ticks}, input {} x{}, output {} x{}",
                if input.is_empty() { "empty" } else { &input.item },
                input.count,
                if output.is_empty() { "empty" } else { &output.item },
                output.count,
            );
            game.native_feedback(horizon_client::text::Text::literal(text));
        }
        ["mekanism", "energy", amount, x, y, z] => {
            *ran = true;
            let Some(pos) = parse_pos(&[x, y, z]) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "Usage: /mekanism energy <amount> <x> <y> <z>",
                ));
                return;
            };
            let Ok(amount) = amount.parse::<i32>() else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "The amount must be a whole number of FE.",
                ));
                return;
            };
            let Some(after) = game.native_with_mod_block_entity(pos, |entity| {
                let energy = clamp_energy(amount, machine_capacity(&entity.id));
                set_be_energy(entity, energy);
                energy
            }) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "No block entity there.",
                ));
                return;
            };
            game.native_feedback(horizon_client::text::Text::literal(format!(
                "Energy set to {after} FE."
            )));
        }
        ["mekanism", "progress", x, y, z] => {
            *ran = true;
            let Some(pos) = parse_pos(&[x, y, z]) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "Usage: /mekanism progress <x> <y> <z>",
                ));
                return;
            };
            let Some((progress, ticks)) = game.native_with_mod_block_entity(pos, |entity| {
                (be_progress(entity), machine_ticks(&entity.id))
            }) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "No block entity there.",
                ));
                return;
            };
            game.native_feedback(horizon_client::text::Text::literal(format!(
                "Progress {progress}/{ticks}."
            )));
        }
        _ => {}
    }
}
