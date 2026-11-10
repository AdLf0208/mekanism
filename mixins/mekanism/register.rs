// Declare `/mekanism info|energy|progress`.
fn observe_register(commands: &mut Vec<NativeCommand>) {
    crate::game::mekanism_state::on_register_commands(commands);
}
