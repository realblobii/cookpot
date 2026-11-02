// A block named by a loaded machine definition gets Cookpot's block entity.
fn cookpot_entity(game: &mut crate::game::Game<'_>, block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && cookpot::machine_for(block, || cookpot::FILE_TYPES.iter().flat_map(|t| game.native_recipes_of_type(t).iter().map(|r| (r.id.clone(), (*r.json).clone()))).collect()).is_some() {
        *entity = Some((cookpot::ENTITY_ID.to_owned(), Compound::new()));
    }
}
