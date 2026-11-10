// The three blocks get a block entity.
fn observe_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::mekanism_state::on_block_entity(block, entity);
}
