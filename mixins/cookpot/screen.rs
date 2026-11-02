// A machine's screen: slot positions and size from its definition.
fn cookpot_screen(game: &crate::game::Game<'_>, block: &str, screen: &mut Option<NativeScreen>) {
    if screen.is_some() {
        return;
    }
    let Some(m) = cookpot::machine_for(block, || cookpot::FILE_TYPES.iter().flat_map(|t| game.native_recipes_of_type(t).iter().map(|r| (r.id.clone(), (*r.json).clone()))).collect()) else {
        return;
    };
    *screen = Some(NativeScreen {
        width: m.width,
        height: m.height,
        slots: m.slots.iter().map(|s| Some(s.pos)).collect(),
        inventory_y: m.inventory_y,
        inventory_x: 8.0,
        own_labels: false,
    });
}
