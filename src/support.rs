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
    CONDENSENTRATOR_CAPACITY, CONDENSENTRATOR_OPERATION, CONDENSENTRATOR_TANK,
    CONDENSENTRATOR_TICKS, CRUSHER_CAPACITY, CRUSHER_OPERATION, CRUSHER_TICKS,
    CRUSHING_FACTORY_CAPACITY, CRYSTALLIZER_CAPACITY, CRYSTALLIZER_OPERATION, CRYSTALLIZER_TANK,
    CRYSTALLIZER_TICKS, CUBE_CAPACITY, CUBE_RATE, CondensentratorMode, CrystallizeRecipe,
    DISSOLUTION_CAPACITY, DISSOLUTION_OPERATION, DISSOLUTION_TANK, DISSOLUTION_TICKS,
    DROPPER_CAPACITY, EMPTY_CHEMICAL, ENRICHING_FACTORY_CAPACITY, FACTORY_PROCESSES, FACTORY_TICKS,
    FactoryProcess, FluidTank, GasRecipe, INFUSER_CAPACITY, INFUSER_OPERATION, INFUSER_TANK,
    INFUSER_TICKS, INJECTING_FACTORY_CAPACITY, INJECTING_FACTORY_TANK, INJECTION_CAPACITY,
    INJECTION_OPERATION, INJECTION_TANK, INJECTION_TICKS, IdleReason, ItemAmount, Machine,
    MachineRecipe, OXIDIZER_CAPACITY, OXIDIZER_OPERATION, OXIDIZER_TANK, OXIDIZER_TICKS, Operation,
    Outcome, PURIFY_CAPACITY, PURIFY_OPERATION, PURIFY_TANK, PURIFY_TICKS,
    PURIFYING_FACTORY_CAPACITY, PURIFYING_FACTORY_TANK, SEPARATOR_CAPACITY, SEPARATOR_FLUID,
    SEPARATOR_GAS, SEPARATOR_OPERATION, SEPARATOR_TICKS, SLURRIES, SMELTER_CAPACITY,
    SMELTER_OPERATION, SMELTER_TICKS, SMELTING_FACTORY_CAPACITY, TANK_CAPACITY, Tank,
    WASHER_CAPACITY, WASHER_OPERATION, WASHER_SLURRY, WASHER_TICKS, WASHER_WATER, WATER,
    clamp_energy, condensentrating, crushing, crystallizer_pull, crystallizing, dissolve_pull,
    dissolving, dropper_move, energy_after_extract, energy_after_insert, enriching,
    extract_chemical, factory_gas_want, gas_pull, infusing, injecting, insert_chemical,
    operations_per_tick, oxidizing, process_condensentrate_tick, process_crystal_tick,
    process_dissolve_tick, process_factory_tick, process_gas_factory_tick, process_gas_tick,
    process_infuse_tick, process_oxidize_tick, process_separate_tick, process_tick,
    process_wash_tick, purifying, room_for, room_in_tank, separating, smelting, washer_pull,
    washing,
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

/// Slice 9's machine: the Chemical Crystallizer, the washer's consumer — a clean
/// slurry in, a crystal item out.
pub(crate) const CRYSTALLIZER: &str = "mekanism:chemical_crystallizer";

/// Slice 15's machine: the Chemical Dissolution Chamber, the head of the slurry
/// chain — an ore and sulfuric acid in, a dirty slurry out.
pub(crate) const DISSOLUTION: &str = "mekanism:chemical_dissolution_chamber";

/// Slice 17's machines: the head of the acid chain. The oxidizer makes SO2 from
/// sulfur dust; the infuser turns SO2 + oxygen into SO3 and SO3 + water vapour
/// into the sulfuric acid the dissolution chamber consumes.
pub(crate) const OXIDIZER: &str = "mekanism:chemical_oxidizer";
pub(crate) const INFUSER: &str = "mekanism:chemical_infuser";

/// Slice 18's machine: the Electrolytic Separator, the tail of the acid chain's
/// feed — water in, hydrogen and oxygen out, so the infuser's two gas inputs no
/// longer need a hand-inserted chemical.
pub(crate) const SEPARATOR: &str = "mekanism:electrolytic_separator";

/// Slice 19's machine: the rotary Condensentrator, the last hand-filled feed of
/// the acid chain once the separator makes oxygen — water in, water vapour out.
pub(crate) const CONDENSENTRATOR: &str = "mekanism:rotary_condensentrator";

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
const CRYSTALLIZER_TYPE: &str = "chemical_crystallizer";
const DISSOLUTION_TYPE: &str = "chemical_dissolution_chamber";
const OXIDIZER_TYPE: &str = "chemical_oxidizer";
const INFUSER_TYPE: &str = "chemical_infuser";
const SEPARATOR_TYPE: &str = "electrolytic_separator";
const CONDENSENTRATOR_TYPE: &str = "rotary_condensentrator";

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

/// The crystallizer's one slot: its output.
const CRYSTAL_SLOT: usize = 0;

/// The dissolution chamber's one slot: the ore (or raw ore) it dissolves.
const DISSOLVE_SLOT: usize = 0;

/// The oxidizer's one slot: the sulfur dust it burns.
const OXIDIZE_SLOT: usize = 0;

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
    // Slice 9: the Chemical Crystallizer — the washer's consumer. One
    // 10,000-unit tank answered through `mekanism:chemical` that takes a clean
    // slurry with a row and nothing else (upstream's `containsRecipe`), a
    // 160,000 FE insert-only buffer, and one output slot for the crystal. It has
    // no input slot: its input is the tank, as upstream's is; the chemical-fill
    // and energy slots upstream also declares are container conveniences this
    // port does not draw (recorded as a deviation).
    let key = registrar.block_key(CRYSTALLIZER);
    registrar.block_container(&key, crystallizer_container());
    registrar.block_energy(&key, machine_battery(CRYSTALLIZER_CAPACITY));
    let kind = registrar
        .block_entity_type(CRYSTALLIZER_TYPE)
        .blocks(&[CRYSTALLIZER_TYPE])
        .build();
    registrar.block_entity_capability(&CHEMICAL, &kind, |_, at| {
        Some(Box::new(CrystallizerTank {
            side: at,
            capacity: CRYSTALLIZER_TANK,
        }) as Box<dyn ResourceHandler<Chemical>>)
    });
    if let Err(error) = registrar.block_entity_part(&kind, CrystallizerPart) {
        context.refuse(&error.to_string());
    }
    // Slice 15: the Chemical Dissolution Chamber — the head of the slurry chain.
    // A one-slot container (the ore), a 160,000 FE insert-only buffer, and two
    // 10,000-unit tanks answered through `mekanism:chemical`: tank 0 the acid it
    // takes (`MAX_CHEMICAL`, fill only), tank 1 the dirty slurry it makes
    // (extract only). One operation is 100 ticks at 400 FE/t, and every row in
    // the jar spends its acid per tick.
    let key = registrar.block_key(DISSOLUTION);
    registrar.block_container(&key, dissolution_container());
    registrar.block_energy(&key, machine_battery(DISSOLUTION_CAPACITY));
    let kind = registrar
        .block_entity_type(DISSOLUTION_TYPE)
        .blocks(&[DISSOLUTION_TYPE])
        .build();
    registrar.block_entity_capability(&CHEMICAL, &kind, |_, at| {
        Some(Box::new(DissolutionTanks {
            side: at,
            capacity: DISSOLUTION_TANK,
        }) as Box<dyn ResourceHandler<Chemical>>)
    });
    if let Err(error) = registrar.block_entity_part(&kind, DissolutionPart) {
        context.refuse(&error.to_string());
    }
    // Slice 17: the acid chain head. Both machines reuse the declarations the
    // other machines use: a container (the oxidizer's one input slot), an
    // insert-only energy buffer and one block entity type each. The oxidizer's
    // chemical handler has ONE output-only tank it makes gas in; the infuser's
    // has THREE — tank 0 and 1 its two insert-only gas inputs, tank 2 its
    // extract-only centre output. The machines have no pipes upstream here, so
    // the oxidizer pushes its product into a neighbour and the infuser pulls
    // both inputs from its neighbours through `mekanism:chemical` (the gas
    // machines' recorded deviation, applied symmetrically).
    let key = registrar.block_key(OXIDIZER);
    registrar.block_container(&key, oxidizer_container());
    registrar.block_energy(&key, machine_battery(OXIDIZER_CAPACITY));
    let kind = registrar
        .block_entity_type(OXIDIZER_TYPE)
        .blocks(&[OXIDIZER_TYPE])
        .build();
    registrar.block_entity_capability(&CHEMICAL, &kind, |_, at| {
        Some(Box::new(OxidizerTank {
            side: at,
            capacity: OXIDIZER_TANK,
        }) as Box<dyn ResourceHandler<Chemical>>)
    });
    if let Err(error) = registrar.block_entity_part(&kind, OxidizerPart) {
        context.refuse(&error.to_string());
    }
    let key = registrar.block_key(INFUSER);
    registrar.block_energy(&key, machine_battery(INFUSER_CAPACITY));
    let kind = registrar
        .block_entity_type(INFUSER_TYPE)
        .blocks(&[INFUSER_TYPE])
        .build();
    registrar.block_entity_capability(&CHEMICAL, &kind, |_, at| {
        Some(Box::new(InfuserTanks {
            side: at,
            capacity: INFUSER_TANK,
        }) as Box<dyn ResourceHandler<Chemical>>)
    });
    if let Err(error) = registrar.block_entity_part(&kind, InfuserPart) {
        context.refuse(&error.to_string());
    }
    // Slice 18: the Electrolytic Separator. The acid chain's last hand-filled
    // inputs are water and oxygen: this machine makes both from water, with the
    // washer's fluid declaration (one 24,000 mB water tank, fill only) and two
    // 2,400-unit chemical output tanks beside its 160,000 FE insert-only buffer.
    // Like the oxidizer it pushes its products to neighbours through
    // `mekanism:chemical` — this port ships no ejectors or pipes.
    let key = registrar.block_key(SEPARATOR);
    registrar.block_energy(&key, machine_battery(SEPARATOR_CAPACITY));
    registrar.block_fluids(&key, separator_fluids());
    let kind = registrar
        .block_entity_type(SEPARATOR_TYPE)
        .blocks(&[SEPARATOR_TYPE])
        .build();
    registrar.block_entity_capability(&CHEMICAL, &kind, |_, at| {
        Some(Box::new(SeparatorTanks {
            side: at,
            capacity: SEPARATOR_GAS,
        }) as Box<dyn ResourceHandler<Chemical>>)
    });
    if let Err(error) = registrar.block_entity_part(&kind, SeparatorPart) {
        context.refuse(&error.to_string());
    }
    // Slice 19: the rotary Condensentrator. One 10,000-unit water tank through
    // the game's own `horizon:fluids` and one 10,000-unit gas tank through
    // `mekanism:chemical`, beside a 20,000 FE insert-only buffer. The default
    // mode is fluid -> gas (the acid chain needs water vapour), and the gas is
    // pushed to neighbours through the chemical capability — this port has no
    // ejectors or pipes.
    let key = registrar.block_key(CONDENSENTRATOR);
    registrar.block_energy(&key, machine_battery(CONDENSENTRATOR_CAPACITY));
    registrar.block_fluids(&key, condensentrator_fluids());
    let kind = registrar
        .block_entity_type(CONDENSENTRATOR_TYPE)
        .blocks(&[CONDENSENTRATOR_TYPE])
        .build();
    registrar.block_entity_capability(&CHEMICAL, &kind, |_, at| {
        Some(Box::new(CondensentratorTanks {
            side: at,
            capacity: CONDENSENTRATOR_TANK,
        }) as Box<dyn ResourceHandler<Chemical>>)
    });
    if let Err(error) = registrar.block_entity_part(&kind, CondensentratorPart) {
        context.refuse(&error.to_string());
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

/// The separator's fluid tanks, as `block_fluids` declares them: one 24,000 mB
/// water tank of upstream's `BasicFluidTank.input(MAX_FLUID, …)` shape — a fill
/// may reach it from every face, but no drain may take from it. The machine
/// spends its water itself when an operation finishes.
pub(crate) fn separator_fluids() -> ModFluidTanks {
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![i32::try_from(SEPARATOR_FLUID).unwrap_or(i32::MAX)];
    tanks.faces = std::array::from_fn(|_| vec![0]);
    tanks.inputs = vec![0];
    tanks.outputs = Vec::new();
    tanks
}

/// The condensentrator's fluid tanks: one 10,000 mB water tank a fill may reach
/// from every face. It is also declared as a drainable output: in gas → fluid
/// mode the machine makes water into this tank, which is the fluid half's output
/// path (`horizon:fluids` extraction and `/mekanism info`). Upstream switches the
/// tank's extract predicate with `mode`; a declaration cannot change per block
/// entity, so the tank is both fillable and drainable on every face and the
/// machine's own tick is the only producer (a recorded deviation).
pub(crate) fn condensentrator_fluids() -> ModFluidTanks {
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![i32::try_from(CONDENSENTRATOR_TANK).unwrap_or(i32::MAX)];
    tanks.faces = std::array::from_fn(|_| vec![0]);
    tanks.inputs = vec![0];
    tanks.outputs = vec![0];
    tanks
}

/// The block entity key upstream writes the direction under:
/// `SerializationConstants.MODE = "mode"` in `readSustainedData`/`writeSustainedData`
/// (`TileEntityRotaryCondensentrator.java:236-245`, tag `a00109e4`). An NBT
/// boolean is the byte 1 (`true`, fluid → gas) or 0 (`false`, gas → fluid).
const CONDENSENTRATOR_MODE_KEY: &str = "mode";

/// Read the direction from a saved compound. A fresh block entity has no key
/// yet; this port's recorded default is fluid → gas (the acid chain's vapour),
/// which is upstream's `true`.
fn read_condensentrator_mode(saved: &Tag) -> CondensentratorMode {
    match saved.get(CONDENSENTRATOR_MODE_KEY) {
        Some(Tag::Byte(0)) => CondensentratorMode::Condensentrating,
        _ => CondensentratorMode::Decondensentrating,
    }
}

/// The patch `/mekanism mode` merges into the block entity: the same key and
/// byte value upstream saves, so `/data get block … mode` reads it back.
fn condensentrator_mode_patch(mode: CondensentratorMode) -> Compound {
    let byte: i8 = match mode {
        CondensentratorMode::Decondensentrating => 1,
        CondensentratorMode::Condensentrating => 0,
    };
    Compound::from([(CONDENSENTRATOR_MODE_KEY.into(), Tag::Byte(byte))])
}

/// Upstream's `nextMode()` flips the boolean and calls `setChanged()`.
fn next_condensentrator_mode(mode: CondensentratorMode) -> CondensentratorMode {
    match mode {
        CondensentratorMode::Decondensentrating => CondensentratorMode::Condensentrating,
        CondensentratorMode::Condensentrating => CondensentratorMode::Decondensentrating,
    }
}

/// The name `/mekanism info` and `/mekanism mode` report.
fn condensentrator_mode_name(mode: CondensentratorMode) -> &'static str {
    match mode {
        CondensentratorMode::Decondensentrating => "decondensentrating",
        CondensentratorMode::Condensentrating => "condensentrating",
    }
}

/// The gas tank's insert gate, from the artefact's `getInitialChemicalTanks`:
/// `canInsert = INTERNAL || !mode`, so foreign automation may insert only in
/// gas → fluid mode, and only a chemical with a rotary row (`isValidGas`).
fn may_insert_condensentrator_gas(mode: CondensentratorMode, id: &str) -> bool {
    mode == CondensentratorMode::Condensentrating
        && condensentrating::recipe_for_chemical(id).is_some()
}

/// The gas tank's extract gate: `canExtract = MANUAL || mode`, so external
/// automation may take the gas out only while it is the output (fluid → gas).
fn may_extract_condensentrator_gas(mode: CondensentratorMode) -> bool {
    mode == CondensentratorMode::Decondensentrating
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

/// The crystallizer's container: one slot, the output the crystal lands in.
///
/// Its input is a chemical (the tank), so there is no input slot at all, and the
/// chemical-fill and energy slots upstream declares are container conveniences
/// this port does not draw (the same deviation the washer records for its four).
pub(crate) fn crystallizer_container() -> ModContainer {
    ModContainer {
        slots: 1,
        faces: std::array::from_fn(|_| vec![CRYSTAL_SLOT]),
        inputs: Vec::new(),
        outputs: vec![CRYSTAL_SLOT],
        menu: None,
    }
}

/// The dissolution chamber's container: one slot, the ore it dissolves.
///
/// Its outputs are chemicals (the two tanks), and the acid-fill, chemical-drain
/// and energy slots upstream declares are container conveniences this port does
/// not draw — the same deviation the washer and the crystallizer record.
pub(crate) fn dissolution_container() -> ModContainer {
    ModContainer {
        slots: 1,
        faces: std::array::from_fn(|_| vec![DISSOLVE_SLOT]),
        inputs: vec![DISSOLVE_SLOT],
        outputs: Vec::new(),
        menu: None,
    }
}

/// The oxidizer's container: one slot, the sulfur dust it burns. Its output is
/// the gas tank, so there is no output item slot; the drain and energy slots
/// upstream declares are the recorded container-convenience deviation.
pub(crate) fn oxidizer_container() -> ModContainer {
    ModContainer {
        slots: 1,
        faces: std::array::from_fn(|_| vec![OXIDIZE_SLOT]),
        inputs: vec![OXIDIZE_SLOT],
        outputs: Vec::new(),
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
        CRYSTALLIZER => Some(CRYSTALLIZER_CAPACITY),
        DISSOLUTION => Some(DISSOLUTION_CAPACITY),
        OXIDIZER => Some(OXIDIZER_CAPACITY),
        INFUSER => Some(INFUSER_CAPACITY),
        SEPARATOR => Some(SEPARATOR_CAPACITY),
        CONDENSENTRATOR => Some(CONDENSENTRATOR_CAPACITY),
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
        CRYSTALLIZER => Some(CRYSTALLIZER_TICKS),
        DISSOLUTION => Some(DISSOLUTION_TICKS),
        OXIDIZER => Some(OXIDIZER_TICKS),
        INFUSER => Some(INFUSER_TICKS),
        SEPARATOR => Some(SEPARATOR_TICKS),
        CONDENSENTRATOR => Some(CONDENSENTRATOR_TICKS),
        _ => None,
    }
}

/// Whether the Chemical Washer stands at `id`: its `/mekanism info` line is its
/// three tanks', not a slot machine's.
pub(crate) fn is_washer(id: &str) -> bool {
    id == WASHER
}

/// Whether the Chemical Crystallizer stands at `id`: its `/mekanism info` line is
/// its tank's and its output slot's.
pub(crate) fn is_crystallizer(id: &str) -> bool {
    id == CRYSTALLIZER
}

/// Whether the Chemical Dissolution Chamber stands at `id`: its `/mekanism info`
/// line is its ore slot's and its two tanks'.
pub(crate) fn is_dissolution(id: &str) -> bool {
    id == DISSOLUTION
}

/// Whether the Chemical Oxidizer stands at `id`: its `/mekanism info` line is
/// its input slot's and its gas tank's.
pub(crate) fn is_oxidizer(id: &str) -> bool {
    id == OXIDIZER
}

/// Whether the Chemical Infuser stands at `id`: its `/mekanism info` line is its
/// three tanks'.
pub(crate) fn is_infuser(id: &str) -> bool {
    id == INFUSER
}

/// Whether the Electrolytic Separator stands at `id`: its `/mekanism info` line
/// reports its water and two gas tanks rather than a slot machine's.
pub(crate) fn is_separator(id: &str) -> bool {
    id == SEPARATOR
}

/// Whether the rotary Condensentrator stands at `id`: its `/mekanism info` line
/// reports its water and gas tanks rather than a slot machine's.
pub(crate) fn is_condensentrator(id: &str) -> bool {
    id == CONDENSENTRATOR
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

/// The `/mekanism info` report line for the Chemical Crystallizer: its buffer,
/// its progress, the clean slurry in its own tank and the crystal in its output
/// slot.
pub(crate) fn crystallizer_info_text(
    id: &str,
    capacity: i32,
    machine: Machine,
    tank: &Tank,
    output: &ItemAmount,
) -> String {
    format!(
        "{id}: {}/{} FE, progress {}/{}, input {} x{}, output {} x{}",
        machine.energy,
        capacity,
        machine.progress,
        CRYSTALLIZER_TICKS,
        tank_name(tank),
        tank.amount,
        stack_name(output),
        output.count
    )
}

/// The `/mekanism info` report line for the Chemical Dissolution Chamber: its
/// buffer, its progress, the ore in its slot and both of its tanks, each named
/// by what it holds (`empty` at 0).
pub(crate) fn dissolution_info_text(
    id: &str,
    capacity: i32,
    machine: Machine,
    acid: &Tank,
    slurry: &Tank,
    input: &ItemAmount,
) -> String {
    format!(
        "{id}: {}/{} FE, progress {}/{}, input {} x{}, acid {} {}/{}, output {} {}/{}",
        machine.energy,
        capacity,
        machine.progress,
        DISSOLUTION_TICKS,
        stack_name(input),
        input.count,
        tank_name(acid),
        acid.amount,
        DISSOLUTION_TANK,
        tank_name(slurry),
        slurry.amount,
        DISSOLUTION_TANK
    )
}

/// The `/mekanism info` report line for the Chemical Oxidizer: its buffer,
/// progress, sulfur slot and the gas tank it makes.
pub(crate) fn oxidizer_info_text(
    id: &str,
    capacity: i32,
    machine: Machine,
    input: &ItemAmount,
    gas: &Tank,
) -> String {
    format!(
        "{id}: {}/{} FE, progress {}/{}, input {} x{}, output {} {}/{}",
        machine.energy,
        capacity,
        machine.progress,
        OXIDIZER_TICKS,
        stack_name(input),
        input.count,
        tank_name(gas),
        gas.amount,
        OXIDIZER_TANK
    )
}

/// The `/mekanism info` report line for the Chemical Infuser: its buffer and
/// all three tanks (left input, right input, centre output).
pub(crate) fn infuser_info_text(
    id: &str,
    capacity: i32,
    machine: Machine,
    left: &Tank,
    right: &Tank,
    output: &Tank,
) -> String {
    format!(
        "{id}: {}/{} FE, progress {}/{}, left {} {}/{}, right {} {}/{}, output {} {}/{}",
        machine.energy,
        capacity,
        machine.progress,
        INFUSER_TICKS,
        tank_name(left),
        left.amount,
        INFUSER_TANK,
        tank_name(right),
        right.amount,
        INFUSER_TANK,
        tank_name(output),
        output.amount,
        INFUSER_TANK
    )
}

/// The `/mekanism info` report line for the Electrolytic Separator: its buffer,
/// its one-tick progress, its water tank and its two gas output tanks.
pub(crate) fn separator_info_text(
    id: &str,
    capacity: i32,
    machine: Machine,
    water: &FluidTank,
    left: &Tank,
    right: &Tank,
) -> String {
    format!(
        "{id}: {}/{} FE, progress {}/{}, water {}/{} mB, left {} {}/{}, right {} {}/{}",
        machine.energy,
        capacity,
        machine.progress,
        SEPARATOR_TICKS,
        water.amount,
        SEPARATOR_FLUID,
        tank_name(left),
        left.amount,
        SEPARATOR_GAS,
        tank_name(right),
        right.amount,
        SEPARATOR_GAS
    )
}

/// The `/mekanism info` report line for the rotary Condensentrator: its buffer,
/// its one-tick progress, its water tank and its one gas tank.
pub(crate) fn condensentrator_info_text(
    id: &str,
    capacity: i32,
    machine: Machine,
    water: &FluidTank,
    gas: &Tank,
    mode: CondensentratorMode,
) -> String {
    format!(
        "{id}: {}/{} FE, progress {}/{}, water {}/{} mB, gas {} {}/{}, mode {}",
        machine.energy,
        capacity,
        machine.progress,
        CONDENSENTRATOR_TICKS,
        water.amount,
        CONDENSENTRATOR_TANK,
        tank_name(gas),
        gas.amount,
        CONDENSENTRATOR_TANK,
        condensentrator_mode_name(mode)
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

/// The Chemical Crystallizer's behaviour: one operation at a time over one
/// 10,000-unit tank of a clean slurry, its own 160,000 FE buffer and one output
/// slot. The washer's consumer — the slurry is spent when the operation
/// finishes, and the crystal is an item, not a chemical.
///
/// It has no input slot: the tank is its input, as upstream's is. The crystal
/// leaves through the container's one slot, which is what a hopper pulls from.
pub(crate) struct CrystallizerPart;

#[mod_api::horizon_part]
impl BlockPart for CrystallizerPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for CrystallizerPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: draw the operation's clean slurry from a neighbour when
    /// the machine could work (through `mekanism:chemical`; the gas machines'
    /// recorded deviation — no pipes, so a machine takes what it is about to
    /// spend), then run one operation's tick over the tank, the buffer and the
    /// output slot. An idle tick that only held its progress (no energy or no
    /// output room) changes nothing and answers `Unchanged`; a reset (an empty
    /// tank, a chemical with no row, too little slurry) is a change and is
    /// written back.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let resized = event.entity.items.len() != 1;
        if resized {
            event.entity.items.resize(1, None);
        }
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut machine = read_machine(&saved, CRYSTALLIZER_CAPACITY);
        machine.progress = machine.progress.clamp(0, (CRYSTALLIZER_TICKS - 1).max(0));
        let mut tank = read_tank(&saved);
        let mut output = slot_stack(&*host, event.entity.items[CRYSTAL_SLOT].as_ref());
        let limit = stack_limit(&*host, &output);
        let pulled = crystallizer_draw(&machine, &mut tank, &output, limit, host, event.pos);
        let outcome = process_crystal_tick(
            &mut machine,
            CRYSTALLIZER_OPERATION,
            &mut tank,
            &mut output,
            limit,
            &crystallizing::recipe_for,
        );
        write_state(&mut saved, machine.energy, machine.progress);
        write_tank(&mut saved, &tank);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        set_slot(&*host, &mut event.entity.items, CRYSTAL_SLOT, &output);
        let idle = matches!(
            outcome,
            Outcome::Idle(IdleReason::NoEnergy | IdleReason::NoOutputSpace)
        );
        if idle && pulled == 0 && !resized {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// Draw the operation's clean slurry from an adjacent chemical handler through
/// [`CHEMICAL`], and answer how much the neighbour gave.
///
/// It draws only when the machine could actually complete an operation — energy
/// for a tick and room for the crystal — so a stalled crystallizer never siphons
/// a neighbour it cannot use. Two shapes, as the washer's draw has: a tank
/// already holding a slurry with a row asks for that slurry, and an empty tank
/// asks for each of the slurries **whose crystal its slot could take**
/// ([`crystallizing::slurries_that_fit`]) in table order — a slot holding one
/// material's crystals must not hide another row, and the rule is the pure
/// core's, not this loop's (the port ships no pipes, so a machine takes what it
/// is about to spend — the gas machines' recorded deviation).
fn crystallizer_draw(
    machine: &Machine,
    tank: &mut Tank,
    output: &ItemAmount,
    limit: u32,
    host: &mut dyn ServerHost,
    pos: [i32; 3],
) -> u32 {
    if machine.energy < CRYSTALLIZER_OPERATION.usage {
        return 0;
    }
    if let Some((held, _)) = tank.held() {
        let Some(recipe) = crystallizing::recipe_for(held) else {
            return 0;
        };
        if room_for(output, &crystal_of(&recipe), limit) < recipe.output_count {
            return 0;
        }
        let Some((chemical, want)) = crystallizer_pull(&recipe, tank) else {
            return 0;
        };
        let pulled = pull_chemical(host, pos, chemical, want);
        let _ = insert_chemical(tank, chemical, pulled, CRYSTALLIZER_TANK);
        return pulled;
    }
    for slurry in crystallizing::slurries_that_fit(output, limit) {
        let Some(recipe) = crystallizing::recipe_for(slurry) else {
            continue;
        };
        let pulled = pull_chemical(host, pos, slurry, recipe.amount);
        if pulled > 0 {
            let _ = insert_chemical(tank, slurry, pulled, CRYSTALLIZER_TANK);
            return pulled;
        }
    }
    0
}

/// A crystallizing row's result as the pure core's stack.
fn crystal_of(recipe: &CrystallizeRecipe) -> ItemAmount {
    ItemAmount::new(recipe.output, recipe.output_count)
}

/// The Chemical Dissolution Chamber's behaviour: one operation at a time over an
/// ore in its one slot, sulfuric acid in its own tank, and the dirty slurry it
/// makes in a second tank — over its own 160,000 FE buffer.
///
/// Upstream's acid-fill, chemical-drain and energy slots are container
/// conveniences this port does not draw (the washer's and the crystallizer's
/// recorded deviation), so the tanks are the chemical interface and `/mekanism
/// info` reports them. The chamber is the **head** of the slurry chain: what it
/// makes is what the washer beside it takes.
pub(crate) struct DissolutionPart;

#[mod_api::horizon_part]
impl BlockPart for DissolutionPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for DissolutionPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: draw the tick's acid from a neighbour when the machine
    /// could work (through `mekanism:chemical`; the gas machines' recorded
    /// deviation — no pipes, so a machine takes what it is about to spend), then
    /// run one operation's tick over the slot, both tanks and the buffer. An idle
    /// tick that only held its progress (no energy, no output room, no acid)
    /// changes nothing and answers `Unchanged`; a reset (an empty slot, an item
    /// with no row, too few items) is a change and is written back.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let single = event.entity.items.len() != 1;
        if single {
            event.entity.items.resize(1, None);
        }
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut machine = read_machine(&saved, DISSOLUTION_CAPACITY);
        machine.progress = machine.progress.clamp(0, (DISSOLUTION_TICKS - 1).max(0));
        let mut acid = read_prefixed_tank(&saved, "acid");
        let mut slurry = read_prefixed_tank(&saved, "output");
        let mut input = slot_stack(&*host, event.entity.items[DISSOLVE_SLOT].as_ref());
        let pulled = dissolution_draw(&machine, &mut acid, &slurry, &input, host, event.pos);
        let outcome = process_dissolve_tick(
            &mut machine,
            DISSOLUTION_OPERATION,
            &mut acid,
            &mut slurry,
            &mut input,
            &dissolving::recipe_for,
        );
        write_state(&mut saved, machine.energy, machine.progress);
        write_prefixed_tank(&mut saved, "acid", &acid);
        write_prefixed_tank(&mut saved, "output", &slurry);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        set_slot(&*host, &mut event.entity.items, DISSOLVE_SLOT, &input);
        let idle = matches!(
            outcome,
            Outcome::Idle(
                IdleReason::NoEnergy | IdleReason::NoOutputSpace | IdleReason::NoChemical
            )
        );
        if idle && pulled == 0 && !single {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// Draw one tick's acid from an adjacent chemical handler through [`CHEMICAL`],
/// and answer how much the neighbour gave.
///
/// It draws only when the machine could actually work — an item with a row, the
/// row's item count, room for the whole batch of slurry and energy for a tick —
/// so a stalled chamber never siphons a neighbour it cannot use (the gas
/// machines' slice-7 guard, pinned here from the start).
fn dissolution_draw(
    machine: &Machine,
    acid: &mut Tank,
    slurry: &Tank,
    input: &ItemAmount,
    host: &mut dyn ServerHost,
    pos: [i32; 3],
) -> u32 {
    if machine.energy < DISSOLUTION_OPERATION.usage || input.is_empty() {
        return 0;
    }
    let Some(recipe) = dissolving::recipe_for(&input.item) else {
        return 0;
    };
    if input.count < recipe.input.max(1) {
        return 0;
    }
    if room_in_tank(
        slurry,
        recipe.output,
        recipe.output_amount,
        DISSOLUTION_TANK,
    ) < recipe.output_amount
    {
        return 0;
    }
    let Some((chemical, want)) = dissolve_pull(&recipe, acid) else {
        return 0;
    };
    let room = DISSOLUTION_TANK.saturating_sub(acid.amount);
    let want = want.min(room);
    if want == 0 {
        return 0;
    }
    let pulled = pull_chemical(host, pos, chemical, want);
    let _ = insert_chemical(acid, chemical, pulled, DISSOLUTION_TANK);
    pulled
}

/// The Chemical Oxidizer: one sulfur slot, one gas tank and a 20,000 FE
/// operation. Its product is pushed to a neighbouring chemical handler on its
/// tick, so the infuser beside it can take SO2 without a pipe.
pub(crate) struct OxidizerPart;

#[mod_api::horizon_part]
impl BlockPart for OxidizerPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for OxidizerPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let single = event.entity.items.len() != 1;
        if single {
            event.entity.items.resize(1, None);
        }
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut machine = read_machine(&saved, OXIDIZER_CAPACITY);
        machine.progress = machine.progress.clamp(0, (OXIDIZER_TICKS - 1).max(0));
        let mut gas = read_tank(&saved);
        let mut input = slot_stack(&*host, event.entity.items[OXIDIZE_SLOT].as_ref());
        let outcome = process_oxidize_tick(
            &mut machine,
            OXIDIZER_OPERATION,
            &mut gas,
            &mut input,
            &oxidizing::recipe_for,
        );
        let pushed = push_chemical(host, event.pos, &mut gas, OXIDIZER_TANK);
        write_state(&mut saved, machine.energy, machine.progress);
        write_tank(&mut saved, &gas);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        set_slot(&*host, &mut event.entity.items, OXIDIZE_SLOT, &input);
        let idle = matches!(
            outcome,
            Outcome::Idle(IdleReason::NoEnergy | IdleReason::NoOutputSpace | IdleReason::NoInput)
        );
        if idle && pushed == 0 && !single {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// The Chemical Infuser: two input tanks and one centre output tank. A one-tick
/// machine — each tick it pulls a whole row's two gases from its neighbours and
/// spends 200 FE to make the row's output. Its centre product is pushed to a
/// neighbouring handler, so the next infuser or the dissolution chamber beside
/// it can take it through the same capability.
pub(crate) struct InfuserPart;

#[mod_api::horizon_part]
impl BlockPart for InfuserPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for InfuserPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut machine = read_machine(&saved, INFUSER_CAPACITY);
        let mut left = read_prefixed_tank(&saved, "left");
        let mut right = read_prefixed_tank(&saved, "right");
        let mut output = read_prefixed_tank(&saved, "center");
        let pulled = infuser_draw(&machine, &mut left, &mut right, &output, host, event.pos);
        let outcome = process_infuse_tick(
            &mut machine,
            INFUSER_OPERATION,
            &mut left,
            &mut right,
            &mut output,
            &infusing::recipe_for,
        );
        // An intermediate (SO3) has a consumer beside this machine, which pulls
        // it from the centre tank; only a final product (the acid) is pushed to
        // a neighbour, so an emptied SO2 tank cannot silently become a sink.
        let pushed = if output.held().is_some_and(|(id, _)| !infusing::is_input(id)) {
            push_chemical(host, event.pos, &mut output, INFUSER_TANK)
        } else {
            0
        };
        write_state(&mut saved, machine.energy, machine.progress);
        write_prefixed_tank(&mut saved, "left", &left);
        write_prefixed_tank(&mut saved, "right", &right);
        write_prefixed_tank(&mut saved, "center", &output);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        let idle = matches!(
            outcome,
            Outcome::Idle(
                IdleReason::NoEnergy | IdleReason::NoChemical | IdleReason::NoOutputSpace
            )
        );
        if idle && pulled == 0 && pushed == 0 {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// Pull one row's gases into the infuser's side tanks, but only for a row it
/// could actually finish this tick (energy and room for the product), and never
/// a partial amount that would stall the machine. How much moved.
fn infuser_draw(
    machine: &Machine,
    left: &mut Tank,
    right: &mut Tank,
    output: &Tank,
    host: &mut dyn ServerHost,
    pos: [i32; 3],
) -> u32 {
    if machine.energy < INFUSER_OPERATION.usage {
        return 0;
    }
    for row in infusing::rows() {
        let (left_chem, left_amount, right_chem, right_amount, product, amount) = *row;
        if room_in_tank(output, product, amount, INFUSER_TANK) < amount {
            continue;
        }
        if left.held().is_some_and(|(held, _)| held != left_chem)
            || right.held().is_some_and(|(held, _)| held != right_chem)
        {
            continue;
        }
        let want_left = left_amount
            .saturating_sub(left.amount)
            .min(INFUSER_TANK.saturating_sub(left.amount));
        let want_right = right_amount
            .saturating_sub(right.amount)
            .min(INFUSER_TANK.saturating_sub(right.amount));
        if want_left == 0 && want_right == 0 {
            continue;
        }
        let got_left = if want_left > 0 {
            let came = pull_chemical(host, pos, left_chem, want_left);
            let _ = insert_chemical(left, left_chem, came, INFUSER_TANK);
            came
        } else {
            0
        };
        let got_right = if want_right > 0 {
            let came = pull_chemical(host, pos, right_chem, want_right);
            let _ = insert_chemical(right, right_chem, came, INFUSER_TANK);
            came
        } else {
            0
        };
        if got_left + got_right > 0 {
            return got_left + got_right;
        }
    }
    0
}

/// Move this machine's product into a neighbouring chemical handler through
/// [`CHEMICAL`] — upstream's ejector, the smallest shape that lets the acid
/// chain run without pipes. The neighbour's own `insert` decides what it takes:
/// a tank of another chemical, or an output-only tank, refuses it. How much
/// actually left the tank.
fn push_chemical(host: &mut dyn ServerHost, pos: [i32; 3], tank: &mut Tank, capacity: u32) -> u32 {
    let Some((id, held)) = tank.held() else {
        return 0;
    };
    let Some(chemical) = Chemical::new(id) else {
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
    let want = held.min(capacity);
    let gave = host.transaction(move |host, tx| {
        let mut gave = 0u32;
        for handler in &neighbours {
            if gave >= want {
                break;
            }
            // Tanks that take input are index 0 (and index 1 on the infuser);
            // output tanks refuse an insert. Try every tank that accepts one.
            for index in 0..handler.size(host) {
                if gave >= want {
                    break;
                }
                gave = gave.saturating_add(handler.insert(host, tx, index, &chemical, want - gave));
            }
        }
        Ok::<u32, ()>(gave)
    });
    let gave = gave.unwrap_or(0);
    tank.spend(gave);
    gave
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
            // Every tank of the neighbour is asked, not just tank 0: the chamber's
            // slurry is its second tank (its first holds the acid), and a machine
            // must be able to take a product from any tank that holds it.
            for index in 0..handler.size(host) {
                if got >= want {
                    break;
                }
                if handler.resource(host, index).as_ref() != Some(&chemical) {
                    continue;
                }
                let came = handler.extract(host, tx, index, &chemical, want - got);
                got = got.saturating_add(came);
            }
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

/// The chemical capability for the Chemical Dissolution Chamber: its **two**
/// tanks, tank 0 the sulfuric acid it takes (insert only, upstream's
/// `BasicChemicalTank.inputModern`) and tank 1 the dirty slurry it makes (extract
/// only, upstream's `output`), both kept in the entity's saved compound
/// (`acid_*`/`output_*`).
///
/// The slurry at index 1 is what the washer beside it pulls: with this handler's
/// two tanks the washer asks tank 0 (the acid, which it does not take) and then
/// tank 1, so the chamber's product is reachable through the capability and not
/// just through this port's commands.
pub(crate) struct DissolutionTanks {
    side: BlockSide,
    capacity: u32,
}

impl DissolutionTanks {
    /// What the block entity here holds; two empty tanks for a stale one.
    fn held(&self, host: &mut dyn ServerHost) -> (Tank, Tank) {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return (Tank::default(), Tank::default());
        };
        let Some(saved) = entity.data(&*host) else {
            return (Tank::default(), Tank::default());
        };
        let saved = Tag::Compound(saved);
        (
            read_prefixed_tank(&saved, "acid"),
            read_prefixed_tank(&saved, "output"),
        )
    }

    /// Write both tanks back into the block entity here: whether it loaded.
    fn store(&self, host: &mut dyn ServerHost, acid: &Tank, slurry: &Tank) -> bool {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return false;
        };
        let mut saved = Tag::Compound(Compound::new());
        write_prefixed_tank(&mut saved, "acid", acid);
        write_prefixed_tank(&mut saved, "output", slurry);
        let Tag::Compound(patch) = saved else {
            return false;
        };
        entity
            .merge_data(host, patch)
            .ok()
            .is_some_and(|loaded| loaded)
    }
}

impl ResourceHandler<Chemical> for DissolutionTanks {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        2
    }

    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        let (acid, slurry) = self.held(host);
        let tank = match index {
            0 => acid,
            1 => slurry,
            _ => return None,
        };
        tank.held().and_then(|(id, _)| Chemical::new(id))
    }

    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        let (acid, slurry) = self.held(host);
        match index {
            0 => acid.amount,
            1 => slurry.amount,
            _ => 0,
        }
    }

    fn capacity(&self, _: &mut dyn ServerHost, index: usize, _: &Chemical) -> u32 {
        if index < 2 { self.capacity } else { 0 }
    }

    /// A fill goes into the **acid** tank only: the slurry tank is filled by the
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
        let (mut acid, slurry) = self.held(host);
        match insert_chemical(&mut acid, chemical.as_str(), amount, self.capacity) {
            Ok(0) => 0,
            Ok(went) if self.store(host, &acid, &slurry) => went,
            Ok(_) => 0,
            // A chemical Mekanism does not register: the handler cannot name it
            // back, so it answers 0 (`/mekanism chemical insert` names it).
            Err(_) => 0,
        }
    }

    /// A drain takes from the **slurry** tank only: the acid is automation's to
    /// fill, not to empty.
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
        let (acid, mut slurry) = self.held(host);
        let came = extract_chemical(&mut slurry, chemical.as_str(), amount);
        if came == 0 {
            return 0;
        }
        if self.store(host, &acid, &slurry) {
            came
        } else {
            0
        }
    }
}

/// The Chemical Oxidizer's one chemical tank: the gas it makes, extract only
/// (upstream's `BasicChemicalTank.output`). The infuser beside it takes from it
/// through this handler, and the saved compound is written back on each drain.
pub(crate) struct OxidizerTank {
    side: BlockSide,
    capacity: u32,
}

impl OxidizerTank {
    fn held(&self, host: &mut dyn ServerHost) -> Tank {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return Tank::default();
        };
        entity
            .data(&*host)
            .map(|saved| read_tank(&Tag::Compound(saved)))
            .unwrap_or_default()
    }

    fn store(&self, host: &mut dyn ServerHost, tank: &Tank) -> bool {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return false;
        };
        let mut saved = Tag::Compound(Compound::new());
        write_tank(&mut saved, tank);
        let Tag::Compound(patch) = saved else {
            return false;
        };
        entity
            .merge_data(host, patch)
            .ok()
            .is_some_and(|loaded| loaded)
    }
}

impl ResourceHandler<Chemical> for OxidizerTank {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        1
    }

    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        (index == 0)
            .then(|| self.held(host))
            .and_then(|tank| tank.held().and_then(|(id, _)| Chemical::new(id)))
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
        _: &mut dyn ServerHost,
        _: &Transaction,
        _: usize,
        _: &Chemical,
        _: u32,
    ) -> u32 {
        0 // an output tank takes nothing from automation
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
        let mut gas = self.held(host);
        let came = extract_chemical(&mut gas, chemical.as_str(), amount);
        if came == 0 {
            return 0;
        }
        if self.store(host, &gas) { came } else { 0 }
    }
}

/// The Chemical Infuser's three chemical tanks: index 0 and 1 the two gases it
/// takes (insert only, upstream's `inputModern`), index 2 the centre tank whose
/// product it makes (extract only, upstream's `output`). All three live in the
/// entity's saved compound under `left_*`, `right_*` and `center_*`.
pub(crate) struct InfuserTanks {
    side: BlockSide,
    capacity: u32,
}

impl InfuserTanks {
    fn held(&self, host: &mut dyn ServerHost) -> (Tank, Tank, Tank) {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return (Tank::default(), Tank::default(), Tank::default());
        };
        let Some(saved) = entity.data(&*host) else {
            return (Tank::default(), Tank::default(), Tank::default());
        };
        let saved = Tag::Compound(saved);
        (
            read_prefixed_tank(&saved, "left"),
            read_prefixed_tank(&saved, "right"),
            read_prefixed_tank(&saved, "center"),
        )
    }

    fn store(&self, host: &mut dyn ServerHost, left: &Tank, right: &Tank, output: &Tank) -> bool {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return false;
        };
        let mut saved = Tag::Compound(Compound::new());
        write_prefixed_tank(&mut saved, "left", left);
        write_prefixed_tank(&mut saved, "right", right);
        write_prefixed_tank(&mut saved, "center", output);
        let Tag::Compound(patch) = saved else {
            return false;
        };
        entity
            .merge_data(host, patch)
            .ok()
            .is_some_and(|loaded| loaded)
    }
}

impl ResourceHandler<Chemical> for InfuserTanks {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        3
    }

    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        let (left, right, output) = self.held(host);
        let tank = match index {
            0 => left,
            1 => right,
            2 => output,
            _ => return None,
        };
        tank.held().and_then(|(id, _)| Chemical::new(id))
    }

    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        let (left, right, output) = self.held(host);
        match index {
            0 => left.amount,
            1 => right.amount,
            2 => output.amount,
            _ => 0,
        }
    }

    fn capacity(&self, _: &mut dyn ServerHost, index: usize, _: &Chemical) -> u32 {
        if index < 3 { self.capacity } else { 0 }
    }

    /// A fill goes into one of the two input tanks, never the centre output.
    fn insert(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index > 1 || !infusing::is_input(chemical.as_str()) {
            return 0;
        }
        let (mut left, mut right, output) = self.held(host);
        let target = if index == 0 { &mut left } else { &mut right };
        let went =
            insert_chemical(target, chemical.as_str(), amount, self.capacity).unwrap_or_default();
        if went > 0 && self.store(host, &left, &right, &output) {
            went
        } else {
            0
        }
    }

    /// A drain takes the product from the centre tank only: the two side tanks
    /// are automation's to fill, not to empty.
    fn extract(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index != 2 {
            return 0;
        }
        let (left, right, mut output) = self.held(host);
        let came = extract_chemical(&mut output, chemical.as_str(), amount);
        if came == 0 {
            return 0;
        }
        if self.store(host, &left, &right, &output) {
            came
        } else {
            0
        }
    }
}

/// The Electrolytic Separator: one water tank and two gas output tanks, its own
/// 160,000 FE buffer and a one-tick operation. Each tick it spends 2 mB of water
/// for the row's hydrogen and oxygen, then pushes both gases to neighbours
/// through [`CHEMICAL`] (upstream's ejectors, which this port does not have).
pub(crate) struct SeparatorPart;

#[mod_api::horizon_part]
impl BlockPart for SeparatorPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for SeparatorPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mut machine = read_machine(&saved, SEPARATOR_CAPACITY);
        let mut water = read_washer_fluid(&saved);
        let (mut left, mut right) = read_separator_tanks(&saved);
        let outcome = process_separate_tick(
            &mut machine,
            SEPARATOR_OPERATION,
            &mut water,
            &mut left,
            &mut right,
            &separating::recipe_for,
        );
        let pushed = push_chemical(host, event.pos, &mut left, SEPARATOR_GAS)
            + push_chemical(host, event.pos, &mut right, SEPARATOR_GAS);
        write_state(&mut saved, machine.energy, machine.progress);
        write_washer_fluid(&mut saved, &water);
        write_separator_tanks(&mut saved, &left, &right);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        let idle = matches!(
            outcome,
            Outcome::Idle(
                IdleReason::NoEnergy
                    | IdleReason::NoFluid
                    | IdleReason::NoRecipe
                    | IdleReason::NoOutputSpace
            )
        );
        if idle && pushed == 0 {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// The separator's two gas tanks, as its saved compound holds them: `left_*`
/// the hydrogen it makes and `right_*` the oxygen.
#[must_use]
pub(crate) fn read_separator_tanks(saved: &Tag) -> (Tank, Tank) {
    (
        read_prefixed_tank(saved, "left"),
        read_prefixed_tank(saved, "right"),
    )
}

/// Write both of the separator's gas tanks into its saved compound.
pub(crate) fn write_separator_tanks(saved: &mut Tag, left: &Tank, right: &Tank) {
    write_prefixed_tank(saved, "left", left);
    write_prefixed_tank(saved, "right", right);
}

/// The separator's two chemical tanks, both output-only (upstream's
/// `BasicChemicalTank.output`): a foreign insert is refused, and a neighbour
/// takes the gas the machine has just made through `mekanism:chemical`.
pub(crate) struct SeparatorTanks {
    side: BlockSide,
    capacity: u32,
}

impl SeparatorTanks {
    fn held(&self, host: &mut dyn ServerHost) -> (Tank, Tank) {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return (Tank::default(), Tank::default());
        };
        entity
            .data(&*host)
            .map(|saved| read_separator_tanks(&Tag::Compound(saved)))
            .unwrap_or_default()
    }

    fn store(&self, host: &mut dyn ServerHost, left: &Tank, right: &Tank) -> bool {
        let Some(entity) = host.block_entity(self.side.level, self.side.pos) else {
            return false;
        };
        let mut saved = Tag::Compound(Compound::new());
        write_separator_tanks(&mut saved, left, right);
        let Tag::Compound(patch) = saved else {
            return false;
        };
        entity
            .merge_data(host, patch)
            .ok()
            .is_some_and(|loaded| loaded)
    }
}

impl ResourceHandler<Chemical> for SeparatorTanks {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        2
    }

    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        let (left, right) = self.held(host);
        let tank = match index {
            0 => left,
            1 => right,
            _ => return None,
        };
        tank.held().and_then(|(id, _)| Chemical::new(id))
    }

    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        let (left, right) = self.held(host);
        match index {
            0 => left.amount,
            1 => right.amount,
            _ => 0,
        }
    }

    fn capacity(&self, _: &mut dyn ServerHost, index: usize, _: &Chemical) -> u32 {
        if index < 2 { self.capacity } else { 0 }
    }

    fn insert(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        _: usize,
        _: &Chemical,
        _: u32,
    ) -> u32 {
        0 // both output tanks take nothing from automation
    }

    fn extract(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        let (mut left, mut right) = self.held(host);
        let target = match index {
            0 => &mut left,
            1 => &mut right,
            _ => return 0,
        };
        let came = extract_chemical(target, chemical.as_str(), amount);
        if came > 0 && self.store(host, &left, &right) {
            came
        } else {
            0
        }
    }
}

/// The rotary Condensentrator: one water tank and one gas tank, its own
/// 20,000 FE buffer and a one-tick operation. In the default mode it spends 1 mB
/// of water for one unit of water vapour, then pushes that gas to neighbours
/// through [`CHEMICAL`] (upstream's ejectors, which this port does not have).
pub(crate) struct CondensentratorPart;

#[mod_api::horizon_part]
impl BlockPart for CondensentratorPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for CondensentratorPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let mut saved = Tag::Compound(event.entity.nbt.clone());
        let mode = read_condensentrator_mode(&saved);
        let mut machine = read_machine(&saved, CONDENSENTRATOR_CAPACITY);
        let mut water = read_washer_fluid(&saved);
        let mut gas = read_tank(&saved);
        let outcome = process_condensentrate_tick(
            &mut machine,
            CONDENSENTRATOR_OPERATION,
            &mut water,
            &mut gas,
            mode,
            &condensentrating::recipe_for_fluid,
            &condensentrating::recipe_for_chemical,
        );
        // Upstream ejects the *gas* only in fluid → gas mode. In gas → fluid mode
        // the product is water and it stays in the machine's fluid tank, which
        // `/mekanism info` reports and `horizon:fluids` can drain.
        let pushed = if mode == CondensentratorMode::Decondensentrating {
            push_chemical(host, event.pos, &mut gas, CONDENSENTRATOR_TANK)
        } else {
            0
        };
        write_state(&mut saved, machine.energy, machine.progress);
        write_washer_fluid(&mut saved, &water);
        write_tank(&mut saved, &gas);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        let idle = matches!(
            outcome,
            Outcome::Idle(
                IdleReason::NoEnergy
                    | IdleReason::NoChemical
                    | IdleReason::NoFluid
                    | IdleReason::NoRecipe
                    | IdleReason::NoOutputSpace
            )
        );
        if idle && pushed == 0 {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// The condensentrator's one gas tank, an output (upstream's
/// `BasicChemicalTank` in the default fluid → gas mode): a foreign insert is
/// refused, and a neighbour takes the vapour the machine has just made through
/// `mekanism:chemical`.
pub(crate) struct CondensentratorTanks {
    side: BlockSide,
    capacity: u32,
}

impl CondensentratorTanks {
    fn held(&self, host: &mut dyn ServerHost) -> Tank {
        host.block_entity(self.side.level, self.side.pos)
            .and_then(|entity| entity.data(&*host))
            .map(|saved| read_tank(&Tag::Compound(saved)))
            .unwrap_or_default()
    }

    /// The direction saved on the block entity the capability answers for.
    fn mode(&self, host: &mut dyn ServerHost) -> CondensentratorMode {
        host.block_entity(self.side.level, self.side.pos)
            .and_then(|entity| entity.data(&*host))
            .map(|saved| read_condensentrator_mode(&Tag::Compound(saved)))
            .unwrap_or(CondensentratorMode::Decondensentrating)
    }

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

impl ResourceHandler<Chemical> for CondensentratorTanks {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        1
    }

    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        if index != 0 {
            return None;
        }
        self.held(host).held().and_then(|(id, _)| Chemical::new(id))
    }

    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        if index != 0 {
            0
        } else {
            self.held(host).amount
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
        if index != 0 || !may_insert_condensentrator_gas(self.mode(host), chemical.as_str()) {
            return 0; // the gas tank is an output in fluid → gas mode
        }
        let mut tank = self.held(host);
        let went = insert_chemical(&mut tank, chemical.as_str(), amount, self.capacity)
            .unwrap_or_default();
        if went > 0 && self.store(host, &tank) {
            went
        } else {
            0
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
        if index != 0 || !may_extract_condensentrator_gas(self.mode(host)) {
            return 0;
        }
        let mut tank = self.held(host);
        let came = extract_chemical(&mut tank, chemical.as_str(), amount);
        if came > 0 && self.store(host, &tank) {
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

/// The chemical capability for the Chemical Crystallizer's one tank: a clean
/// slurry with a row and nothing else (upstream's `containsRecipe` predicate),
/// insert-only, kept in the entity's saved compound (`chemical` + `amount`).
///
/// A chemical with no row — a dirty slurry, a gas, water — is refused by name:
/// the insert answers 0 and the tank is unchanged, so `/mekanism chemical insert`
/// reads back what really happened.
pub(crate) struct CrystallizerTank {
    side: BlockSide,
    capacity: u32,
}

impl CrystallizerTank {
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

impl ResourceHandler<Chemical> for CrystallizerTank {
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

    /// A fill goes in only when the tank's own predicate accepts the chemical: a
    /// clean slurry the jar has a crystallizing row for.
    fn insert(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        if index != 0 || !crystallizing::accepts(chemical.as_str()) {
            return 0;
        }
        let mut tank = self.held(host);
        match insert_chemical(&mut tank, chemical.as_str(), amount, self.capacity) {
            Ok(0) => 0,
            Ok(went) if self.store(host, &tank) => went,
            Ok(_) => 0,
            Err(_) => 0,
        }
    }

    /// The machine's tank is insert-only (upstream's `BasicChemicalTank.inputModern`):
    /// a foreign drain takes nothing.
    fn extract(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        _: usize,
        _: &Chemical,
        _: u32,
    ) -> u32 {
        0
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
    if is_crystallizer(&id) {
        let output = slot_stack(
            &*host,
            entity
                .item_slot(&*host, CRYSTAL_SLOT as u32)
                .ok()
                .flatten()
                .as_ref(),
        );
        return Some(crystallizer_info_text(
            &id,
            capacity,
            read_machine(&saved, capacity),
            &read_tank(&saved),
            &output,
        ));
    }
    if is_dissolution(&id) {
        let input = slot_stack(
            &*host,
            entity
                .item_slot(&*host, DISSOLVE_SLOT as u32)
                .ok()
                .flatten()
                .as_ref(),
        );
        return Some(dissolution_info_text(
            &id,
            capacity,
            read_machine(&saved, capacity),
            &read_prefixed_tank(&saved, "acid"),
            &read_prefixed_tank(&saved, "output"),
            &input,
        ));
    }
    if is_oxidizer(&id) {
        let input = slot_stack(
            &*host,
            entity
                .item_slot(&*host, OXIDIZE_SLOT as u32)
                .ok()
                .flatten()
                .as_ref(),
        );
        return Some(oxidizer_info_text(
            &id,
            capacity,
            read_machine(&saved, capacity),
            &input,
            &read_tank(&saved),
        ));
    }
    if is_infuser(&id) {
        return Some(infuser_info_text(
            &id,
            capacity,
            read_machine(&saved, capacity),
            &read_prefixed_tank(&saved, "left"),
            &read_prefixed_tank(&saved, "right"),
            &read_prefixed_tank(&saved, "center"),
        ));
    }
    if is_separator(&id) {
        let (left, right) = read_separator_tanks(&saved);
        return Some(separator_info_text(
            &id,
            capacity,
            read_machine(&saved, capacity),
            &read_washer_fluid(&saved),
            &left,
            &right,
        ));
    }
    if is_condensentrator(&id) {
        return Some(condensentrator_info_text(
            &id,
            capacity,
            read_machine(&saved, capacity),
            &read_washer_fluid(&saved),
            &read_tank(&saved),
            read_condensentrator_mode(&saved),
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

/// `/mekanism mode <pos>`: flip a rotary Condensentrator's direction, the
/// harness's stand-in for upstream's Configurator right-click (`nextMode()`).
/// It touches only this mod's block and writes the saved `mode` byte through
/// `merge_data`, which also marks the chunk changed (`setChanged()` upstream).
pub(crate) fn toggle_mode(host: &mut dyn ServerHost, pos: [i32; 3]) -> String {
    let Some(level) = host.level(host.dimension()) else {
        return "No rotary condensentrator there.".to_owned();
    };
    let Some(id) = level.block(&*host, pos).map(|block| block.id(&*host)) else {
        return "No rotary condensentrator there.".to_owned();
    };
    if !is_condensentrator(&id) {
        return "No rotary condensentrator there.".to_owned();
    }
    let Some(entity) = host.block_entity(level, pos) else {
        return "No rotary condensentrator there.".to_owned();
    };
    let saved = Tag::Compound(entity.data(&*host).unwrap_or_default());
    let next = next_condensentrator_mode(read_condensentrator_mode(&saved));
    if entity
        .merge_data(host, condensentrator_mode_patch(next))
        .is_err()
    {
        return "No rotary condensentrator there.".to_owned();
    }
    format!("Mode set to {}.", condensentrator_mode_name(next))
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
        assert_eq!(machine_capacity(WASHER), Some(WASHER_CAPACITY));
        assert_eq!(machine_capacity(CRYSTALLIZER), Some(CRYSTALLIZER_CAPACITY));
        assert_eq!(machine_capacity(DISSOLUTION), Some(DISSOLUTION_CAPACITY));
        assert_eq!(machine_capacity(CUBE), Some(CUBE_CAPACITY));
        assert_eq!(machine_capacity(CABLE), Some(CABLE_CAPACITY));
        assert_eq!(machine_capacity("minecraft:chest"), None);
        assert_eq!(machine_ticks(SMELTER), Some(200));
        assert_eq!(machine_ticks(CHAMBER), Some(200));
        assert_eq!(machine_ticks(CRUSHER), Some(200));
        assert_eq!(machine_ticks(INJECTION), Some(200));
        assert_eq!(machine_ticks(PURIFICATION), Some(200));
        assert_eq!(machine_ticks(WASHER), Some(200));
        assert_eq!(machine_ticks(CRYSTALLIZER), Some(200));
        assert_eq!(
            machine_ticks(DISSOLUTION),
            Some(100),
            "the dissolution chamber's own BASE_TICKS_REQUIRED = 5 x 20"
        );
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

#[cfg(test)]
mod crystallizer_support_tests {
    use super::*;

    /// The crystallizer is the shared machine shape with the jar's own numbers:
    /// a 400 FE/t usage, a 160,000 FE insert-only buffer, a 200-tick operation
    /// and one 10,000-unit tank.
    #[test]
    fn the_crystallizer_is_the_jar_s_400_fe_a_tick_and_its_160_000_buffer() {
        assert_eq!(CRYSTALLIZER, "mekanism:chemical_crystallizer");
        assert_eq!(CRYSTALLIZER_TYPE, "chemical_crystallizer");
        assert_eq!(machine_capacity(CRYSTALLIZER), Some(160_000));
        assert_eq!(machine_ticks(CRYSTALLIZER), Some(200));
        assert_eq!(CRYSTALLIZER_OPERATION.usage, 400, "UsageConfig");
        assert_eq!(CRYSTALLIZER_TANK, 10_000, "MAX_CHEMICAL");
        let buffer = machine_battery(CRYSTALLIZER_CAPACITY);
        assert_eq!(
            (buffer.max_insert, buffer.max_extract),
            (CRYSTALLIZER_CAPACITY, 0),
            "a machine takes energy and gives none"
        );
        assert_eq!(
            machine_capacity(WASHER),
            Some(80_000),
            "and the washer's own buffer is untouched by this slice"
        );
    }

    /// Its container is one output slot and no input: its input is the tank, so
    /// automation may pull the crystal and cannot push an item into nothing.
    #[test]
    fn the_crystallizer_declares_one_output_slot_and_no_input() {
        let container = crystallizer_container();
        assert_eq!(container.slots, 1);
        assert_eq!(container.outputs, vec![CRYSTAL_SLOT]);
        assert!(
            container.inputs.is_empty(),
            "the machine's input is a chemical, so there is no input slot"
        );
        assert_eq!(
            container.faces,
            std::array::from_fn(|_| vec![CRYSTAL_SLOT]),
            "every face reaches the output: a hopper under it takes the crystal"
        );
        assert_eq!(container.menu, None, "one slot is not a menu");
    }

    /// The `/mekanism info` line the in-game leg asserts, with the jar's own
    /// capacity and tick count.
    #[test]
    fn the_crystallizer_info_line_is_the_one_the_e2e_asserts() {
        assert_eq!(
            crystallizer_info_text(
                CRYSTALLIZER,
                CRYSTALLIZER_CAPACITY,
                Machine {
                    energy: 80_000,
                    progress: 0
                },
                &Tank::new("mekanism:clean_iron", 200),
                &ItemAmount::new("mekanism:crystal_iron", 5),
            ),
            "mekanism:chemical_crystallizer: 80000/160000 FE, progress 0/200, \
             input mekanism:clean_iron x200, output mekanism:crystal_iron x5"
        );
        assert_eq!(
            crystallizer_info_text(
                CRYSTALLIZER,
                CRYSTALLIZER_CAPACITY,
                Machine::default(),
                &Tank::default(),
                &ItemAmount::default(),
            ),
            "mekanism:chemical_crystallizer: 0/160000 FE, progress 0/200, \
             input empty x0, output empty x0"
        );
    }

    /// The tank keeps its contents in the entity's saved compound and the
    /// handler's predicate is the table's: a dirty slurry is refused, a clean one
    /// is taken, and only tank 0 exists.
    #[test]
    fn the_crystallizer_tank_takes_a_clean_slurry_and_refuses_the_rest() {
        assert!(is_crystallizer(CRYSTALLIZER));
        assert!(!is_crystallizer(WASHER));
        assert!(crystallizing::accepts("mekanism:clean_iron"));
        assert!(!crystallizing::accepts("mekanism:dirty_iron"));
        let mut saved = Tag::Compound(Compound::new());
        write_tank(&mut saved, &Tank::new("mekanism:clean_iron", 200));
        assert_eq!(read_tank(&saved), Tank::new("mekanism:clean_iron", 200));
        assert_eq!(
            saved.get("chemical"),
            Some(&Tag::String("mekanism:clean_iron".into())),
            "the saved chemical is the id `/data get block` reads"
        );
    }

    /// Slice 15: the Chemical Dissolution Chamber is the jar's own machine — 400
    /// FE/t, a 160,000 FE buffer, 100 ticks an operation and two 10,000-unit
    /// tanks — with one input slot, and its info line reports the ore and both
    /// tanks.
    #[test]
    fn the_dissolution_chamber_is_the_jar_s_numbers_with_one_input_slot() {
        assert_eq!(DISSOLUTION, "mekanism:chemical_dissolution_chamber");
        assert_eq!(DISSOLUTION_TYPE, "chemical_dissolution_chamber");
        assert_eq!(machine_capacity(DISSOLUTION), Some(160_000));
        assert_eq!(machine_ticks(DISSOLUTION), Some(100));
        assert_eq!(DISSOLUTION_OPERATION.usage, 400, "UsageConfig");
        assert_eq!(DISSOLUTION_OPERATION.ticks, 100, "BASE_TICKS_REQUIRED");
        assert_eq!(DISSOLUTION_TANK, 10_000, "MAX_CHEMICAL");
        let buffer = machine_battery(DISSOLUTION_CAPACITY);
        assert_eq!(
            (buffer.max_insert, buffer.max_extract),
            (DISSOLUTION_CAPACITY, 0),
            "a machine takes energy and gives none"
        );
        let container = dissolution_container();
        assert_eq!(container.slots, 1);
        assert_eq!(container.inputs, vec![DISSOLVE_SLOT]);
        assert!(
            container.outputs.is_empty(),
            "its output is a chemical, so there is no output slot"
        );
        assert_eq!(container.menu, None, "one slot is not a menu");
        assert!(is_dissolution(DISSOLUTION));
        assert!(!is_dissolution(CRYSTALLIZER));
    }

    /// The chamber's info line the in-game leg asserts: the ore, the acid in
    /// tank 0 and the slurry in tank 1, each with its own 10,000 capacity.
    #[test]
    fn the_dissolution_info_line_is_the_one_the_e2e_asserts() {
        assert_eq!(
            dissolution_info_text(
                DISSOLUTION,
                DISSOLUTION_CAPACITY,
                Machine {
                    energy: 120_000,
                    progress: 0
                },
                &Tank::new("mekanism:sulfuric_acid", 900),
                &Tank::new("mekanism:dirty_iron", 1_000),
                &ItemAmount::default(),
            ),
            "mekanism:chemical_dissolution_chamber: 120000/160000 FE, progress 0/100, \
             input empty x0, acid mekanism:sulfuric_acid 900/10000, \
             output mekanism:dirty_iron 1000/10000"
        );
        assert_eq!(
            dissolution_info_text(
                DISSOLUTION,
                DISSOLUTION_CAPACITY,
                Machine::default(),
                &Tank::default(),
                &Tank::default(),
                &ItemAmount::new("minecraft:iron_ore", 1),
            ),
            "mekanism:chemical_dissolution_chamber: 0/160000 FE, progress 0/100, \
             input minecraft:iron_ore x1, acid empty 0/10000, output empty 0/10000"
        );
    }

    /// The chamber's two tanks save and read back under their own prefixes, so a
    /// tick's acid and slurry survive a save and `/data get block` can read them.
    #[test]
    fn the_dissolution_tanks_save_and_read_back() {
        let mut saved = Tag::Compound(Compound::new());
        write_prefixed_tank(
            &mut saved,
            "acid",
            &Tank::new("mekanism:sulfuric_acid", 900),
        );
        write_prefixed_tank(
            &mut saved,
            "output",
            &Tank::new("mekanism:dirty_iron", 1_000),
        );
        assert_eq!(
            read_prefixed_tank(&saved, "acid"),
            Tank::new("mekanism:sulfuric_acid", 900)
        );
        assert_eq!(
            read_prefixed_tank(&saved, "output"),
            Tank::new("mekanism:dirty_iron", 1_000)
        );
        assert_eq!(
            saved.get("acid_chemical"),
            Some(&Tag::String("mekanism:sulfuric_acid".into()))
        );
        assert_eq!(
            saved.get("output_amount"),
            Some(&Tag::Int(1_000)),
            "the saved amount is the int `/data get block` reads"
        );
    }
}

#[cfg(test)]
mod separator_glue_tests {
    use super::*;

    /// The separator declares exactly one 24,000 mB water tank a fill from any
    /// face may reach, as upstream's `BasicFluidTank.input(MAX_FLUID, …)`.
    #[test]
    fn the_separator_declares_one_24_000_mb_water_tank_a_fill_may_reach() {
        let tanks = separator_fluids();
        assert_eq!(tanks.tanks, vec![24_000]);
        assert!(tanks.faces.iter().all(|face| face == &vec![0]));
        assert_eq!(tanks.inputs, vec![0]);
        assert!(tanks.outputs.is_empty());
    }

    /// Its two gas tanks are saved under the same prefixed keys the neighbour
    /// handler reads, so a tank written by the tick and one written by a
    /// capability call are the same state.
    #[test]
    fn the_separator_s_two_gas_tanks_save_and_read_back() {
        let mut saved = Tag::Compound(Compound::new());
        write_separator_tanks(
            &mut saved,
            &Tank::new("mekanism:hydrogen", 2),
            &Tank::new("mekanism:oxygen", 1),
        );
        assert_eq!(
            read_separator_tanks(&saved),
            (
                Tank::new("mekanism:hydrogen", 2),
                Tank::new("mekanism:oxygen", 1)
            )
        );
        assert_eq!(
            saved.get("left_amount"),
            Some(&Tag::Int(2)),
            "the saved amount is the int `/data get block` reads"
        );
    }

    /// The exact line the in-game leg asserts: the FE, the one-tick progress,
    /// the water and both gases, named by what they hold.
    #[test]
    fn the_separator_info_line_is_the_one_the_e2e_asserts() {
        assert_eq!(
            separator_info_text(
                "mekanism:electrolytic_separator",
                160_000,
                Machine {
                    energy: 159_600,
                    progress: 0
                },
                &FluidTank::new("minecraft:water", 998),
                &Tank::new("mekanism:hydrogen", 2),
                &Tank::new("mekanism:oxygen", 1),
            ),
            "mekanism:electrolytic_separator: 159600/160000 FE, progress 0/1, \
             water 998/24000 mB, left mekanism:hydrogen 2/2400, \
             right mekanism:oxygen 1/2400"
        );
    }
}

#[cfg(test)]
mod condensentrator_glue_tests {
    use super::*;

    /// One 10,000 mB tank, fillable from every face. In gas → fluid mode it is
    /// also drainable: upstream's fluid tank is an output when `mode = false`,
    /// and that is the fluid half of the reverse direction's output path.
    #[test]
    fn the_condensentrator_declares_one_10_000_mb_water_tank_a_fill_may_reach() {
        let tanks = condensentrator_fluids();
        assert_eq!(tanks.tanks, vec![10_000]);
        assert!(tanks.faces.iter().all(|face| face == &vec![0]));
        assert_eq!(tanks.inputs, vec![0]);
        assert_eq!(tanks.outputs, vec![0]);
        assert_eq!(
            machine_capacity(CONDENSENTRATOR),
            Some(CONDENSENTRATOR_CAPACITY)
        );
        assert_eq!(machine_ticks(CONDENSENTRATOR), Some(1));
    }

    /// Upstream saves the direction as the boolean `mode`
    /// (`SerializationConstants.MODE`). The patch this port writes must be the
    /// same key and value `/data get block` reads, and a fresh compound must
    /// still mean fluid → gas so the 14 earlier e2e logs stay green.
    #[test]
    fn the_mode_saved_boolean_round_trips_upstream_s_true_and_false() {
        let fresh = Tag::Compound(Compound::new());
        assert_eq!(
            read_condensentrator_mode(&fresh),
            CondensentratorMode::Decondensentrating,
            "a fresh block entity keeps the port's fluid -> gas default"
        );
        for (mode, byte) in [
            (CondensentratorMode::Decondensentrating, 1i8),
            (CondensentratorMode::Condensentrating, 0i8),
        ] {
            let saved = Tag::Compound(condensentrator_mode_patch(mode));
            assert_eq!(saved.get("mode"), Some(&Tag::Byte(byte)));
            assert_eq!(read_condensentrator_mode(&saved), mode);
        }
    }

    /// `nextMode()` flips the boolean and calls `setChanged()`; the pure half of
    /// that is a two-way toggle, nothing a third state could slip into.
    #[test]
    fn a_mode_toggle_flips_between_the_two_directions() {
        assert_eq!(
            next_condensentrator_mode(CondensentratorMode::Decondensentrating),
            CondensentratorMode::Condensentrating
        );
        assert_eq!(
            next_condensentrator_mode(CondensentratorMode::Condensentrating),
            CondensentratorMode::Decondensentrating
        );
    }

    /// The gas tank's gates, read from `TileEntityRotaryCondensentrator` at
    /// `a00109e4`: a foreign insert is refused while the gas is an output
    /// (fluid → gas) and accepted for a rowed chemical in gas → fluid mode;
    /// external extraction is the mirror image.
    #[test]
    fn the_gas_handler_gates_follow_upstream_s_mode_boolean() {
        let vapour = "mekanism:water_vapor";
        assert!(!may_insert_condensentrator_gas(
            CondensentratorMode::Decondensentrating,
            vapour
        ));
        assert!(may_insert_condensentrator_gas(
            CondensentratorMode::Condensentrating,
            vapour
        ));
        assert!(!may_insert_condensentrator_gas(
            CondensentratorMode::Condensentrating,
            "mekanism:unobtainium"
        ));
        assert!(may_extract_condensentrator_gas(
            CondensentratorMode::Decondensentrating
        ));
        assert!(!may_extract_condensentrator_gas(
            CondensentratorMode::Condensentrating
        ));
    }

    /// Its gas tank is saved under the same keys the neighbour handler reads,
    /// so a tick's write and a capability call see the same chemical and amount.
    #[test]
    fn the_condensentrator_s_gas_tank_saves_and_reads_back() {
        let mut saved = Tag::Compound(Compound::new());
        write_tank(&mut saved, &Tank::new("mekanism:water_vapor", 1));
        assert_eq!(read_tank(&saved), Tank::new("mekanism:water_vapor", 1));
        assert_eq!(
            saved.get("amount"),
            Some(&Tag::Int(1)),
            "the saved amount is the int `/data get block` reads"
        );
    }

    /// The exact line the in-game leg asserts: the FE, the one-tick progress,
    /// the water and the gas tank's parsed id and amount, and the mode the
    /// `/mekanism mode` toggle saved.
    #[test]
    fn the_condensentrator_info_line_is_the_one_the_e2e_asserts() {
        assert_eq!(
            condensentrator_info_text(
                "mekanism:rotary_condensentrator",
                20_000,
                Machine {
                    energy: 19_950,
                    progress: 0
                },
                &FluidTank::new("minecraft:water", 999),
                &Tank::new("mekanism:water_vapor", 1),
                CondensentratorMode::Condensentrating,
            ),
            "mekanism:rotary_condensentrator: 19950/20000 FE, progress 0/1, \
             water 999/10000 mB, gas mekanism:water_vapor 1/10000, mode condensentrating"
        );
    }
}
