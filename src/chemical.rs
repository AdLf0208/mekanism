//! The chemical capability this port publishes: `mekanism:chemical`.
//!
//! Mekanism's identity is its chemicals, and at the campaign pin no capability
//! answers one — the game publishes energy, fluids and items, and the pack's own
//! integrations stop without a chemical handler
//! (`refinedstorage_mekanism_integration`, `mekanisticrouters`). This module is
//! the smallest honest provider of that surface: the resource type, the block
//! capability's key and the item capability's key. The handlers behind them (a
//! block entity's one tank and the gauge dropper's) are the glue in
//! [`crate::support`]; the rules are [`crate::runtime`]'s.
//!
//! A key is a **name plus a handler type**: `PartRegistrar::capability` refuses
//! one name answered with two handler types, and a lookup of a mod's own
//! capability downcasts to the handler trait object the asker names. So the
//! contract another mod's crate uses is [`CHEMICAL`] (or [`ITEM_CHEMICAL`]) and
//! [`Chemical`] — both public here, in the shipped crate, not a test fixture.
//! This is the `[api]` shape the SDK documents: a crate that publishes its
//! capability publishes the key and the resource type with it.
use mod_api::{CapabilityKey, ItemCapabilityKey, ResourceHandler};

use std::fmt;

/// One chemical: its registry id (`mekanism:hydrogen`) and nothing else.
///
/// The shape is checked here; whether the id is a chemical Mekanism registers is
/// [`crate::runtime::known_chemical`]'s question, asked wherever a chemical goes
/// in (so an id saved by a later version still reads back instead of being
/// dropped).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chemical(String);

impl Chemical {
    /// The chemical `id`, `None` when it is not a `namespace:path` id.
    #[must_use]
    pub fn new(id: &str) -> Option<Self> {
        let (namespace, path) = id.split_once(':')?;
        let shaped = !namespace.is_empty()
            && !path.is_empty()
            && id.bytes().all(|b| {
                b.is_ascii_alphanumeric() || matches!(b, b':' | b'_' | b'-' | b'.' | b'/')
            });
        shaped.then(|| Self(id.to_owned()))
    }

    /// The registry id (`mekanism:hydrogen`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Chemical {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One block entity's chemical tanks, as NeoForge 26.3's
/// `Capabilities.Chemical.BLOCK`: a tank numbered from 0 as one side reaches it,
/// with a capacity, and an amount in and out per call.
pub const CHEMICAL: CapabilityKey<dyn ResourceHandler<Chemical>> =
    CapabilityKey::new("mekanism:chemical");

/// The chemical an item holds, as `Capabilities.Chemical.ITEM`: the handler for
/// the stack at one place, whose writes rewrite that stack.
///
/// The same chemical as [`CHEMICAL`] on the item side, as `horizon:fluids` and
/// `horizon:item_fluids` are the two sides of one fluid capability; the two are
/// separate tables in the SDK, so they are two keys with two names.
pub const ITEM_CHEMICAL: ItemCapabilityKey<dyn ResourceHandler<Chemical>> =
    ItemCapabilityKey::new("mekanism:item_chemical");

/// The component an item's chemical lives in: `{id, amount}`, the shape
/// `ChemicalStack`'s codec writes.
pub const ITEM_COMPONENT: &str = "mekanism:chemical";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{CHEMICALS, known_chemical};

    #[test]
    fn a_chemical_is_a_namespaced_id() {
        assert_eq!(
            Chemical::new("mekanism:hydrogen").map(|c| c.as_str().to_owned()),
            Some("mekanism:hydrogen".to_owned())
        );
        assert!(Chemical::new("hydrogen").is_none(), "no namespace");
        assert!(Chemical::new("mekanism:").is_none(), "no path");
        assert!(Chemical::new(":hydrogen").is_none(), "empty namespace");
        assert!(Chemical::new("mekanism:hydro gen").is_none(), "a space");
        assert_eq!(
            Chemical::new("mekanism:hydrogen").map(|c| c.to_string()),
            Some("mekanism:hydrogen".to_owned()),
            "Display is the id"
        );
    }

    #[test]
    fn the_keys_name_one_capability_on_two_sides() {
        assert_eq!(CHEMICAL.name, "mekanism:chemical");
        assert_eq!(ITEM_CHEMICAL.name, "mekanism:item_chemical");
        assert_eq!(ITEM_COMPONENT, "mekanism:chemical");
    }

    #[test]
    fn every_chemical_the_port_knows_is_a_chemical_value() {
        for id in CHEMICALS {
            assert_eq!(Chemical::new(id).map(|c| c.0), Some(id.to_owned()));
            assert_eq!(known_chemical(id), Some(id));
        }
        assert!(Chemical::new(crate::runtime::EMPTY_CHEMICAL).is_some());
        assert_eq!(
            known_chemical(crate::runtime::EMPTY_CHEMICAL),
            None,
            "the empty chemical is not one a tank takes"
        );
    }
}
