// One tick of a smelter or a cable.
fn observe_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::mekanism_state::on_block_entity_tick(game, pos, entity);
}
