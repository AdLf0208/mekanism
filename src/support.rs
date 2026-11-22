//! The glue: the five block entity types, their containers and energy buffers,
//! the machines' tick, the cable's transfers and the `/mekanism` command bodies.
//!
//! Every rule lives in [`crate::runtime`] (the retired bundle's `runtime.rs`,
//! byte-identical); this file reads the game, calls it and writes the result
//! back. Its tests pin what it *declares* (three slots, the buffers), that the
//! state it writes reads back, and the exact lines `/mekanism` answers.
use mod_api::{
    BlockEntityPart, BlockEntityTickEvent, BlockEntityTickReply, BlockEntityTicksEvent, BlockPart,
    BlockSide, Capability, CommandSource, Compound, Direction, ENERGY, EnergyHandler, FLUIDS,
    FluidKey, FluidResource, FluidStack, Hand, Host, HostTransaction, Identifier,
    ItemComponentHandler, ItemPlace, ItemStack, ModContainer, ModEnergyBuffer, ModFluidTanks,
    Registrar, RegistrationContext, ResourceHandler, ServerHost, Tag, Transaction,
};

use crate::chemical::{CHEMICAL, Chemical, ITEM_CHEMICAL, ITEM_COMPONENT};
use crate::runtime::{
    CABLE_CAPACITY, CHAMBER_CAPACITY, CHAMBER_OPERATION, CHAMBER_TICKS, CHEMICALS,
    CRUSHER_CAPACITY, CRUSHER_OPERATION, CRUSHER_TICKS, CRUSHING_FACTORY_CAPACITY, CUBE_CAPACITY,
    CUBE_RATE, DROPPER_CAPACITY, EMPTY_CHEMICAL, ENRICHING_FACTORY_CAPACITY, FACTORY_PROCESSES,
    FACTORY_TICKS, FactoryProcess, FluidTank, GasRecipe, INJECTING_FACTORY_CAPACITY,
    INJECTING_FACTORY_TANK, INJECTION_CAPACITY, INJECTION_OPERATION, INJECTION_TANK,
    INJECTION_TICKS, IdleReason, ItemAmount, Machine, MachineRecipe, Operation, Outcome,
    PURIFY_CAPACITY, PURIFY_OPERATION, PURIFY_TANK, PURIFY_TICKS, PURIFYING_FACTORY_CAPACITY,
    PURIFYING_FACTORY_TANK, SLURRIES, SMELTER_CAPACITY, SMELTER_OPERATION, SMELTER_TICKS,
    SMELTING_FACTORY_CAPACITY, TANK_CAPACITY, Tank, WASHER_CAPACITY, WASHER_OPERATION,
    WASHER_SLURRY, WASHER_TICKS, WASHER_WATER, WATER, clamp_energy, crushing, dropper_move,
    energy_after_extract, energy_after_insert, enriching, extract_chemical, factory_gas_want,
    gas_pull, injecting, insert_chemical, operations_per_tick, process_factory_tick,
    process_gas_factory_tick, process_gas_tick, process_tick, process_wash_tick, purifying,
    room_for, room_in_tank, smelting, washer_pull, washing,
};

/// The five blocks of slices 1-3.
pub(crate) const SMELTER: &str = "mekanism:energized_smelter";
pub(crate) const CUBE: &str = "mekanism:basic_energy_cube";
pub(crate) const CABLE: &str = "mekanism:basic_universal_cable";
pub(crate) const CHAMBER: &str = "mekanism:enrichment_chamber";
pub(crate) const CRUSHER: &str = "mekanism:crusher";

/// Slice 4's one block and one item: the Basic Chemical Tank, whose one tank
/// answers [`CHEMICAL`], and the Gauge Dropper, the mod's own chemical container.
pub(crate) const CHEMICAL_TANK: &str = "mekanism:basic_chemical_tank";
pub(crate) const GAUGE_DROPPER: &str = "mekanism:gauge_dropper";

/// Slice 5's machine: the consumer of the chemical capability.
pub(crate) const INJECTION: &str = "mekanism:chemical_injection_chamber";

/// Slice 6's machine: the same gas machine on oxygen.
pub(crate) const PURIFICATION: &str = "mekanism:purification_chamber";

/// Slice 7's five machines: the basic factory of each item machine this port
/// runs (`FactoryType.SMELTING`, `ENRICHING`, `CRUSHING`, `INJECTING`,
/// `PURIFYING`).
pub(crate) const SMELTING_FACTORY: &str = "mekanism:basic_smelting_factory";
pub(crate) const ENRICHING_FACTORY: &str = "mekanism:basic_enriching_factory";
pub(crate) const CRUSHING_FACTORY: &str = "mekanism:basic_crushing_factory";
pub(crate) const INJECTING_FACTORY: &str = "mekanism:basic_injecting_factory";
pub(crate) const PURIFYING_FACTORY: &str = "mekanism:basic_purifying_factory";

/// Slice 8's machine: the Chemical Washer, the port's first machine whose input
/// is a **fluid** (water) beside a chemical (dirty slurry).
pub(crate) const WASHER: &str = "mekanism:chemical_washer";

/// The block entity types, in this mod's namespace (`/mekanism` has no
/// bearing on them). One type per block, as the retired bundle's
/// `block_entity` hook named the block it was asked about.
const SMELTER_TYPE: &str = "energized_smelter";
const CUBE_TYPE: &str = "basic_energy_cube";
const CABLE_TYPE: &str = "basic_universal_cable";
const CHAMBER_TYPE: &str = "enrichment_chamber";
const CRUSHER_TYPE: &str = "crusher";
const TANK_TYPE: &str = "basic_chemical_tank";
const INJECTION_TYPE: &str = "chemical_injection_chamber";
const PURIFICATION_TYPE: &str = "purification_chamber";
const SMELTING_FACTORY_TYPE: &str = "basic_smelting_factory";
const ENRICHING_FACTORY_TYPE: &str = "basic_enriching_factory";
const CRUSHING_FACTORY_TYPE: &str = "basic_crushing_factory";
const INJECTING_FACTORY_TYPE: &str = "basic_injecting_factory";
const PURIFYING_FACTORY_TYPE: &str = "basic_purifying_factory";
const WASHER_TYPE: &str = "chemical_washer";

/// A machine's menu slots: input, unused (the furnace menu's fuel slot) and
/// output.
const INPUT_SLOT: usize = 0;
const UNUSED_SLOT: usize = 1;
const OUTPUT_SLOT: usize = 2;

/// How many item slots a machine's container declares (the furnace menu's three).
const MENU_SLOTS: usize = 3;

/// A factory's slots: one input and one output per process, the inputs first so
/// that automation can fill a process without knowing the layout.
const FACTORY_SLOTS: usize = FACTORY_PROCESSES * 2;
const FACTORY_INPUT_SLOTS: [usize; FACTORY_PROCESSES] = [0, 1, 2];
const FACTORY_OUTPUT_SLOTS: [usize; FACTORY_PROCESSES] = [3, 4, 5];

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

/// One gas machine's registration: the block, its block entity type's path, its
/// operation, its buffer's capacity, its own tank's capacity and its table.
///
/// The Chemical Injection Chamber and the Purification Chamber differ only in
/// these values, so they are declared as two rows of one table (a five-field
/// tuple trips `clippy::type_complexity`, and the loop then reads as the table
/// it is).
struct GasDecl {
    block: &'static str,
    kind: &'static str,
    operation: Operation,
    capacity: i32,
    tank: u32,
    recipe_for: fn(&str) -> Option<GasRecipe>,
}

/// One basic factory's registration: the block, its block entity type's path,
/// its operation, its buffer, and the table its processes read.
struct FactoryDecl {
    block: &'static str,
    kind: &'static str,
    operation: Operation,
    capacity: i32,
    lookup: fn(&str) -> Option<MachineRecipe>,
}

/// A gas factory's registration: the same, plus its one shared tank.
struct GasFactoryDecl {
    block: &'static str,
    kind: &'static str,
    operation: Operation,
    capacity: i32,
    tank: u32,
    recipe_for: fn(&str) -> Option<GasRecipe>,
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
    // Slice 4: one block entity holding one tank of one chemical, answered
    // through the port's own capability, and the item that carries a chemical.
    // No container, no energy buffer and no tick: nothing here moves on its own,
    // so a capability call is the only writer (no tick write-back to lose it to).
    let kind = registrar
        .block_entity_type(TANK_TYPE)
        .blocks(&[TANK_TYPE])
        .build();
    registrar.block_entity_capability(&CHEMICAL, &kind, |_, at| {
        Some(Box::new(ChemicalTank {
            side: at,
            capacity: TANK_CAPACITY,
        }) as Box<dyn ResourceHandler<Chemical>>)
    });
    registrar.item_capability(&ITEM_CHEMICAL, GAUGE_DROPPER, |_, place| {
        Some(Box::new(ItemChemicalTank(place.clone())) as Box<dyn ResourceHandler<Chemical>>)
    });
    // Slice 5: the Chemical Injection Chamber. The same machine shape as the
    // three above — a container, an insert-only buffer, a 200-tick operation —
    // plus a gas tank of its own (upstream's `MAX_GAS`) that answers the chemical
    // capability and is topped up from a neighbour on its tick. A machine is an
    // insert-only chemical handler upstream (`ConstantPredicates.notExternal()`),
    // so a foreign extract at it takes nothing.
    //
    // Slice 6: the Purification Chamber is that machine again — the same
    // upstream class and recipe type — on oxygen, with its own constants and
    // its own table. One part declaration serves both.
    let gas_machines = [
        GasDecl {
            block: INJECTION,
            kind: INJECTION_TYPE,
            operation: INJECTION_OPERATION,
            capacity: INJECTION_CAPACITY,
            tank: INJECTION_TANK,
            recipe_for: injecting::recipe_for,
        },
        GasDecl {
            block: PURIFICATION,
            kind: PURIFICATION_TYPE,
            operation: PURIFY_OPERATION,
            capacity: PURIFY_CAPACITY,
            tank: PURIFY_TANK,
            recipe_for: purifying::recipe_for,
        },
    ];
    for machine in gas_machines {
        let key = registrar.block_key(machine.block);
        registrar.block_container(&key, container());
        registrar.block_energy(&key, machine_battery(machine.capacity));
        let kind = registrar
            .block_entity_type(machine.kind)
            .blocks(&[machine.kind])
            .build();
        let tank = machine.tank;
        registrar.block_entity_capability(&CHEMICAL, &kind, move |_, at| {
            Some(Box::new(ChemicalTank {
                side: at,
                capacity: tank,
            }) as Box<dyn ResourceHandler<Chemical>>)
        });
        let part = GasPart {
            operation: machine.operation,
            capacity: machine.capacity,
            tank: machine.tank,
            recipe_for: machine.recipe_for,
        };
        if let Err(error) = registrar.block_entity_part(&kind, part) {
            context.refuse(&error.to_string());
        }
    }
    // Slice 7: the basic factories. The same operations as the five machines
    // above, run side by side — one input and one output slot per process over
    // one shared buffer — and, for the two gas factories, over one shared tank
    // of `MAX_GAS` per process. Upstream draws its own factory GUI, which this
    // port does not have, so the container declares no menu; automation and
    // `/mekanism info` reach the six slots.
    let factories = [
        FactoryDecl {
            block: SMELTING_FACTORY,
            kind: SMELTING_FACTORY_TYPE,
            operation: SMELTER_OPERATION,
            capacity: SMELTING_FACTORY_CAPACITY,
            lookup: smelting::lookup,
        },
        FactoryDecl {
            block: ENRICHING_FACTORY,
            kind: ENRICHING_FACTORY_TYPE,
            operation: CHAMBER_OPERATION,
            capacity: ENRICHING_FACTORY_CAPACITY,
            lookup: enriching::lookup,
        },
        FactoryDecl {
            block: CRUSHING_FACTORY,
            kind: CRUSHING_FACTORY_TYPE,
            operation: CRUSHER_OPERATION,
            capacity: CRUSHING_FACTORY_CAPACITY,
            lookup: crushing::lookup,
        },
    ];
    for factory in factories {
        let key = registrar.block_key(factory.block);
        registrar.block_container(&key, factory_container());
        registrar.block_energy(&key, machine_battery(factory.capacity));
        let kind = registrar
            .block_entity_type(factory.kind)
            .blocks(&[factory.kind])
            .build();
        let part = FactoryPart {
            operation: factory.operation,
            capacity: factory.capacity,
            lookup: factory.lookup,
        };
        if let Err(error) = registrar.block_entity_part(&kind, part) {
            context.refuse(&error.to_string());
        }
    }
    let gas_factories = [
        GasFactoryDecl {
            block: INJECTING_FACTORY,
            kind: INJECTING_FACTORY_TYPE,
            operation: INJECTION_OPERATION,
            capacity: INJECTING_FACTORY_CAPACITY,
            tank: INJECTING_FACTORY_TANK,
            recipe_for: injecting::recipe_for,
        },
        GasFactoryDecl {
            block: PURIFYING_FACTORY,
            kind: PURIFYING_FACTORY_TYPE,
            operation: PURIFY_OPERATION,
            capacity: PURIFYING_FACTORY_CAPACITY,
            tank: PURIFYING_FACTORY_TANK,
            recipe_for: purifying::recipe_for,
        },
    ];
    for factory in gas_factories {
        let key = registrar.block_key(factory.block);
        registrar.block_container(&key, factory_container());
        registrar.block_energy(&key, machine_battery(factory.capacity));
        let kind = registrar
            .block_entity_type(factory.kind)
            .blocks(&[factory.kind])
            .build();
        let tank = factory.tank;
        registrar.block_entity_capability(&CHEMICAL, &kind, move |_, at| {
            Some(Box::new(ChemicalTank {
                side: at,
                capacity: tank,
            }) as Box<dyn ResourceHandler<Chemical>>)
        });
        let part = GasFactoryPart {
            operation: factory.operation,
            capacity: factory.capacity,
            tank: factory.tank,
            recipe_for: factory.recipe_for,
        };
        if let Err(error) = registrar.block_entity_part(&kind, part) {
            context.refuse(&error.to_string());
        }
    }
    // Slice 8: the Chemical Washer — the same machine shape with a **fluid**
    // input beside its chemicals. It has no container: its inputs are tanks
    // (upstream's four slots are container conveniences, recorded as a
    // deviation), an 80,000 FE buffer, one 10,000 mB water tank declared through
    // `block_fluids` (the game's own `horizon:fluids`, whose contents live in
    // the entity's saved `fluids` list), and two 10,000-unit slurry tanks
    // answered through `mekanism:chemical`: the dirty one it takes and the clean
    // one it makes.
    let key = registrar.block_key(WASHER);
    registrar.block_energy(&key, machine_battery(WASHER_CAPACITY));
    registrar.block_fluids(&key, washer_fluids());
    let kind = registrar
        .block_entity_type(WASHER_TYPE)
        .blocks(&[WASHER_TYPE])
        .build();
    registrar.block_entity_capability(&CHEMICAL, &kind, |_, at| {
        Some(Box::new(WasherTanks {
            side: at,
            capacity: WASHER_SLURRY,
        }) as Box<dyn ResourceHandler<Chemical>>)
    });
    if let Err(error) = registrar.block_entity_part(&kind, WasherPart) {
        context.refuse(&error.to_string());
    }
    // The fluid key the machine's tank takes, resolved once at registration: a
    // `FluidKey` is a load-owned slot handle, and `/mekanism fluid insert` builds
    // its `FluidStack` from it (the mod API offers no other way to name a fluid
    // outside the registrar).
    if WATER_KEY.set(registrar.fluid_key(WATER)).is_err() {
        context.refuse("mekanism: the water fluid key was resolved twice");
    }
}

/// The one fluid the washer's tank takes, kept as the registration-time key the
/// fluid capability's calls are built from.
static WATER_KEY: std::sync::OnceLock<FluidKey> = std::sync::OnceLock::new();

/// The washer's fluid tanks, as `block_fluids` declares them: one 10,000 mB tank
/// of water that a fill may reach from every face and a drain may not take —
/// upstream's `BasicFluidTank.input(MAX_FLUID, fluid -> containsRecipeAB(...))`.
pub(crate) fn washer_fluids() -> ModFluidTanks {
    // `ModFluidTanks` is `#[non_exhaustive]`: its fields are public but a literal
    // cannot be built outside the SDK's crate, so it is filled one field at a time.
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![i32::try_from(WASHER_WATER).unwrap_or(i32::MAX)];
    tanks.faces = std::array::from_fn(|_| vec![0]);
    tanks.inputs = vec![0];
    tanks.outputs = Vec::new();
    tanks
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

/// A factory's container: one input and one output slot per process, every
/// face reaching all six, and no vanilla menu (the six-slot factory GUI is
/// upstream's, which this port does not draw).
pub(crate) fn factory_container() -> ModContainer {
    let all = (0..FACTORY_SLOTS).collect::<Vec<_>>();
    ModContainer {
        slots: FACTORY_SLOTS,
        faces: std::array::from_fn(|_| all.clone()),
        inputs: FACTORY_INPUT_SLOTS.to_vec(),
        outputs: FACTORY_OUTPUT_SLOTS.to_vec(),
        menu: None,
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
        INJECTION => Some(INJECTION_CAPACITY),
        PURIFICATION => Some(PURIFY_CAPACITY),
        SMELTING_FACTORY => Some(SMELTING_FACTORY_CAPACITY),
        ENRICHING_FACTORY => Some(ENRICHING_FACTORY_CAPACITY),
        CRUSHING_FACTORY => Some(CRUSHING_FACTORY_CAPACITY),
        INJECTING_FACTORY => Some(INJECTING_FACTORY_CAPACITY),
        PURIFYING_FACTORY => Some(PURIFYING_FACTORY_CAPACITY),
        WASHER => Some(WASHER_CAPACITY),
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
        INJECTION => Some(INJECTION_TICKS),
        PURIFICATION => Some(PURIFY_TICKS),
        SMELTING_FACTORY | ENRICHING_FACTORY | CRUSHING_FACTORY | INJECTING_FACTORY
        | PURIFYING_FACTORY => Some(FACTORY_TICKS),
        WASHER => Some(WASHER_TICKS),
        _ => None,
    }
}

/// Whether the Chemical Washer stands at `id`: its `/mekanism info` line is its
/// three tanks', not a slot machine's.
pub(crate) fn is_washer(id: &str) -> bool {
    id == WASHER
}

/// Whether one of the five basic factories stands at `id`.
pub(crate) fn is_factory(id: &str) -> bool {
    matches!(
        id,
        SMELTING_FACTORY
            | ENRICHING_FACTORY
            | CRUSHING_FACTORY
            | INJECTING_FACTORY
            | PURIFYING_FACTORY
    )
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

/// A factory's saved progress: the `progress` int array upstream writes, one
/// entry per process, zeros when it is absent or of another length (a factory
/// saved before this slice, or by another tier).
pub(crate) fn read_factory_progress(saved: &Tag, processes: usize) -> Vec<i32> {
    match saved.get("progress") {
        Some(Tag::IntArray(values)) if values.len() == processes => {
            values.iter().map(|value| (*value).max(0)).collect()
        }
        _ => vec![0; processes],
    }
}

/// Write a factory's tick result: the shared buffer's amount and every
/// process's progress, as the `progress` int array upstream saves.
pub(crate) fn write_factory_state(saved: &mut Tag, energy: i32, processes: &[FactoryProcess]) {
    write_energy(saved, energy);
    if let Tag::Compound(nbt) = saved {
        let progress = processes.iter().map(|process| process.progress).collect();
        nbt.insert("progress".into(), Tag::IntArray(progress));
    }
}

/// A factory's processes as the pure core reads them: one input and one output
/// slot per process, and the saved progress.
fn factory_processes(
    host: &dyn Host,
    items: &[Option<ItemStack>],
    progress: &[i32],
) -> Vec<FactoryProcess> {
    (0..FACTORY_PROCESSES)
        .map(|index| FactoryProcess {
            progress: progress.get(index).copied().unwrap_or(0),
            input: slot_stack(
                host,
                items
                    .get(FACTORY_INPUT_SLOTS[index])
                    .and_then(Option::as_ref),
            ),
            output: slot_stack(
                host,
                items
                    .get(FACTORY_OUTPUT_SLOTS[index])
                    .and_then(Option::as_ref),
            ),
        })
        .collect()
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

/// The name a stack reports in `/mekanism info`: its item id, or `empty`.
fn stack_name(stack: &ItemAmount) -> String {
    if stack.is_empty() {
        "empty".to_owned()
    } else {
        stack.item.clone()
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
    format!(
        "{id}: {}/{} FE, progress {}/{}, input {} x{}, output {} x{}",
        machine.energy,
        capacity,
        machine.progress,
        ticks,
        stack_name(input),
        input.count,
        stack_name(output),
        output.count
    )
}

/// The sum of a factory's process slots: empty when they all are, the common
/// item with the summed count when they agree, and `mixed` otherwise — one line
/// for `/mekanism info`, exactly as a single machine answers.
pub(crate) fn aggregate_stacks(stacks: &[ItemAmount]) -> ItemAmount {
    let Some(first) = stacks.iter().find(|stack| !stack.is_empty()) else {
        return ItemAmount::default();
    };
    let count = stacks.iter().map(|stack| stack.count).sum();
    if stacks
        .iter()
        .all(|stack| stack.is_empty() || stack.item == first.item)
    {
        ItemAmount::new(&first.item, count)
    } else {
        ItemAmount::new("mixed", count)
    }
}

/// The `/mekanism info` report line for one factory: its shared buffer, the
/// furthest-along process, and the sum of its three input and output slots.
pub(crate) fn factory_info_text(
    id: &str,
    capacity: i32,
    energy: i32,
    processes: &[FactoryProcess],
) -> String {
    let progress = processes
        .iter()
        .map(|process| process.progress)
        .max()
        .unwrap_or(0);
    let ticks = machine_ticks(id).unwrap_or(0);
    let inputs: Vec<ItemAmount> = processes
        .iter()
        .map(|process| process.input.clone())
        .collect();
    let outputs: Vec<ItemAmount> = processes
        .iter()
        .map(|process| process.output.clone())
        .collect();
    let input = aggregate_stacks(&inputs);
    let output = aggregate_stacks(&outputs);
    format!(
        "{id}: {energy}/{capacity} FE ({} processes), progress {progress}/{ticks}, \
         input {} x{}, output {} x{}",
        processes.len(),
        stack_name(&input),
        input.count,
        stack_name(&output),
        output.count
    )
}

/// The `/mekanism info` report line for the Chemical Washer: its buffer, its
/// progress and its three tanks, each named by what it holds (`empty` at 0).
pub(crate) fn washer_info_text(
    id: &str,
    capacity: i32,
    machine: Machine,
    water: &FluidTank,
    input: &Tank,
    output: &Tank,
) -> String {
    format!(
        "{id}: {}/{} FE, progress {}/{}, water {}/{} mB, input {} x{}, output {} x{}",
        machine.energy,
        capacity,
        machine.progress,
        WASHER_TICKS,
        water.amount,
        WASHER_WATER,
        tank_name(input),
        input.amount,
        tank_name(output),
        output.amount
    )
}

/// A tank's chemical for a readout, `empty` when it holds none.
fn tank_name(tank: &Tank) -> String {
    tank.held()
        .map_or_else(|| "empty".to_owned(), |(id, _)| id.to_owned())
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

/// A gas machine's behaviour — the Chemical Injection Chamber and the
/// Purification Chamber: the shared machine tick plus a gas input, the machine's
/// own tank topped up from a neighbour through the chemical capability, and one
/// tick's gas spent per operating tick.
///
/// This is the **consumer** half of slice 4's capability: the tank beside it is
/// asked through [`CHEMICAL`] by name (never by this port's block ids), exactly
/// as another mod's crate would ask, and what the handler gives is what the
/// machine spends. Slice 6 is the same machine on oxygen, so the two carry one
/// Part and differ only in the values [`GasDecl`] gives it.
pub(crate) struct GasPart {
    operation: Operation,
    capacity: i32,
    tank: u32,
    recipe_for: fn(&str) -> Option<GasRecipe>,
}

#[mod_api::horizon_part]
impl BlockPart for GasPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for GasPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: top the machine's own tank up from a neighbour when this
    /// tick could work, then run the operation. A pull that moved anything is
    /// always written back (an idle tick never answers `Unchanged` after drawing
    /// gas), and an idle tick that drew nothing is `Unchanged`, as the other
    /// machines are.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let resized = event.entity.items.len() != MENU_SLOTS;
        if resized {
            event.entity.items.resize(MENU_SLOTS, None);
        }
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut machine = read_machine(&saved, self.capacity);
        machine.progress = machine.progress.clamp(0, (self.operation.ticks - 1).max(0));
        let mut tank = read_tank(&saved);
        let mut input = slot_stack(&*host, event.entity.items[INPUT_SLOT].as_ref());
        let mut output = slot_stack(&*host, event.entity.items[OUTPUT_SLOT].as_ref());
        let limit = stack_limit(&*host, &output);
        let pulled = {
            let mut tick = GasTick {
                machine,
                tank: &mut tank,
                input: &input,
                output: &output,
                limit,
            };
            top_up_tank(self, host, event.pos, &mut tick)
        };
        let outcome = process_gas_tick(
            &mut machine,
            self.operation,
            &mut tank,
            &mut input,
            &mut output,
            limit,
            &self.recipe_for,
        );
        write_state(&mut saved, machine.energy, machine.progress);
        write_tank(&mut saved, &tank);
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
            Outcome::Idle(
                IdleReason::NoEnergy | IdleReason::NoOutputSpace | IdleReason::NoChemical
            )
        );
        if idle && !resized && pulled == 0 {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// A basic item factory's behaviour — the Basic Smelting, Enriching and
/// Crushing factories: three processes of the shared operation, each with its
/// own slots and progress, over one shared energy buffer.
pub(crate) struct FactoryPart {
    operation: Operation,
    capacity: i32,
    lookup: fn(&str) -> Option<MachineRecipe>,
}

#[mod_api::horizon_part]
impl BlockPart for FactoryPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for FactoryPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick of all three processes. Every working process spends its
    /// own tick's energy from the one buffer, so a factory working flat out
    /// spends three times what a single machine would. An idle tick where every
    /// process is out of energy or out of output room changes nothing and
    /// answers `Unchanged`, as the single machines do.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let resized = event.entity.items.len() != FACTORY_SLOTS;
        if resized {
            event.entity.items.resize(FACTORY_SLOTS, None);
        }
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut energy = read_energy(&saved, self.capacity);
        let saved_progress = read_factory_progress(&saved, FACTORY_PROCESSES);
        let mut processes = factory_processes(&*host, &event.entity.items, &saved_progress);
        let ceiling = (self.operation.ticks - 1).max(0);
        for process in &mut processes {
            process.progress = process.progress.clamp(0, ceiling);
        }
        let limits: Vec<u32> = processes
            .iter()
            .map(|process| stack_limit(&*host, &process.output))
            .collect();
        let operations = operations_per_tick(self.operation.ticks * 10, 1);
        let outcomes = process_factory_tick(
            &mut energy,
            &mut processes,
            self.operation,
            &limits,
            operations,
            &self.lookup,
        );
        write_factory_state(&mut saved, energy, &processes);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        for (index, process) in processes.iter().enumerate() {
            set_slot(
                &*host,
                &mut event.entity.items,
                FACTORY_INPUT_SLOTS[index],
                &process.input,
            );
            set_slot(
                &*host,
                &mut event.entity.items,
                FACTORY_OUTPUT_SLOTS[index],
                &process.output,
            );
        }
        let idle = outcomes.iter().all(|outcome| {
            matches!(
                outcome,
                Outcome::Idle(IdleReason::NoEnergy | IdleReason::NoOutputSpace)
            )
        });
        if idle && !resized {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// A basic gas factory's behaviour — the Basic Injecting and Basic Purifying
/// factories: the item factory's three processes over one shared tank, which is
/// topped up from an adjacent chemical handler for every process that can work
/// this tick.
pub(crate) struct GasFactoryPart {
    operation: Operation,
    capacity: i32,
    tank: u32,
    recipe_for: fn(&str) -> Option<GasRecipe>,
}

#[mod_api::horizon_part]
impl BlockPart for GasFactoryPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for GasFactoryPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: draw the gas every working process needs (one tick's
    /// worth each) through the chemical capability, then run all three. The
    /// factory holds none of the gas after a working tick: the tank is a buffer,
    /// as the single machines' 210-unit tanks are.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let resized = event.entity.items.len() != FACTORY_SLOTS;
        if resized {
            event.entity.items.resize(FACTORY_SLOTS, None);
        }
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut energy = read_energy(&saved, self.capacity);
        let mut tank = read_tank(&saved);
        let saved_progress = read_factory_progress(&saved, FACTORY_PROCESSES);
        let mut processes = factory_processes(&*host, &event.entity.items, &saved_progress);
        let ceiling = (self.operation.ticks - 1).max(0);
        for process in &mut processes {
            process.progress = process.progress.clamp(0, ceiling);
        }
        let limits: Vec<u32> = processes
            .iter()
            .map(|process| stack_limit(&*host, &process.output))
            .collect();
        let pulled = match factory_gas_want(
            energy,
            self.operation,
            &tank,
            &processes,
            &limits,
            &self.recipe_for,
        ) {
            Some((chemical, want)) => {
                let room = self.tank.saturating_sub(tank.amount);
                let want = want.min(room);
                if want == 0 {
                    0
                } else {
                    let pulled = pull_chemical(host, event.pos, chemical, want);
                    let _ = insert_chemical(&mut tank, chemical, pulled, self.tank);
                    pulled
                }
            }
            None => 0,
        };
        let operations = operations_per_tick(self.operation.ticks * 10, 1);
        let outcomes = process_gas_factory_tick(
            &mut energy,
            &mut tank,
            &mut processes,
            self.operation,
            &limits,
            operations,
            &self.recipe_for,
        );
        write_factory_state(&mut saved, energy, &processes);
        write_tank(&mut saved, &tank);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        for (index, process) in processes.iter().enumerate() {
            set_slot(
                &*host,
                &mut event.entity.items,
                FACTORY_INPUT_SLOTS[index],
                &process.input,
            );
            set_slot(
                &*host,
                &mut event.entity.items,
                FACTORY_OUTPUT_SLOTS[index],
                &process.output,
            );
        }
        let idle = outcomes.iter().all(|outcome| {
            matches!(
                outcome,
                Outcome::Idle(
                    IdleReason::NoEnergy | IdleReason::NoOutputSpace | IdleReason::NoChemical
                )
            )
        });
        if idle && !resized && pulled == 0 {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// The Chemical Washer's behaviour: one operation at a time over its three
/// tanks — a fluid input (water), a chemical input (dirty slurry) and a chemical
/// output (clean slurry) — and its own 80,000 FE buffer.
///
/// It has no container and no menu: upstream's four slots are container
/// conveniences (a bucket in, a dropper out), recorded as a deviation, so the
/// tanks are the interface and `/mekanism info` reports them.
pub(crate) struct WasherPart;

#[mod_api::horizon_part]
impl BlockPart for WasherPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for WasherPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: draw the operation's dirty slurry from a neighbour when
    /// the machine could work (through `mekanism:chemical`, the gas machines'
    /// recorded deviation: no pipes, so a machine takes what it is about to
    /// spend), then run one operation's tick over the three tanks and the buffer.
    /// An idle tick that only held its progress (no energy or no output room)
    /// changes nothing and answers `Unchanged`; a reset (no water, no recipe) is a
    /// change, so it is written back.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut machine = read_machine(&saved, WASHER_CAPACITY);
        machine.progress = machine.progress.clamp(0, (WASHER_TICKS - 1).max(0));
        let mut water = read_washer_fluid(&saved);
        let (mut input, mut output) = read_washer_tanks(&saved);
        let pulled = washer_draw(&machine, &water, &mut input, &output, host, event.pos);
        let outcome = process_wash_tick(
            &mut machine,
            WASHER_OPERATION,
            &mut water,
            &mut input,
            &mut output,
            WASHER_SLURRY,
            &washing::recipe_for,
        );
        write_state(&mut saved, machine.energy, machine.progress);
        write_washer_fluid(&mut saved, &water);
        write_washer_tanks(&mut saved, &input, &output);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        let idle = matches!(
            outcome,
            Outcome::Idle(IdleReason::NoEnergy | IdleReason::NoOutputSpace)
        );
        if idle && pulled == 0 {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// Draw the operation's dirty slurry from an adjacent chemical handler through
/// [`CHEMICAL`], and answer how much the neighbour gave.
///
/// It draws only when the machine could actually complete an operation — water
/// for it, room for the clean slurry and energy for a tick — so a stalled washer
/// never siphons a neighbour it cannot use (the defect the gas factories found in
/// their own first version, pinned here from the start).
///
/// Two shapes, because a washer's recipe is chosen by its **chemical** while a gas
/// machine's is chosen by its input **item**: a tank already holding a slurry the
/// jar has a row for is topped up with that slurry, and an empty tank asks for a
/// dirty slurry any neighbour holds, in the table's order (the port ships no
/// pipes, so a machine takes what it is about to spend — the gas machines'
/// recorded deviation).
fn washer_draw(
    machine: &Machine,
    water: &FluidTank,
    input: &mut Tank,
    output: &Tank,
    host: &mut dyn ServerHost,
    pos: [i32; 3],
) -> u32 {
    if machine.energy < WASHER_OPERATION.usage {
        return 0;
    }
    let water_ready = water.held().is_some_and(|(fluid, amount)| {
        fluid == WATER
            && amount
                >= washing::recipe_for("mekanism:dirty_iron")
                    .map_or(1, |recipe| recipe.fluid_amount)
    });
    if !water_ready {
        return 0;
    }
    if let Some((dirty, _)) = input.held() {
        let Some(recipe) = washing::recipe_for(dirty) else {
            return 0;
        };
        if room_in_tank(output, recipe.output, recipe.output_amount, WASHER_SLURRY)
            < recipe.output_amount
        {
            return 0;
        }
        let Some((chemical, want)) = washer_pull(&recipe, input) else {
            return 0;
        };
        let pulled = pull_chemical(host, pos, chemical, want);
        let _ = insert_chemical(input, chemical, pulled, WASHER_SLURRY);
        return pulled;
    }
    // An empty tank: a recipe's output must have room before any is drawn, and
    // every washing row makes one unit of its own clean slurry.
    for dirty in SLURRIES {
        let Some(recipe) = washing::recipe_for(dirty) else {
            continue;
        };
        if room_in_tank(output, recipe.output, recipe.output_amount, WASHER_SLURRY)
            < recipe.output_amount
        {
            return 0;
        }
        let pulled = pull_chemical(host, pos, dirty, recipe.chemical_amount);
        if pulled > 0 {
            let _ = insert_chemical(input, dirty, pulled, WASHER_SLURRY);
            return pulled;
        }
    }
    0
}

/// The state one gas-machine tick works on, bundled so the top-up and the
/// operation read the same view of it.
struct GasTick<'a> {
    machine: Machine,
    tank: &'a mut Tank,
    input: &'a ItemAmount,
    output: &'a ItemAmount,
    limit: u32,
}

/// Top the machine's own tank up for the tick it is about to run: the recipe's
/// gas, one tick's worth, taken from an adjacent chemical handler through the
/// capability. How much the neighbour gave.
///
/// Nothing is drawn when the machine could not work this tick (no energy, no
/// output room, no recipe, a tank of another chemical), so a stalled machine
/// does not siphon a neighbour it cannot use.
fn top_up_tank(
    part: &GasPart,
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    tick: &mut GasTick<'_>,
) -> u32 {
    if tick.input.is_empty() || tick.machine.energy < part.operation.usage {
        return 0;
    }
    let Some(recipe) = (part.recipe_for)(&tick.input.item) else {
        return 0;
    };
    if room_for(tick.output, &recipe.output, tick.limit) < recipe.output.count {
        return 0;
    }
    let Some((chemical, want)) = gas_pull(&recipe, tick.tank) else {
        return 0;
    };
    let room = part.tank.saturating_sub(tick.tank.amount);
    let want = want.min(room);
    if want == 0 {
        return 0;
    }
    let pulled = pull_chemical(host, pos, chemical, want);
    let _ = insert_chemical(tick.tank, chemical, pulled, part.tank);
    pulled
}

/// Take up to `want` of `chemical` out of the chemical handlers beside `pos`,
/// through [`CHEMICAL`]: what the neighbours gave.
///
/// Each move is measured, as the cable's energy moves are: the handler's own
/// `extract` answers what really left, and only a handler that holds the asked
/// chemical is asked at all (a tank of oxygen beside a machine that needs
/// hydrogen chloride gives nothing).
fn pull_chemical(host: &mut dyn ServerHost, pos: [i32; 3], chemical: &str, want: u32) -> u32 {
    let Some(chemical) = Chemical::new(chemical) else {
        return 0;
    };
    let Some(level) = host.level(host.dimension()) else {
        return 0;
    };
    let mut neighbours: Vec<Capability<dyn ResourceHandler<Chemical>>> = Vec::with_capacity(6);
    for face in Direction::ALL {
        let [dx, dy, dz] = face.normal();
        let at = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
        if let Some(handler) = level.capability(host, &CHEMICAL, at, Some(face.opposite())) {
            neighbours.push(handler);
        }
    }
    if neighbours.is_empty() {
        return 0;
    }
    let after = host.transaction(move |host, tx| {
        let mut got = 0u32;
        for handler in &neighbours {
            if got >= want {
                break;
            }
            if handler.resource(host, 0).as_ref() != Some(&chemical) {
                continue;
            }
            let came = handler.extract(host, tx, 0, &chemical, want - got);
            got = got.saturating_add(came);
        }
        Ok::<u32, ()>(got)
    });
    after.unwrap_or(0)
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

// ---------------------------------------------------------------------------
// Slice 4: the chemical capability's two handlers and its readouts
// ---------------------------------------------------------------------------

/// A tank as the saved compound holds it: `chemical` and `amount`, the empty
/// chemical at 0 for an empty tank (the shape `ChemicalStack`'s codec writes for
/// an empty stack).
pub(crate) fn read_tank(saved: &Tag) -> Tank {
    let chemical = match saved.get("chemical") {
        Some(Tag::String(id)) => Some(id.to_string_lossy()),
        _ => None,
    };
    let amount = u32::try_from(int(saved, "amount").max(0)).unwrap_or(0);
    match chemical {
        Some(id) if amount > 0 && id != EMPTY_CHEMICAL => Tank {
            chemical: Some(id),
            amount,
        },
        _ => Tank::default(),
    }
}

/// Write a tank into a saved compound: both keys, so an emptied tank names the
/// empty chemical at 0 rather than leaving a stale id behind.
pub(crate) fn write_tank(saved: &mut Tag, tank: &Tank) {
    if let Tag::Compound(nbt) = saved {
        let (chemical, amount) = tank.held().unwrap_or((EMPTY_CHEMICAL, 0));
        nbt.insert("chemical".into(), Tag::String(chemical.into()));
        nbt.insert(
            "amount".into(),
            Tag::Int(i32::try_from(amount).unwrap_or(i32::MAX)),
        );
    }
}

/// The saved-data patch that writes `tank`, as [`write_tank`] would.
fn tank_patch(tank: &Tank) -> Compound {
    let mut saved = Tag::Compound(Compound::new());
    write_tank(&mut saved, tank);
    match saved {
        Tag::Compound(nbt) => nbt,
        _ => Compound::new(),
    }
}

/// One of the washer's two slurry tanks, from its `input_*`/`output_*` keys.
fn read_prefixed_tank(saved: &Tag, prefix: &str) -> Tank {
    let chemical = match saved.get(&format!("{prefix}_chemical")) {
        Some(Tag::String(id)) => Some(id.to_string_lossy()),
        _ => None,
    };
    let amount = u32::try_from(int(saved, &format!("{prefix}_amount")).max(0)).unwrap_or(0);
    match chemical {
        Some(id) if amount > 0 && id != EMPTY_CHEMICAL => Tank {
            chemical: Some(id),
            amount,
        },
        _ => Tank::default(),
    }
}

/// Write one of the washer's slurry tanks, as [`read_prefixed_tank`] reads it.
fn write_prefixed_tank(saved: &mut Tag, prefix: &str, tank: &Tank) {
    if let Tag::Compound(nbt) = saved {
        let (chemical, amount) = tank.held().unwrap_or((EMPTY_CHEMICAL, 0));
        let id_key = format!("{prefix}_chemical");
        let amount_key = format!("{prefix}_amount");
        nbt.insert(id_key.as_str().into(), Tag::String(chemical.into()));
        nbt.insert(
            amount_key.as_str().into(),
            Tag::Int(i32::try_from(amount).unwrap_or(i32::MAX)),
        );
    }
}

/// The washer's two slurry tanks as its saved compound holds them: the dirty
/// one it takes, then the clean one it makes.
pub(crate) fn read_washer_tanks(saved: &Tag) -> (Tank, Tank) {
    (
        read_prefixed_tank(saved, "input"),
        read_prefixed_tank(saved, "output"),
    )
}

/// Write the washer's two slurry tanks into its saved compound.
pub(crate) fn write_washer_tanks(saved: &mut Tag, input: &Tank, output: &Tank) {
    write_prefixed_tank(saved, "input", input);
    write_prefixed_tank(saved, "output", output);
}

/// The washer's water tank, as the game's own `fluids` list holds it: the first
/// tank's `Fluid` and `Amount` (millibuckets), the shape the fluid capability
/// writes and `/data get block <pos> fluids` reads.
pub(crate) fn read_washer_fluid(saved: &Tag) -> FluidTank {
    let Some(Tag::List(list)) = saved.get("fluids") else {
        return FluidTank::default();
    };
    let Some(tank @ Tag::Compound(_)) = list.first() else {
        return FluidTank::default();
    };
    let fluid = match tank.get("Fluid") {
        Some(Tag::String(id)) if id.to_string_lossy() != "minecraft:empty" => {
            Some(id.to_string_lossy())
        }
        _ => None,
    };
    let amount = match tank.get("Amount") {
        Some(Tag::Int(amount)) => u32::try_from((*amount).max(0)).unwrap_or(0),
        _ => 0,
    };
    match fluid {
        Some(fluid) if amount > 0 => FluidTank {
            fluid: Some(fluid),
            amount,
        },
        _ => FluidTank::default(),
    }
}

/// Write the washer's water tank into the `fluids` list the fluid capability
/// reads: an emptied tank names `minecraft:empty` at 0 so the tank stays in
/// place, as the game's own writer does.
pub(crate) fn write_washer_fluid(saved: &mut Tag, tank: &FluidTank) {
    if let Tag::Compound(nbt) = saved {
        let (fluid, amount) = tank.held().unwrap_or(("minecraft:empty", 0));
        let mut entry = Compound::new();
        entry.insert("Fluid".into(), Tag::String(fluid.into()));
        entry.insert(
            "Amount".into(),
            Tag::Int(i32::try_from(amount).unwrap_or(i32::MAX)),
        );
        nbt.insert("fluids".into(), Tag::List(vec![Tag::Compound(entry)]));
    }
}

/// The chemical capability for the washer: its **two** slurry tanks, tank 0 the
/// dirty one it takes (insert only, upstream's `inputModern`) and tank 1 the
/// clean one it makes (extract only, upstream's `output`), both kept in the
/// entity's saved compound (`input_*`/`output_*`).
///
/// Each call reads the entity and writes it back through `merge_data`, as the
/// tank block's handler does, so a call from a command, from automation or from
/// another mod sees the saved data and the chunk is marked changed.
pub(crate) struct WasherTanks {
    side: BlockSide,
    capacity: u32,
}

impl WasherTanks {
    /// What the block entity here holds; two empty tanks for a stale one.
    fn held(&self, host: &mut dyn ServerHost) -> (Tank, Tank) {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return (Tank::default(), Tank::default());
        };
        let Some(saved) = entity.data(&*host) else {
            return (Tank::default(), Tank::default());
        };
        read_washer_tanks(&Tag::Compound(saved))
    }

    /// Write both tanks back into the block entity here: whether it loaded.
    fn store(&self, host: &mut dyn ServerHost, input: &Tank, output: &Tank) -> bool {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return false;
        };
        let mut saved = Tag::Compound(Compound::new());
        write_washer_tanks(&mut saved, input, output);
        let Tag::Compound(patch) = saved else {
            return false;
        };
        entity
            .merge_data(host, patch)
            .ok()
            .is_some_and(|loaded| loaded)
    }
}

impl ResourceHandler<Chemical> for WasherTanks {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        2
    }

    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        let (input, output) = self.held(host);
        let tank = match index {
            0 => input,
            1 => output,
            _ => return None,
        };
        tank.held().and_then(|(id, _)| Chemical::new(id))
    }

    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        let (input, output) = self.held(host);
        match index {
            0 => input.amount,
            1 => output.amount,
            _ => 0,
        }
    }

    fn capacity(&self, _: &mut dyn ServerHost, index: usize, _: &Chemical) -> u32 {
        if index < 2 { self.capacity } else { 0 }
    }

    /// A fill goes into the **input** tank only: the clean one is filled by the
    /// machine's own operation, as upstream's `BasicChemicalTank.output` is.
    fn insert(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index != 0 {
            return 0;
        }
        let (mut input, output) = self.held(host);
        match insert_chemical(&mut input, chemical.as_str(), amount, self.capacity) {
            Ok(0) => 0,
            Ok(went) if self.store(host, &input, &output) => went,
            Ok(_) => 0,
            // A chemical Mekanism does not register: the handler cannot name it
            // back, so it answers 0 (`/mekanism chemical insert` names it).
            Err(_) => 0,
        }
    }

    /// A drain takes from the **output** tank only: the dirty one is
    /// automation's to fill, not to empty.
    fn extract(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index != 1 {
            return 0;
        }
        let (input, mut output) = self.held(host);
        let came = extract_chemical(&mut output, chemical.as_str(), amount);
        if came == 0 {
            return 0;
        }
        if self.store(host, &input, &output) {
            came
        } else {
            0
        }
    }
}

/// The chemical capability for one block entity: its one tank, kept in the
/// entity's saved compound (`chemical` + `amount`), as the SDK's `test:chemical`
/// fixture keeps a tank.
///
/// Nothing formal goes in a field: each call reads the entity and writes it back
/// through `merge_data`, so a call from another mod, from a command or from
/// automation all see the saved data, and the chunk is marked changed. The tank's
/// `capacity` is the block's own (64,000 for the Basic Chemical Tank, 210 for the
/// injection chamber's `MAX_GAS` buffer).
pub(crate) struct ChemicalTank {
    side: BlockSide,
    capacity: u32,
}

impl ChemicalTank {
    /// What the block entity here holds; empty for a stale or missing one.
    fn held(&self, host: &mut dyn ServerHost) -> Tank {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return Tank::default();
        };
        let Some(saved) = entity.data(&*host) else {
            return Tank::default();
        };
        read_tank(&Tag::Compound(saved))
    }

    /// Write `tank` into the block entity here: whether it loaded.
    fn store(&self, host: &mut dyn ServerHost, tank: &Tank) -> bool {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return false;
        };
        entity
            .merge_data(host, tank_patch(tank))
            .ok()
            .is_some_and(|loaded| loaded)
    }
}

impl ResourceHandler<Chemical> for ChemicalTank {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        1
    }

    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        let held = self.held(host).held().filter(|_| index == 0)?.0.to_owned();
        Chemical::new(&held)
    }

    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        if index == 0 {
            self.held(host).amount
        } else {
            0
        }
    }

    fn capacity(&self, _: &mut dyn ServerHost, index: usize, _: &Chemical) -> u32 {
        if index == 0 { self.capacity } else { 0 }
    }

    fn insert(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index != 0 {
            return 0;
        }
        let mut tank = self.held(host);
        match insert_chemical(&mut tank, chemical.as_str(), amount, self.capacity) {
            Ok(0) => 0,
            Ok(went) if self.store(host, &tank) => went,
            Ok(_) => 0,
            // A chemical Mekanism does not register: the handler has no way to
            // name it back, so it answers 0 (`/mekanism chemical insert` is what
            // reports the id).
            Err(_) => 0,
        }
    }

    fn extract(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index != 0 {
            return 0;
        }
        let mut tank = self.held(host);
        let came = extract_chemical(&mut tank, chemical.as_str(), amount);
        match came {
            0 => 0,
            came if self.store(host, &tank) => came,
            _ => 0,
        }
    }
}

/// The chemical capability for the item at one place: the Gauge Dropper's one
/// tank, kept in the stack's `mekanism:chemical` component (`{id, amount}`,
/// `ChemicalStack`'s own shape) and rewritten through an exchange of one, so a
/// filled dropper splits off a stack of them as a bucket does.
///
/// Its capacity and per-call transfer are upstream's `GearConfig` defaults
/// (`gaugeDropperCapacity` 16,000, `gaugeDroppedTransferRate` 250).
pub(crate) struct ItemChemicalTank(ItemPlace);

impl ItemChemicalTank {
    /// The component handler for the stack here; `None` for an id the game would
    /// refuse, which cannot happen for the constant this port names.
    fn item(&self) -> Option<ItemComponentHandler> {
        Some(ItemComponentHandler::new(
            self.0.clone(),
            mod_api::ComponentId::new(ITEM_COMPONENT)?,
        ))
    }

    fn held(&self, host: &mut dyn ServerHost) -> Tank {
        let Some(item) = self.item() else {
            return Tank::default();
        };
        let Some(Tag::Compound(content)) = item.read(host) else {
            return Tank::default();
        };
        let chemical = match content.get(&"id".into()) {
            Some(Tag::String(id)) => Some(id.to_string_lossy()),
            _ => None,
        };
        let amount = match content.get(&"amount".into()) {
            Some(&Tag::Int(amount)) => u32::try_from(amount).unwrap_or(0),
            _ => 0,
        };
        match chemical {
            Some(id) if amount > 0 && id != EMPTY_CHEMICAL => Tank {
                chemical: Some(id),
                amount,
            },
            _ => Tank::default(),
        }
    }

    fn store(&self, host: &mut dyn ServerHost, tx: &Transaction, tank: &Tank) -> bool {
        let Some(item) = self.item() else {
            return false;
        };
        let value = match tank.held() {
            None => None,
            Some((chemical, amount)) => {
                let mut content = Compound::new();
                content.insert("id".into(), Tag::String(chemical.into()));
                content.insert(
                    "amount".into(),
                    Tag::Int(i32::try_from(amount).unwrap_or(i32::MAX)),
                );
                Some(Tag::Compound(content))
            }
        };
        item.write(host, tx, value)
    }
}

impl ResourceHandler<Chemical> for ItemChemicalTank {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        1
    }

    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        let held = self.held(host).held().filter(|_| index == 0)?.0.to_owned();
        Chemical::new(&held)
    }

    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        if index == 0 {
            self.held(host).amount
        } else {
            0
        }
    }

    fn capacity(&self, _: &mut dyn ServerHost, index: usize, _: &Chemical) -> u32 {
        if index == 0 { DROPPER_CAPACITY } else { 0 }
    }

    fn insert(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index != 0 {
            return 0;
        }
        let mut tank = self.held(host);
        match insert_chemical(
            &mut tank,
            chemical.as_str(),
            dropper_move(amount),
            DROPPER_CAPACITY,
        ) {
            Ok(0) => 0,
            Ok(went) if self.store(host, tx, &tank) => went,
            Ok(_) => 0,
            Err(_) => 0,
        }
    }

    fn extract(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index != 0 {
            return 0;
        }
        let mut tank = self.held(host);
        let came = extract_chemical(&mut tank, chemical.as_str(), dropper_move(amount));
        match came {
            0 => 0,
            came if self.store(host, tx, &tank) => came,
            _ => 0,
        }
    }
}

/// What a chemical handler at `pos` answers: the tank's own word, through the
/// capability (not through this port's block ids), and who answered it.
pub(crate) fn chemical_readout(host: &mut dyn ServerHost, pos: [i32; 3]) -> String {
    let Some(level) = host.level(host.dimension()) else {
        return format!("chemical: no level for {pos:?}");
    };
    let block = level
        .block(&*host, pos)
        .map_or_else(|| "nothing".to_owned(), |id| id.id(&*host));
    let Some(handler) = level.capability(host, &CHEMICAL, pos, None) else {
        return format!(
            "chemical: no chemical handler at {} {} {}",
            pos[0], pos[1], pos[2]
        );
    };
    let owner = handler.owner();
    let held = handler.resource(host, 0);
    let amount = handler.amount(host, 0);
    // The capacity the handler itself answers, asked with a probe chemical so an
    // empty tank can report it too.
    let probe = Chemical::new(CHEMICALS[0]).expect("a registered chemical");
    let capacity = handler.capacity(host, 0, &probe);
    match held {
        Some(chemical) => {
            format!("chemical: {block} holds {chemical} {amount}/{capacity} (answered by {owner})")
        }
        None => format!("chemical: {block} holds nothing 0/{capacity} (answered by {owner})"),
    }
}

/// `/mekanism chemical list`: the chemicals this port knows and the block
/// capabilities the game publishes, by name (the table another mod's crate asks).
pub(crate) fn chemical_list(host: &mut dyn ServerHost) -> String {
    let capabilities = mod_api::block_capability_names(&*host).join(", ");
    // The id is looked up in the table, so the answer claims a member, not a size:
    // a table without it would say "none" and the check would fail.
    let hydrogen = crate::runtime::known_chemical("mekanism:hydrogen").unwrap_or("none");
    format!(
        "chemical: {} chemicals, {hydrogen} among them; published capabilities {capabilities}",
        CHEMICALS.len()
    )
}

/// `/mekanism chemical insert <amount> <id> <pos>`: put a chemical into the
/// tank through the capability, as another mod's crate would.
pub(crate) fn chemical_insert(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    id: &str,
    amount: u32,
) -> String {
    let Some(level) = host.level(host.dimension()) else {
        return format!("chemical: no level for {pos:?}");
    };
    let block = level
        .block(&*host, pos)
        .map_or_else(|| "nothing".to_owned(), |id| id.id(&*host));
    let Some(handler) = level.capability(host, &CHEMICAL, pos, None) else {
        return format!(
            "chemical: no chemical handler at {} {} {}",
            pos[0], pos[1], pos[2]
        );
    };
    let Some(chemical) = Chemical::new(id) else {
        return format!("chemical: {id} is not a chemical id");
    };
    // The port's registry check, by name: the id a caller asked for is the id
    // the answer names, whether it is unknown or the tank refuses it.
    if crate::runtime::known_chemical(id).is_none() {
        return format!("chemical: unknown chemical {id}");
    }
    let went =
        host.transaction(|host, tx| Ok::<u32, ()>(handler.insert(host, tx, 0, &chemical, amount)));
    let went = went.unwrap_or(0);
    let held = handler.amount(host, 0);
    let capacity = handler.capacity(host, 0, &chemical);
    format!("chemical: inserted {went} {id} into {block} ({held}/{capacity})")
}

/// `/mekanism chemical drain <amount> <pos>`: take a chemical out of the tank
/// through the capability and put it into the gauge dropper in the caller's main
/// hand through the item capability, moving at most the dropper's transfer rate.
pub(crate) fn chemical_drain(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
    amount: u32,
) -> String {
    drain_tank_into_dropper(host, source, pos, 0, amount)
}

/// `/mekanism chemical drain-tank <amount> <tank> <pos>`: the same drain from a
/// named tank of a handler with more than one (the washer's output slurry is
/// tank 1; its tank 0 is the dirty input automation fills).
pub(crate) fn chemical_drain_tank(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
    tank: usize,
    amount: u32,
) -> String {
    drain_tank_into_dropper(host, source, pos, tank, amount)
}

fn drain_tank_into_dropper(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
    tank: usize,
    amount: u32,
) -> String {
    let CommandSource::Player { player, .. } = source else {
        return "chemical: draining needs a player holding the gauge dropper".to_owned();
    };
    let Some(level) = host.level(host.dimension()) else {
        return format!("chemical: no level for {pos:?}");
    };
    let Some(handler) = level.capability(host, &CHEMICAL, pos, None) else {
        return format!(
            "chemical: no chemical handler at {} {} {}",
            pos[0], pos[1], pos[2]
        );
    };
    let Some(held) = handler.resource(host, tank) else {
        return "chemical: the tank holds nothing to drain".to_owned();
    };
    let place = ItemPlace::PlayerHand {
        player: player.uuid(),
        hand: Hand::Main,
    };
    let Some(item) = place.capability(host, &ITEM_CHEMICAL) else {
        return "chemical: no gauge dropper in the main hand".to_owned();
    };
    let asked = dropper_move(amount);
    let moved = host.transaction(|host, tx| {
        let came = handler.extract(host, tx, tank, &held, asked);
        if came == 0 {
            return Ok::<u32, ()>(0);
        }
        let went = item.insert(host, tx, 0, &held, came);
        // What the dropper would not take goes back: never lose a chemical to a
        // refused insert.
        if went < came {
            let _ = handler.insert(host, tx, tank, &held, came - went);
        }
        Ok(went)
    });
    let moved = moved.unwrap_or(0);
    let held_amount = handler.amount(host, tank);
    let tank_capacity = handler.capacity(host, tank, &held);
    let dropper = item.amount(host, 0);
    format!(
        "chemical: drained {moved} {held} into the gauge dropper \
         (tank {held_amount}/{tank_capacity}, dropper {dropper}/{DROPPER_CAPACITY})"
    )
}

/// `/mekanism chemical dropper`: what the gauge dropper in the caller's main
/// hand holds, read through the item capability.
pub(crate) fn chemical_dropper(host: &mut dyn ServerHost, source: &CommandSource) -> String {
    let CommandSource::Player { player, .. } = source else {
        return "chemical: the dropper is read from a player's hand".to_owned();
    };
    let place = ItemPlace::PlayerHand {
        player: player.uuid(),
        hand: Hand::Main,
    };
    let Some(item) = place.capability(host, &ITEM_CHEMICAL) else {
        return "chemical: no gauge dropper in the main hand".to_owned();
    };
    match item.resource(host, 0) {
        Some(chemical) => {
            let amount = item.amount(host, 0);
            format!("chemical: the gauge dropper holds {chemical} {amount}/{DROPPER_CAPACITY}")
        }
        None => format!("chemical: the gauge dropper holds nothing 0/{DROPPER_CAPACITY}"),
    }
}

/// The fluid resource a call is built from: water's, resolved at registration.
fn water_resource() -> Option<FluidResource> {
    WATER_KEY
        .get()
        .map(|key| FluidResource::of(&FluidStack::new(*key, 1)))
}

/// A fluid's registry name for a readout (`minecraft:water`).
fn fluid_name(host: &dyn Host, resource: &FluidResource) -> String {
    match resource.fluid().key() {
        Some(key) => key.id(host),
        None => "an unresolved fluid".to_owned(),
    }
}

/// `/mekanism fluid read <pos>`: asks `horizon:fluids` at `pos` — the **game's**
/// capability, not this port's — and prints what the handler answers.
pub(crate) fn fluid_read(host: &mut dyn ServerHost, pos: [i32; 3]) -> String {
    let Some(level) = host.level(host.dimension()) else {
        return format!("fluid: no level for {pos:?}");
    };
    let block = level
        .block(&*host, pos)
        .map_or_else(|| "nothing".to_owned(), |id| id.id(&*host));
    let Some(handler) = level.capability(host, &FLUIDS, pos, None) else {
        return format!(
            "fluid: no fluid handler at {} {} {}",
            pos[0], pos[1], pos[2]
        );
    };
    let owner = handler.owner();
    let held = handler.resource(host, 0);
    let amount = handler.amount(host, 0);
    let capacity = water_resource().map_or(0, |probe| handler.capacity(host, 0, &probe));
    match held {
        Some(resource) => {
            let name = fluid_name(&*host, &resource);
            format!("fluid: {block} holds {name} {amount}/{capacity} (answered by {owner})")
        }
        None => format!("fluid: {block} holds nothing 0/{capacity} (answered by {owner})"),
    }
}

/// `/mekanism fluid insert <amount> <pos>`: puts up to `amount` millibuckets of
/// water into the block's tank through `horizon:fluids`, as another mod's crate
/// would. The tank's own table decides what it takes (the washer's takes water
/// and nothing else), so a block with no fluid tank, or one that refuses the
/// fluid, moves nothing.
pub(crate) fn fluid_insert(host: &mut dyn ServerHost, pos: [i32; 3], amount: u32) -> String {
    let Some(level) = host.level(host.dimension()) else {
        return format!("fluid: no level for {pos:?}");
    };
    let block = level
        .block(&*host, pos)
        .map_or_else(|| "nothing".to_owned(), |id| id.id(&*host));
    let Some(handler) = level.capability(host, &FLUIDS, pos, None) else {
        return format!(
            "fluid: no fluid handler at {} {} {}",
            pos[0], pos[1], pos[2]
        );
    };
    let Some(water) = water_resource() else {
        return "fluid: the water fluid key was not resolved at registration".to_owned();
    };
    let went = host
        .transaction(|host, tx| Ok::<u32, ()>(handler.insert(host, tx, 0, &water, amount)))
        .unwrap_or(0);
    let held = handler.amount(host, 0);
    let capacity = handler.capacity(host, 0, &water);
    format!("fluid: inserted {went} {WATER} into {block} ({held}/{capacity})")
}

/// Whether one of this port's chemical tanks stands at `pos`: `/mekanism info`
/// answers its tank rather than claiming there is no block entity where there is
/// one (the tank has no FE buffer to report).
pub(crate) fn is_chemical_tank(host: &mut dyn ServerHost, pos: [i32; 3]) -> bool {
    host.level(host.dimension())
        .and_then(|level| level.block(&*host, pos))
        .is_some_and(|id| id.id(&*host) == CHEMICAL_TANK)
}

/// The id and buffer capacity of one of this port's blocks at `pos`: `None`
/// when nothing stands there or the block gives no buffer.
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
    if is_factory(&id) {
        let progress = read_factory_progress(&saved, FACTORY_PROCESSES);
        let items: Vec<Option<ItemStack>> = (0..FACTORY_SLOTS)
            .map(|slot| entity.item_slot(&*host, slot as u32).ok().flatten())
            .collect();
        let processes = factory_processes(&*host, &items, &progress);
        return Some(factory_info_text(
            &id,
            capacity,
            read_energy(&saved, capacity),
            &processes,
        ));
    }
    if is_washer(&id) {
        let (input, output) = read_washer_tanks(&saved);
        return Some(washer_info_text(
            &id,
            capacity,
            read_machine(&saved, capacity),
            &read_washer_fluid(&saved),
            &input,
            &output,
        ));
    }
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
    if is_factory(&id) {
        let progress = read_factory_progress(&saved, FACTORY_PROCESSES);
        let furthest = progress.iter().copied().max().unwrap_or(0);
        return Some(format!(
            "Progress {furthest}/{} of {} processes.",
            machine_ticks(&id).unwrap_or(0),
            progress.len()
        ));
    }
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

    /// The tank's saved shape: `chemical` + `amount`, the empty chemical at 0 for
    /// an empty tank, and nothing else invented.
    #[test]
    fn the_saved_chemical_and_amount_read_back() {
        let mut saved = Tag::Compound(Compound::new());
        write_tank(&mut saved, &Tank::new("mekanism:hydrogen", 4_000));
        assert_eq!(
            saved.get("chemical"),
            Some(&Tag::String("mekanism:hydrogen".into())),
            "the saved chemical is the id a reader sees with `/data get block`"
        );
        assert_eq!(saved.get("amount"), Some(&Tag::Int(4_000)));
        assert_eq!(
            read_tank(&saved),
            Tank::new("mekanism:hydrogen", 4_000),
            "and it reads back as the tank it was"
        );
        let expected = match &saved {
            Tag::Compound(nbt) => nbt.clone(),
            _ => unreachable!("a compound was just written"),
        };
        assert_eq!(
            tank_patch(&Tank::new("mekanism:hydrogen", 4_000)),
            expected,
            "the patch a write merges is the same two keys"
        );
        // An emptied tank names the empty chemical at 0 rather than a stale id.
        write_tank(&mut saved, &Tank::default());
        assert_eq!(
            saved.get("chemical"),
            Some(&Tag::String(EMPTY_CHEMICAL.into()))
        );
        assert_eq!(saved.get("amount"), Some(&Tag::Int(0)));
        assert_eq!(read_tank(&saved), Tank::default(), "and reads as empty");
        // A compound with no chemical at all (a fresh block entity) reads empty.
        assert_eq!(read_tank(&Tag::Compound(Compound::new())), Tank::default());
        // An unknown id saved by a later version still reads back: the registry
        // check belongs to a write, not to a read.
        let mut future = Tag::Compound(Compound::new());
        if let Tag::Compound(nbt) = &mut future {
            nbt.insert(
                "chemical".into(),
                Tag::String("mekanism:unobtainium".into()),
            );
            nbt.insert("amount".into(), Tag::Int(7));
        }
        assert_eq!(read_tank(&future).held(), Some(("mekanism:unobtainium", 7)));
    }

    /// Slice 4 is one block and one item, and the tank takes the value its tier
    /// gives it.
    #[test]
    fn the_chemical_family_is_one_block_one_item_and_one_tank() {
        assert_eq!(
            CHEMICAL_TANK, "mekanism:basic_chemical_tank",
            "/mekanism info answers the tank at this id"
        );
        assert_eq!(GAUGE_DROPPER, "mekanism:gauge_dropper");
        assert_eq!(TANK_TYPE, "basic_chemical_tank");
        assert_eq!(TANK_CAPACITY, 64_000, "ChemicalTankTier.BASIC");
        assert_eq!(DROPPER_CAPACITY, 16_000, "GearConfig.gaugeDropperCapacity");
        assert_eq!(
            machine_capacity(CHEMICAL_TANK),
            None,
            "the tank has no energy buffer (its /mekanism info line is its tank's)"
        );
        assert!(
            !CHEMICALS.is_empty() && CHEMICALS.contains(&"mekanism:hydrogen"),
            "the id table the commands validate against is Mekanism's"
        );
    }

    /// The Chemical Injection Chamber is the shared machine shape with the jar's
    /// own numbers: a 160,000 FE insert-only buffer, a 210-unit gas tank, the
    /// same three slots and a 200-tick operation.
    #[test]
    fn the_injection_machine_is_the_jar_s_400_fe_a_tick_and_its_gas_buffer() {
        assert_eq!(machine_capacity(INJECTION), Some(INJECTION_CAPACITY));
        assert_eq!(
            INJECTION_CAPACITY, 160_000,
            "StorageConfig.chemicalInjectionChamber"
        );
        assert_eq!(machine_ticks(INJECTION), Some(INJECTION_TICKS));
        assert_eq!(
            INJECTION_OPERATION.usage, 400,
            "UsageConfig.chemicalInjectionChamber"
        );
        let buffer = machine_battery(INJECTION_CAPACITY);
        assert_eq!(
            (buffer.max_insert, buffer.max_extract),
            (INJECTION_CAPACITY, 0)
        );
        assert_eq!(INJECTION_TANK, 210, "MAX_GAS");
        assert_eq!(container().slots, MENU_SLOTS, "the same three-slot menu");
    }

    /// The Purification Chamber is the same machine shape with the jar's own
    /// numbers: an 80,000 FE insert-only buffer, a 210-unit oxygen tank, the
    /// same three slots and a 200-tick operation.
    #[test]
    fn the_purification_machine_is_the_jar_s_200_fe_a_tick_and_its_gas_buffer() {
        assert_eq!(machine_capacity(PURIFICATION), Some(PURIFY_CAPACITY));
        assert_eq!(PURIFY_CAPACITY, 80_000, "StorageConfig.purificationChamber");
        assert_eq!(machine_ticks(PURIFICATION), Some(PURIFY_TICKS));
        assert_eq!(
            PURIFY_OPERATION.usage, 200,
            "UsageConfig.purificationChamber"
        );
        let buffer = machine_battery(PURIFY_CAPACITY);
        assert_eq!(
            (buffer.max_insert, buffer.max_extract),
            (PURIFY_CAPACITY, 0)
        );
        assert_eq!(PURIFY_TANK, 210, "MAX_GAS");
        assert_eq!(
            purifying::lookup("minecraft:iron_ore", "mekanism:oxygen")
                .map(|recipe| recipe.output.count),
            Some(3),
            "the machine's table is the jar's purifying one"
        );
    }

    /// The machine's saved compound carries its gas beside its FE and progress,
    /// and each reads back without disturbing the others.
    #[test]
    fn the_gas_machine_saves_its_tank_beside_its_energy_and_progress() {
        let mut saved = Tag::Compound(Compound::new());
        write_state(&mut saved, 80_000, 60);
        write_tank(&mut saved, &Tank::new("mekanism:hydrogen_chloride", 210));
        assert_eq!(
            read_machine(&saved, INJECTION_CAPACITY),
            Machine {
                energy: 80_000,
                progress: 60
            }
        );
        assert_eq!(
            read_tank(&saved),
            Tank::new("mekanism:hydrogen_chloride", 210),
            "the machine's own tank is the same saved shape as the tank block's"
        );
    }

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
    fn every_buffer_block_has_a_capacity_and_only_the_machines_a_length() {
        assert_eq!(machine_capacity(SMELTER), Some(SMELTER_CAPACITY));
        assert_eq!(machine_capacity(CHAMBER), Some(CHAMBER_CAPACITY));
        assert_eq!(machine_capacity(CRUSHER), Some(CRUSHER_CAPACITY));
        assert_eq!(machine_capacity(INJECTION), Some(INJECTION_CAPACITY));
        assert_eq!(machine_capacity(PURIFICATION), Some(PURIFY_CAPACITY));
        assert_eq!(machine_capacity(CUBE), Some(CUBE_CAPACITY));
        assert_eq!(machine_capacity(CABLE), Some(CABLE_CAPACITY));
        assert_eq!(machine_capacity("minecraft:chest"), None);
        assert_eq!(machine_ticks(SMELTER), Some(200));
        assert_eq!(machine_ticks(CHAMBER), Some(200));
        assert_eq!(machine_ticks(CRUSHER), Some(200));
        assert_eq!(machine_ticks(INJECTION), Some(200));
        assert_eq!(machine_ticks(PURIFICATION), Some(200));
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

#[cfg(test)]
mod factory_support_tests {
    use super::*;

    /// Every basic factory declares one input and one output slot per process,
    /// and no vanilla menu (upstream draws its own factory GUI, which this port
    /// does not have).
    #[test]
    fn a_basic_factory_declares_one_input_output_pair_per_process() {
        let container = factory_container();
        assert_eq!(container.slots, FACTORY_SLOTS, "three pairs");
        assert_eq!(
            container.inputs,
            vec![0, 1, 2],
            "automation fills the three process inputs"
        );
        assert_eq!(
            container.outputs,
            vec![3, 4, 5],
            "and takes the three process outputs"
        );
        assert_eq!(
            container.faces,
            std::array::from_fn(|_| vec![0, 1, 2, 3, 4, 5]),
            "every face reaches all six slots: there is no side configuration"
        );
        assert_eq!(container.menu, None, "no vanilla menu fits six slots");
    }

    /// The five basic factories carry the jar's own numbers: half the base
    /// machine's buffer per process, the base machine's usage, a 200-tick
    /// operation and (through the gas rows) a 630-unit tank.
    #[test]
    fn the_five_basic_factories_declare_the_jar_s_numbers() {
        assert_eq!(machine_capacity(SMELTING_FACTORY), Some(30_000));
        assert_eq!(machine_capacity(ENRICHING_FACTORY), Some(30_000));
        assert_eq!(machine_capacity(CRUSHING_FACTORY), Some(30_000));
        assert_eq!(machine_capacity(INJECTING_FACTORY), Some(240_000));
        assert_eq!(machine_capacity(PURIFYING_FACTORY), Some(120_000));
        assert_eq!(machine_ticks(PURIFYING_FACTORY), Some(FACTORY_TICKS));
        assert_eq!(machine_ticks(SMELTING_FACTORY), Some(FACTORY_TICKS));
        let buffer = machine_battery(PURIFYING_FACTORY_CAPACITY);
        assert_eq!(
            (buffer.max_insert, buffer.max_extract),
            (PURIFYING_FACTORY_CAPACITY, 0),
            "a factory takes energy and gives none"
        );
        assert_eq!(
            INJECTING_FACTORY_TANK, 630,
            "MAX_GAS x FactoryTier.BASIC.processes"
        );
        assert_eq!(PURIFYING_FACTORY_TANK, 630);
    }

    /// `/mekanism info` at a factory reports the sum of its process slots, so
    /// nine clumps in three output slots read as `x9` — one line, as the single
    /// machines answer.
    #[test]
    fn a_factory_reports_the_sum_of_its_process_slots() {
        let processes = [
            FactoryProcess {
                progress: 0,
                input: ItemAmount::default(),
                output: ItemAmount::new("mekanism:clump_iron", 3),
            },
            FactoryProcess {
                progress: 0,
                input: ItemAmount::default(),
                output: ItemAmount::new("mekanism:clump_iron", 3),
            },
            FactoryProcess {
                progress: 0,
                input: ItemAmount::default(),
                output: ItemAmount::new("mekanism:clump_iron", 3),
            },
        ];
        assert_eq!(
            factory_info_text(PURIFYING_FACTORY, PURIFYING_FACTORY_CAPACITY, 0, &processes),
            "mekanism:basic_purifying_factory: 0/120000 FE (3 processes), \
             progress 0/200, input empty x0, output mekanism:clump_iron x9"
        );
        // Different items in different slots are not summed into a lie.
        let mixed = [
            FactoryProcess {
                progress: 120,
                input: ItemAmount::new("minecraft:iron_ore", 1),
                output: ItemAmount::new("mekanism:clump_iron", 3),
            },
            FactoryProcess {
                progress: 120,
                input: ItemAmount::new("minecraft:copper_ore", 2),
                output: ItemAmount::new("mekanism:clump_copper", 6),
            },
            FactoryProcess::default(),
        ];
        assert_eq!(
            factory_info_text(PURIFYING_FACTORY, PURIFYING_FACTORY_CAPACITY, 1_000, &mixed),
            "mekanism:basic_purifying_factory: 1000/120000 FE (3 processes), \
             progress 120/200, input mixed x3, output mixed x9"
        );
    }

    /// The washer is the shared machine shape with the jar's own numbers, and it
    /// has **no container**: its inputs are tanks, not slots (`MAX_FLUID` and two
    /// `MAX_SLURRY` tanks), so `/mekanism info` reports the tanks.
    #[test]
    fn the_washer_is_the_jar_s_80_000_fe_buffer_and_three_tanks() {
        assert_eq!(WASHER, "mekanism:chemical_washer");
        assert_eq!(WASHER_TYPE, "chemical_washer");
        assert_eq!(machine_capacity(WASHER), Some(WASHER_CAPACITY));
        assert_eq!(WASHER_CAPACITY, 80_000, "StorageConfig.chemicalWasher");
        assert_eq!(machine_ticks(WASHER), Some(WASHER_TICKS));
        assert_eq!(WASHER_OPERATION.usage, 200, "UsageConfig.chemicalWasher");
        let buffer = machine_battery(WASHER_CAPACITY);
        assert_eq!(
            (buffer.max_insert, buffer.max_extract),
            (WASHER_CAPACITY, 0),
            "a machine takes energy and gives none"
        );
        assert!(is_washer(WASHER));
        assert!(!is_washer(CHEMICAL_TANK));
    }

    /// The washer's one fluid tank: 10,000 mB of water, reachable from every
    /// face, filled by a capability call and never drained — upstream's
    /// `BasicFluidTank.input(MAX_FLUID, ...)`.
    #[test]
    fn the_washer_declares_one_water_tank_a_fill_may_reach() {
        let tanks = washer_fluids();
        assert_eq!(tanks.tanks, vec![WASHER_WATER as i32], "MAX_FLUID");
        assert_eq!(tanks.inputs, vec![0], "a fill may reach it");
        assert_eq!(tanks.outputs, Vec::<usize>::new(), "a drain may not");
        assert_eq!(
            tanks.faces,
            std::array::from_fn(|_| vec![0]),
            "every face reaches the one tank"
        );
        // The tank a chemical block declares is none at all.
        assert!(machine_capacity(CHEMICAL_TANK).is_none());
    }

    /// The washer's three tanks are saved beside each other: the water in the
    /// game's `fluids` list (its own shape), the slurries in the port's
    /// `input_*`/`output_*` keys, and each reads back exactly.
    #[test]
    fn the_washer_s_three_tanks_save_and_read_back() {
        let mut saved = Tag::Compound(Compound::new());
        write_state(&mut saved, 40_000, 67);
        write_washer_fluid(&mut saved, &FluidTank::new(WATER, 9_995));
        write_washer_tanks(
            &mut saved,
            &Tank::new("mekanism:dirty_iron", 999),
            &Tank::new("mekanism:clean_iron", 1),
        );
        assert_eq!(
            read_machine(&saved, WASHER_CAPACITY),
            Machine {
                energy: 40_000,
                progress: 67
            }
        );
        assert_eq!(read_washer_fluid(&saved), FluidTank::new(WATER, 9_995));
        assert_eq!(
            read_washer_tanks(&saved),
            (
                Tank::new("mekanism:dirty_iron", 999),
                Tank::new("mekanism:clean_iron", 1)
            )
        );
        // The game's own list shape, so `/data get block` and another mod's
        // fluid handler read the same bytes.
        let Tag::List(list) = saved.get("fluids").expect("the fluids list") else {
            panic!("fluids is not a list");
        };
        let tank = &list[0];
        assert_eq!(
            tank.get("Fluid"),
            Some(&Tag::String("minecraft:water".into()))
        );
        assert_eq!(tank.get("Amount"), Some(&Tag::Int(9_995)));
        // An emptied tank writes the empty fluid at 0, not a stale id.
        write_washer_fluid(&mut saved, &FluidTank::default());
        assert_eq!(read_washer_fluid(&saved), FluidTank::default());
        assert_eq!(
            read_washer_tanks(&Tag::Compound(Compound::new())),
            (Tank::default(), Tank::default()),
            "a fresh compound reads as three empty tanks"
        );
    }

    /// The line the in-game check asserts, with the numbers it asserts.
    #[test]
    fn the_washer_info_line_is_the_one_the_e2e_asserts() {
        assert_eq!(
            washer_info_text(
                WASHER,
                WASHER_CAPACITY,
                Machine {
                    energy: 40_000,
                    progress: 0
                },
                &FluidTank::new(WATER, 9_995),
                &Tank::new("mekanism:dirty_iron", 999),
                &Tank::new("mekanism:clean_iron", 1),
            ),
            "mekanism:chemical_washer: 40000/80000 FE, progress 0/200, \
             water 9995/10000 mB, input mekanism:dirty_iron x999, \
             output mekanism:clean_iron x1"
        );
        assert_eq!(
            washer_info_text(
                WASHER,
                WASHER_CAPACITY,
                Machine::default(),
                &FluidTank::default(),
                &Tank::default(),
                &Tank::default(),
            ),
            "mekanism:chemical_washer: 0/80000 FE, progress 0/200, \
             water 0/10000 mB, input empty x0, output empty x0"
        );
    }
}
