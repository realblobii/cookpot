// Server tick of a machine: progress lives in the saved compound, and is mirrored into the menu's data slots.
fn cookpot_tick(game: &mut crate::game::Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.id != cookpot::ENTITY_ID {
        return;
    }
    let block = game.native_block(pos);
    let Some(m) = cookpot::machine_for(&block, || game.native_recipes_of_type(cookpot::MACHINE_TYPE).iter().map(|r| (r.id.clone(), (*r.json).clone())).collect()) else {
        return;
    };
    let key = horizon_world::nbt::JavaString::from("progress");
    let progress = match entity.nbt.get(&key) {
        Some(horizon_world::nbt::Tag::Int(n)) => *n,
        _ => 0,
    };
    let active = entity.items.first().is_some_and(|s| s.is_some());
    let next = cookpot::advance(progress, active, cookpot::TEST_TOTAL);
    entity.nbt.insert(key, horizon_world::nbt::Tag::Int(next));
    entity.data = cookpot::data_slots(&m, next, cookpot::TEST_TOTAL);
}
