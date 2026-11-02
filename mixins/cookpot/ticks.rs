// Machines tick every server tick.
fn cookpot_ticks(game: &crate::game::Game<'_>, block: &str, ticks: &mut bool) {
    if cookpot::machine_for(block, || game.native_recipes_of_type(cookpot::MACHINE_TYPE).iter().map(|r| (r.id.clone(), (*r.json).clone())).collect()).is_some() {
        *ticks = true;
    }
}
