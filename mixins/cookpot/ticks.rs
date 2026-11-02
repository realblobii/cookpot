// Machines tick every server tick.
fn cookpot_ticks(game: &crate::game::Game<'_>, block: &str, ticks: &mut bool) {
    if cookpot::machine_for(block, || cookpot::FILE_TYPES.iter().flat_map(|t| game.native_recipes_of_type(t).iter().map(|r| (r.id.clone(), (*r.json).clone()))).collect()).is_some() {
        *ticks = true;
    }
}
