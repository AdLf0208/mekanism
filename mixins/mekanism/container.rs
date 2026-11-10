// The smelter's slots and its furnace menu.
fn observe_block_entity_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::mekanism_state::on_block_entity_container(block, container);
}
