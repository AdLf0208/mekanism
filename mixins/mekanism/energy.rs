// Each block's energy buffer, kept in its saved compound as `energy`.
fn observe_block_entity_energy(
    block: &str,
    energy: &mut Option<horizon_client::containers::ModEnergyBuffer>,
) {
    crate::game::mekanism_state::on_block_entity_energy(block, energy);
}
