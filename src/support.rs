//! The glue: the five block entity types, their containers and energy buffers,
//! the machines' tick, the cable's transfers and the `/mekanism` command bodies.
//!
//! Every rule lives in [`crate::runtime`] (the retired bundle's `runtime.rs`,
//! byte-identical); this file reads the game, calls it and writes the result
//! back. Its tests pin what it *declares* (three slots, the buffers), that the
//! state it writes reads back, and the exact lines `/mekanism` answers.
use mod_api::{
    BlockEntityPart, BlockEntityTickEvent, BlockEntityTickReply, BlockEntityTicksEvent, BlockPart,
    Capability, Compound, Direction, ENERGY, EnergyHandler, Host, HostTransaction, Identifier,
    ItemStack, ModContainer, ModEnergyBuffer, Registrar, RegistrationContext, ServerHost, Tag,
};

use crate::runtime::{
    CABLE_CAPACITY, CHAMBER_CAPACITY, CHAMBER_OPERATION, CHAMBER_TICKS, CRUSHER_CAPACITY,
    CRUSHER_OPERATION, CRUSHER_TICKS, CUBE_CAPACITY, CUBE_RATE, IdleReason, ItemAmount, Machine,
    MachineRecipe, Operation, Outcome, SMELTER_CAPACITY, SMELTER_OPERATION, SMELTER_TICKS,
    clamp_energy, crushing, energy_after_extract, energy_after_insert, enriching, process_tick,
    smelting,
};

/// The five blocks of slices 1-3.
pub(crate) const SMELTER: &str = "mekanism:energized_smelter";
pub(crate) const CUBE: &str = "mekanism:basic_energy_cube";
pub(crate) const CABLE: &str = "mekanism:basic_universal_cable";
pub(crate) const CHAMBER: &str = "mekanism:enrichment_chamber";
pub(crate) const CRUSHER: &str = "mekanism:crusher";

/// The five block entity types, in this mod's namespace (`/mekanism` has no
/// bearing on them). One type per block, as the retired bundle's
/// `block_entity` hook named the block it was asked about.
const SMELTER_TYPE: &str = "energized_smelter";
const CUBE_TYPE: &str = "basic_energy_cube";
const CABLE_TYPE: &str = "basic_universal_cable";
const CHAMBER_TYPE: &str = "enrichment_chamber";
const CRUSHER_TYPE: &str = "crusher";

/// A machine's menu slots: input, unused (the furnace menu's fuel slot) and
/// output.
const INPUT_SLOT: usize = 0;
const UNUSED_SLOT: usize = 1;
const OUTPUT_SLOT: usize = 2;

/// How many item slots a machine's container declares (the furnace menu's three).
const MENU_SLOTS: usize = 3;

/// The vanilla furnace menu's four data slots: flame (stored / capacity) then
/// arrow (progress / required ticks).
const MENU_DATA: usize = 4;

/// An empty output slot takes a vanilla stack; a filled one its own item's size.
const DEFAULT_STACK_LIMIT: u32 = 64;

/// One machine's registration: the block, its block entity type's path, its
/// operation, its buffer's capacity and its recipe table.
struct MachineDecl {
    block: &'static str,
    kind: &'static str,
    operation: Operation,
    capacity: i32,
    lookup: fn(&str) -> Option<MachineRecipe>,
}

/// Declare the five blocks: the three machines' containers and buffers, the
/// cube's and the cable's buffers, and a block entity type each. The four that
/// tick carry a Part; the cube is storage only.
pub(crate) fn declare(registrar: &mut dyn Registrar, context: &RegistrationContext) {
    let machines = [
        MachineDecl {
            block: SMELTER,
            kind: SMELTER_TYPE,
            operation: SMELTER_OPERATION,
            capacity: SMELTER_CAPACITY,
            lookup: smelting::lookup,
        },
        MachineDecl {
            block: CHAMBER,
            kind: CHAMBER_TYPE,
            operation: CHAMBER_OPERATION,
            capacity: CHAMBER_CAPACITY,
            lookup: enriching::lookup,
        },
        MachineDecl {
            block: CRUSHER,
            kind: CRUSHER_TYPE,
            operation: CRUSHER_OPERATION,
            capacity: CRUSHER_CAPACITY,
            lookup: crushing::lookup,
        },
    ];
    for machine in machines {
        let key = registrar.block_key(machine.block);
        registrar.block_container(&key, container());
        registrar.block_energy(&key, machine_battery(machine.capacity));
        // The type must be declared before a Part is attached to it, and it owns
        // the block it is declared for: a foreign block is another key.
        let kind = registrar
            .block_entity_type(machine.kind)
            .blocks(&[machine.kind])
            .build();
        let part = MachinePart {
            operation: machine.operation,
            capacity: machine.capacity,
            lookup: machine.lookup,
        };
        if let Err(error) = registrar.block_entity_part(&kind, part) {
            context.refuse(&error.to_string());
        }
    }
    // The cube is storage only: a buffer and a block entity, no tick and no slots.
    let cube = registrar.block_key(CUBE);
    registrar.block_energy(&cube, cube_battery());
    registrar
        .block_entity_type(CUBE_TYPE)
        .blocks(&[CUBE_TYPE])
        .build();
    let cable = registrar.block_key(CABLE);
    registrar.block_energy(&cable, cable_battery());
    let kind = registrar
        .block_entity_type(CABLE_TYPE)
        .blocks(&[CABLE_TYPE])
        .build();
    if let Err(error) = registrar.block_entity_part(&kind, CablePart) {
        context.refuse(&error.to_string());
    }
}

/// The smelter's and the chamber's container: three slots every face reaches,
/// input and output declared, shown as the vanilla furnace menu.
pub(crate) fn container() -> ModContainer {
    let all = vec![INPUT_SLOT, UNUSED_SLOT, OUTPUT_SLOT];
    ModContainer {
        slots: MENU_SLOTS,
        faces: std::array::from_fn(|_| all.clone()),
        inputs: vec![INPUT_SLOT],
        outputs: vec![OUTPUT_SLOT],
        menu: Some("minecraft:furnace".to_owned()),
    }
}

/// A machine's FE buffer: `capacity` that may be filled from any face and
/// drained by none (upstream's machines take energy and give none).
pub(crate) fn machine_battery(capacity: i32) -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity,
        max_insert: capacity,
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
    }
}

/// The Basic Energy Cube's buffer: 4,000,000 FE moved at the tier's 4,000 FE/t.
pub(crate) fn cube_battery() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: CUBE_CAPACITY,
        max_insert: CUBE_RATE,
        max_extract: CUBE_RATE,
        insert_faces: [true; 6],
        extract_faces: [true; 6],
    }
}

/// The Basic Universal Cable's buffer: one 8,000 FE throughput in transit.
pub(crate) fn cable_battery() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: CABLE_CAPACITY,
        max_insert: CABLE_CAPACITY,
        max_extract: CABLE_CAPACITY,
        insert_faces: [true; 6],
        extract_faces: [true; 6],
    }
}

/// A machine's buffer capacity, `None` for a block this port gives none.
pub(crate) fn machine_capacity(id: &str) -> Option<i32> {
    match id {
        SMELTER => Some(SMELTER_CAPACITY),
        CHAMBER => Some(CHAMBER_CAPACITY),
        CRUSHER => Some(CRUSHER_CAPACITY),
        CUBE => Some(CUBE_CAPACITY),
        CABLE => Some(CABLE_CAPACITY),
        _ => None,
    }
}

/// Ticks one operation of a machine, `None` for a block that runs none.
pub(crate) fn machine_ticks(id: &str) -> Option<i32> {
    match id {
        SMELTER => Some(SMELTER_TICKS),
        CHAMBER => Some(CHAMBER_TICKS),
        CRUSHER => Some(CRUSHER_TICKS),
        _ => None,
    }
}

/// A saved int, 0 when absent or of another type.
fn int(saved: &Tag, key: &str) -> i32 {
    match saved.get(key) {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    }
}

/// A machine's saved state: `energy` clamped into its buffer, and `progress` as
/// saved (the tick clamps it to the operation, which it knows).
pub(crate) fn read_machine(saved: &Tag, capacity: i32) -> Machine {
    Machine {
        energy: clamp_energy(int(saved, "energy"), capacity),
        progress: read_progress(saved),
    }
}

/// A saved buffer's amount, clamped into `capacity`.
pub(crate) fn read_energy(saved: &Tag, capacity: i32) -> i32 {
    clamp_energy(int(saved, "energy"), capacity)
}

/// A machine's saved progress in ticks, 0 when absent or of another type.
pub(crate) fn read_progress(saved: &Tag) -> i32 {
    int(saved, "progress").max(0)
}

/// Write a buffer's amount: the int `energy` the energy capability reads.
pub(crate) fn write_energy(saved: &mut Tag, energy: i32) {
    if let Tag::Compound(nbt) = saved {
        nbt.insert("energy".into(), Tag::Int(energy));
    }
}

/// Write the tick's result into the saved compound: the buffer's amount
/// (`energy`, the int the energy capability reads) and the operation's
/// progress.
pub(crate) fn write_state(saved: &mut Tag, energy: i32, progress: i32) {
    write_energy(saved, energy);
    if let Tag::Compound(nbt) = saved {
        nbt.insert("progress".into(), Tag::Int(progress));
    }
}

/// The `/mekanism info` report line for one machine.
pub(crate) fn info_text(
    id: &str,
    capacity: i32,
    ticks: i32,
    machine: Machine,
    input: &ItemAmount,
    output: &ItemAmount,
) -> String {
    fn name(stack: &ItemAmount) -> String {
        if stack.is_empty() {
            "empty".to_owned()
        } else {
            stack.item.clone()
        }
    }
    format!(
        "{id}: {}/{} FE, progress {}/{}, input {} x{}, output {} x{}",
        machine.energy,
        capacity,
        machine.progress,
        ticks,
        name(input),
        input.count,
        name(output),
        output.count
    )
}

/// A slot as the pure core's stack, empty when the slot is empty.
fn slot_stack(host: &dyn Host, slot: Option<&ItemStack>) -> ItemAmount {
    match slot {
        Some(stack) if stack.count() > 0 => ItemAmount::new(&stack.item().id(host), stack.count()),
        _ => ItemAmount::default(),
    }
}

/// Write the pure core's stack back into `slot`, keeping the previous stack's
/// components when it is the same item.
fn set_slot(host: &dyn Host, items: &mut [Option<ItemStack>], slot: usize, value: &ItemAmount) {
    if slot >= items.len() {
        return;
    }
    if value.is_empty() {
        items[slot] = None;
        return;
    }
    let same = items[slot]
        .as_ref()
        .is_some_and(|current| current.item().id(host) == value.item);
    if same {
        if let Some(current) = items[slot].as_mut() {
            let _ = current.set_count(value.count);
        }
        return;
    }
    items[slot] = Identifier::parse(&value.item)
        .ok()
        .and_then(|id| host.item_key(&id))
        .and_then(|key| ItemStack::new(key, value.count));
}

/// How many of the output's item one slot holds (`Item.getDefaultMaxStackSize`),
/// the vanilla 64 for an unknown item or an empty slot.
fn stack_limit(host: &dyn Host, output: &ItemAmount) -> u32 {
    if output.is_empty() {
        return DEFAULT_STACK_LIMIT;
    }
    Identifier::parse(&output.item)
        .ok()
        .and_then(|id| host.item_key(&id))
        .and_then(|key| host.max_stack_size(key))
        .unwrap_or(DEFAULT_STACK_LIMIT)
        .max(1)
}

/// The furnace menu's four data slots: the stored energy as the flame's
/// fraction and the operation's progress as the arrow's.
fn write_menu_data(
    data: &mut Vec<i32>,
    energy: i32,
    progress: i32,
    operation: Operation,
    capacity: i32,
) {
    let flame = energy.saturating_mul(operation.ticks) / capacity.max(1);
    // Grow only: a wider vector belongs to another Part, and its slots are not ours to drop.
    if data.len() < MENU_DATA {
        data.resize(MENU_DATA, 0);
    }
    data[0] = flame;
    data[1] = operation.ticks;
    data[2] = progress;
    data[3] = operation.ticks;
}

/// A machine's behaviour (smelter, chamber or crusher): a ticked block entity.
pub(crate) struct MachinePart {
    /// The operation this machine runs.
    pub(crate) operation: Operation,
    /// Its buffer's capacity.
    pub(crate) capacity: i32,
    /// The recipe table it reads (`smelting`, `enriching` or `crushing`).
    pub(crate) lookup: fn(&str) -> Option<MachineRecipe>,
}

#[mod_api::horizon_part]
impl BlockPart for MachinePart {}

#[mod_api::horizon_part]
impl BlockEntityPart for MachinePart {
    /// `EntityBlock.getTicker`: the machine ticks. Never clear a value an
    /// earlier mod set.
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick of the operation: spend energy, advance the progress and
    /// produce the recipe's output (upstream `CachedRecipe.process` through the
    /// pure core). An idle tick with no energy or no output room changes
    /// nothing, so it answers `Unchanged` and does not dirty its chunk (the
    /// retired bundle's `mark_modified` condition).
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let resized = event.entity.items.len() != MENU_SLOTS;
        if resized {
            event.entity.items.resize(MENU_SLOTS, None);
        }
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut machine = read_machine(&saved, self.capacity);
        machine.progress = machine.progress.clamp(0, (self.operation.ticks - 1).max(0));
        let mut input = slot_stack(&*host, event.entity.items[INPUT_SLOT].as_ref());
        let mut output = slot_stack(&*host, event.entity.items[OUTPUT_SLOT].as_ref());
        let limit = stack_limit(&*host, &output);
        let outcome = process_tick(
            &mut machine,
            self.operation,
            &mut input,
            &mut output,
            limit,
            &self.lookup,
        );
        write_state(&mut saved, machine.energy, machine.progress);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        set_slot(&*host, &mut event.entity.items, INPUT_SLOT, &input);
        set_slot(&*host, &mut event.entity.items, OUTPUT_SLOT, &output);
        write_menu_data(
            &mut event.entity.data,
            machine.energy,
            machine.progress,
            self.operation,
            self.capacity,
        );
        let idle = matches!(
            outcome,
            Outcome::Idle(IdleReason::NoEnergy | IdleReason::NoOutputSpace)
        );
        if idle && !resized {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// The Basic Universal Cable's behaviour: a ticked block entity that moves
/// energy between the block entities beside it.
pub(crate) struct CablePart;

#[mod_api::horizon_part]
impl BlockPart for CablePart {}

#[mod_api::horizon_part]
impl BlockEntityPart for CablePart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One cable tick: pull from every provider around it into its buffer, then
    /// push into every acceptor, one throughput in or out (upstream's
    /// transmitter).
    ///
    /// Each move is **measured**: the SDK's energy capability answers a side's
    /// buffer with its amount and capacity but not with its per-call insert and
    /// extract limits, so the runtime's arithmetic proposes the amount and the
    /// handler's own `extract`/`insert` answers what really moved. The retired
    /// bundle planned every face first, which needs the neighbour's
    /// `can_extract`; without it a machine's stored energy would be planned as a
    /// pull that yields nothing and would starve the face that does give.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let pos = event.pos;
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let energy = read_energy(&saved, CABLE_CAPACITY);
        let Some(level) = host.level(host.dimension()) else {
            return;
        };
        let mut neighbours: Vec<Capability<dyn EnergyHandler>> = Vec::with_capacity(6);
        for face in Direction::ALL {
            let [dx, dy, dz] = face.normal();
            let at = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            if let Some(handler) = level.capability(host, &ENERGY, at, Some(face.opposite())) {
                neighbours.push(handler);
            }
        }
        if neighbours.is_empty() {
            return;
        }
        let after = host.transaction(move |host, tx| {
            let mut current = energy;
            for handler in &neighbours {
                let room = CABLE_CAPACITY.saturating_sub(current);
                let stored = to_i32(handler.amount(host));
                let (want, _) = energy_after_extract(stored, room, room);
                if want <= 0 {
                    continue;
                }
                let taken = to_i32(handler.extract(host, tx, to_u32(want)));
                current = current.saturating_add(taken);
            }
            for handler in &neighbours {
                if current <= 0 {
                    break;
                }
                let capacity = to_i32(handler.capacity(host));
                let stored = to_i32(handler.amount(host));
                let space = capacity.saturating_sub(stored);
                let (want, _) = energy_after_insert(0, space, space, current);
                if want <= 0 {
                    continue;
                }
                let given = to_i32(handler.insert(host, tx, to_u32(want)));
                current = current.saturating_sub(given);
            }
            Ok::<i32, ()>(current)
        });
        let Ok(after) = after else {
            return;
        };
        let after = clamp_energy(after, CABLE_CAPACITY);
        if after != energy {
            write_energy(&mut saved, after);
            if let Tag::Compound(nbt) = saved {
                event.entity.nbt = nbt;
            }
        }
    }
}

/// A capability's amount as the runtime's `i32` (it never exceeds a buffer's
/// capacity, which is an `i32` here).
fn to_i32(amount: u32) -> i32 {
    i32::try_from(amount).unwrap_or(i32::MAX)
}

/// A wanted amount as the capability's `u32` (positive, by every caller).
fn to_u32(amount: i32) -> u32 {
    u32::try_from(amount.max(0)).unwrap_or(0)
}

/// The id and buffer capacity of one of this port's blocks at `pos`: `None`
/// when nothing stands there or the block is not one of the four.
fn our_block_at(host: &mut dyn ServerHost, pos: [i32; 3]) -> Option<(String, i32)> {
    let level = host.level(host.dimension())?;
    let id = level.block(&*host, pos)?.id(&*host);
    let capacity = machine_capacity(&id)?;
    Some((id, capacity))
}

/// `/mekanism info <pos>`: one machine's buffer, progress and slots.
pub(crate) fn info_line(host: &mut dyn ServerHost, pos: [i32; 3]) -> Option<String> {
    let (id, capacity) = our_block_at(host, pos)?;
    let level = host.level(host.dimension())?;
    let entity = host.block_entity(level, pos)?;
    let saved = Tag::Compound(entity.data(&*host)?);
    let input = slot_stack(
        &*host,
        entity
            .item_slot(&*host, INPUT_SLOT as u32)
            .ok()
            .flatten()
            .as_ref(),
    );
    let output = slot_stack(
        &*host,
        entity
            .item_slot(&*host, OUTPUT_SLOT as u32)
            .ok()
            .flatten()
            .as_ref(),
    );
    Some(info_text(
        &id,
        capacity,
        machine_ticks(&id).unwrap_or(0),
        read_machine(&saved, capacity),
        &input,
        &output,
    ))
}

/// `/mekanism progress <pos>`: a machine's operation progress.
pub(crate) fn progress_line(host: &mut dyn ServerHost, pos: [i32; 3]) -> Option<String> {
    let (id, _) = our_block_at(host, pos)?;
    let level = host.level(host.dimension())?;
    let entity = host.block_entity(level, pos)?;
    let saved = Tag::Compound(entity.data(&*host)?);
    Some(format!(
        "Progress {}/{}.",
        read_progress(&saved),
        machine_ticks(&id).unwrap_or(0)
    ))
}

/// `/mekanism energy <amount> <pos>`: set a machine's stored FE, the harness
/// control the in-game check uses. A foreign block entity is left completely
/// alone: writing `energy` into a chest's saved data would be a foreign key in
/// someone else's compound.
pub(crate) fn set_energy(host: &mut dyn ServerHost, pos: [i32; 3], amount: i32) -> Option<i32> {
    let (_, capacity) = our_block_at(host, pos)?;
    let level = host.level(host.dimension())?;
    let entity = host.block_entity(level, pos)?;
    let energy = clamp_energy(amount, capacity);
    let mut patch = Compound::new();
    patch.insert("energy".into(), Tag::Int(energy));
    // `merge_data` is `/data merge block`: it loads the patch and marks the chunk
    // changed, which is the `setChanged` the retired bundle had to fake.
    entity.merge_data(host, patch).ok().map(|_| energy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_machines_declare_three_slots_with_input_and_output_on_every_face() {
        let container = container();
        assert_eq!(
            container.slots, MENU_SLOTS,
            "the furnace menu's three slots"
        );
        assert_eq!(
            container.inputs,
            vec![INPUT_SLOT],
            "automation fills the input"
        );
        assert_eq!(container.outputs, vec![OUTPUT_SLOT], "and takes the output");
        assert_eq!(
            container.faces,
            std::array::from_fn(|_| vec![INPUT_SLOT, UNUSED_SLOT, OUTPUT_SLOT]),
            "every face reaches all three slots: there is no side configuration"
        );
        assert_eq!(
            container.menu,
            Some("minecraft:furnace".to_owned()),
            "the vanilla furnace menu draws the flame and arrow bars"
        );
    }

    #[test]
    fn a_machine_takes_energy_from_every_face_and_gives_none() {
        let buffer = machine_battery(SMELTER_CAPACITY);
        assert_eq!(buffer.capacity, SMELTER_CAPACITY);
        assert_eq!(
            buffer.max_insert, SMELTER_CAPACITY,
            "one call may move the whole remaining room"
        );
        assert_eq!(buffer.max_extract, 0, "a machine gives no energy back");
        assert_eq!(buffer.insert_faces, [true; 6]);
        assert_eq!(buffer.extract_faces, [false; 6]);
    }

    #[test]
    fn the_cube_and_the_cable_declare_their_own_buffers() {
        let cube = cube_battery();
        assert_eq!(
            (cube.capacity, cube.max_insert, cube.max_extract),
            (CUBE_CAPACITY, CUBE_RATE, CUBE_RATE)
        );
        assert_eq!(
            cube.insert_faces, [true; 6],
            "the cube is filled and drained on every face"
        );
        assert_eq!(cube.extract_faces, [true; 6]);
        let cable = cable_battery();
        assert_eq!(
            (cable.capacity, cable.max_insert, cable.max_extract),
            (CABLE_CAPACITY, CABLE_CAPACITY, CABLE_CAPACITY)
        );
    }

    #[test]
    fn only_the_five_blocks_have_a_capacity_and_only_the_machines_a_length() {
        assert_eq!(machine_capacity(SMELTER), Some(SMELTER_CAPACITY));
        assert_eq!(machine_capacity(CHAMBER), Some(CHAMBER_CAPACITY));
        assert_eq!(machine_capacity(CRUSHER), Some(CRUSHER_CAPACITY));
        assert_eq!(machine_capacity(CUBE), Some(CUBE_CAPACITY));
        assert_eq!(machine_capacity(CABLE), Some(CABLE_CAPACITY));
        assert_eq!(machine_capacity("minecraft:chest"), None);
        assert_eq!(machine_ticks(SMELTER), Some(200));
        assert_eq!(machine_ticks(CHAMBER), Some(200));
        assert_eq!(machine_ticks(CRUSHER), Some(200));
        assert_eq!(machine_ticks(CUBE), None, "the cube runs no operation");
        assert_eq!(machine_ticks(CABLE), None);
    }

    /// The crusher is a machine in the same shape as the smelter and the
    /// chamber: a three-slot container, an insert-only 20,000 FE buffer and a
    /// 200-tick operation, differing in its recipe table.
    #[test]
    fn the_crusher_is_a_machine_with_the_same_shape_as_the_other_two() {
        let container = container();
        assert_eq!(
            container.slots, MENU_SLOTS,
            "the crusher shows the same three-slot furnace menu"
        );
        let buffer = machine_battery(CRUSHER_CAPACITY);
        assert_eq!((buffer.capacity, buffer.max_extract), (20_000, 0));
        assert_eq!(
            info_text(
                CRUSHER,
                CRUSHER_CAPACITY,
                CRUSHER_OPERATION.ticks,
                Machine {
                    energy: 10_000,
                    progress: 0
                },
                &ItemAmount::default(),
                &ItemAmount::new("minecraft:gravel", 1)
            ),
            "mekanism:crusher: 10000/20000 FE, progress 0/200, input empty x0, \
             output minecraft:gravel x1"
        );
    }

    #[test]
    fn a_fresh_compound_reads_as_an_empty_machine() {
        let saved = Tag::Compound(Compound::new());
        assert_eq!(read_machine(&saved, SMELTER_CAPACITY), Machine::default());
        assert_eq!(read_progress(&saved), 0);
    }

    #[test]
    fn the_state_the_tick_writes_reads_back_from_the_compound() {
        let mut saved = Tag::Compound(Compound::new());
        write_state(&mut saved, 12_345, 60);
        assert_eq!(
            read_machine(&saved, SMELTER_CAPACITY),
            Machine {
                energy: 12_345,
                progress: 60
            }
        );
        assert_eq!(read_progress(&saved), 60);
        assert_eq!(
            saved.get("energy"),
            Some(&Tag::Int(12_345)),
            "the buffer is the int `energy` the energy capability reads"
        );
    }

    #[test]
    fn a_buffer_read_is_clamped_into_its_capacity() {
        let mut saved = Tag::Compound(Compound::new());
        write_energy(&mut saved, 25_000);
        assert_eq!(
            read_energy(&saved, SMELTER_CAPACITY),
            SMELTER_CAPACITY,
            "a stored amount above the capacity reads as the capacity"
        );
        write_energy(&mut saved, -5);
        assert_eq!(read_energy(&saved, SMELTER_CAPACITY), 0);
    }

    #[test]
    fn the_menu_bars_never_shrink_a_wider_data_vector() {
        let mut data = vec![-1, -2, -3, -4, 77, 88];
        write_menu_data(&mut data, 10_000, 60, SMELTER_OPERATION, SMELTER_CAPACITY);
        assert_eq!(
            data,
            vec![100, 200, 60, 200, 77, 88],
            "the first four slots are the furnace menu's; a wider vector keeps its extra slots"
        );
        let mut empty: Vec<i32> = Vec::new();
        write_menu_data(&mut empty, 0, 0, SMELTER_OPERATION, SMELTER_CAPACITY);
        assert_eq!(
            empty,
            vec![0, 200, 0, 200],
            "a fresh entity gets the four slots"
        );
    }

    #[test]
    fn the_info_line_is_the_one_the_e2e_asserts() {
        let empty = ItemAmount::default();
        assert_eq!(
            info_text(
                SMELTER,
                SMELTER_CAPACITY,
                SMELTER_OPERATION.ticks,
                Machine {
                    energy: 5_450,
                    progress: 91
                },
                &empty,
                &empty
            ),
            "mekanism:energized_smelter: 5450/20000 FE, progress 91/200, input empty x0, \
             output empty x0"
        );
        assert_eq!(
            info_text(
                CHAMBER,
                CHAMBER_CAPACITY,
                CHAMBER_OPERATION.ticks,
                Machine::default(),
                &ItemAmount::new("minecraft:iron_ore", 1),
                &ItemAmount::new("mekanism:dust_iron", 2)
            ),
            "mekanism:enrichment_chamber: 0/20000 FE, progress 0/200, input minecraft:iron_ore x1, \
             output mekanism:dust_iron x2"
        );
    }
}
