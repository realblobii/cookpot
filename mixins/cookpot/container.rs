// The slots, hopper faces and menu of a machine's block entity.
fn cookpot_container(game: &crate::game::Game<'_>, block: &str, container: &mut Option<ModContainer>) {
    let Some(m) = cookpot::machine_for(block, || game.native_recipes_of_type(cookpot::MACHINE_TYPE).iter().map(|r| (r.id.clone(), (*r.json).clone())).collect()) else {
        return;
    };
    let all: Vec<usize> = (0..m.slots.len()).collect();
    *container = Some(ModContainer {
        slots: m.slots.len(),
        faces: std::array::from_fn(|_| all.clone()),
        inputs: m.slot_indices(cookpot::Role::accepts_items),
        outputs: m.slot_indices(cookpot::Role::is_output),
        menu: Some(m.menu.clone()),
    });
}
