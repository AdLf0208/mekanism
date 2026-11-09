// The smelter and the cable tick.
fn observe_block_entity_ticks(block: &str, ticks: &mut bool) {
    crate::game::mekanism_state::on_block_entity_ticks(block, ticks);
}
