// Server tick of a machine: finds its recipe, heat and fuel, advances the process, and keeps its state in
// the saved compound (mirrored into the menu's data slots).
fn cookpot_tick(game: &mut crate::game::Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.id != cookpot::ENTITY_ID {
        return;
    }
    let block = game.native_block(pos);
    let Some(m) = cookpot::machine_for(&block, || game.native_recipes_of_type(cookpot::MACHINE_TYPE).iter().map(|r| (r.id.clone(), (*r.json).clone())).collect()) else {
        return;
    };
    let recipes = cookpot::recipes_for(&m.id, || game.native_recipes_of_type(cookpot::RECIPE_TYPE).iter().map(|r| (r.id.clone(), (*r.json).clone())).collect());
    if entity.items.len() < m.slots.len() {
        entity.items.resize(m.slots.len(), None);
    }
    let int = |entity: &ModBlockEntity, key: &str| match entity.nbt.get(&horizon_world::nbt::JavaString::from(key)) {
        Some(horizon_world::nbt::Tag::Int(n)) => *n,
        _ => 0,
    };
    let mut st = cookpot::State {
        progress: int(entity, "progress"),
        total: int(entity, "total"),
        burn: int(entity, "burn"),
        burn_total: int(entity, "burn_total"),
        acc: int(entity, "acc"),
        recipe: int(entity, "recipe"),
    };
    let slots: Vec<Option<cookpot::Stack>> = (0..m.slots.len())
        .map(|i| {
            entity.items[i]
                .as_ref()
                .filter(|s| s.count > 0)
                .map(|s| cookpot::Stack { item: s.item.to_string(), count: s.count })
        })
        .collect();

    // Everything that reads the game happens here, before the slots change.
    let step = {
        let in_item_tag = |item: &str, tag: &str| game.items().item_tags.get(tag).is_some_and(|t| t.contains(item));
        let in_block_tag = |block: &str, tag: &str| game.items().block_tags.get(tag).is_some_and(|t| t.contains(block));
        let heat = match &m.heat {
            None => 0.0,
            Some(h) => {
                let at = [pos[0] + h.offset[0], pos[1] + h.offset[1], pos[2] + h.offset[2]];
                let name = game.native_block(at);
                cookpot::heat_speed(&m, &name, &|key: &str| game.native_block_property(at, key), &in_block_tag)
            }
        };
        let fuel_of = |item: &str| {
            cookpot::fuel_value(&m, item, &in_item_tag).unwrap_or_else(|| {
                if m.fuel.as_ref().is_some_and(|f| f.game_table) {
                    // The game's own fuel table: a private API, so a nightly may break this line.
                    let stack = ItemStack { item: std::sync::Arc::from(item), count: 1, damage: 0, extra: None };
                    game.providers().burn_time(game.items(), &stack, false)
                } else {
                    0
                }
            })
        };
        cookpot::step(&m, &recipes, &mut st, &slots, heat, &fuel_of, &in_item_tag)
    };

    // Use up one fuel item.
    if let Some(i) = step.fuel_slot {
        take(&mut entity.items[i], 1);
    }
    if let Some(craft) = &step.craft {
        for &(i, n) in &craft.consume {
            take(&mut entity.items[i], n);
        }
        let outputs = m.slot_indices(cookpot::Role::is_output);
        for res in &recipes[craft.recipe].results {
            let Some(made) = game.native_recipe_result(&res.json) else {
                log::warn!("cookpot: {}: result {} is not an item the game knows", recipes[craft.recipe].id, res.id);
                continue;
            };
            let mut left = made.count;
            // Like stacks first, then empty slots.
            for &i in &outputs {
                if let Some(s) = entity.items[i].as_mut() {
                    if s.item == made.item && s.damage == made.damage && s.extra == made.extra && s.count < cookpot::STACK_MAX {
                        let add = left.min(cookpot::STACK_MAX - s.count);
                        s.count += add;
                        left -= add;
                    }
                }
            }
            for &i in &outputs {
                if left > 0 && entity.items[i].is_none() {
                    let add = left.min(cookpot::STACK_MAX);
                    let mut s = made.clone();
                    s.count = add;
                    entity.items[i] = Some(s);
                    left -= add;
                }
            }
        }
    }

    for (key, value) in [
        ("progress", st.progress),
        ("total", st.total),
        ("burn", st.burn),
        ("burn_total", st.burn_total),
        ("acc", st.acc),
        ("recipe", st.recipe),
    ] {
        entity.nbt.insert(horizon_world::nbt::JavaString::from(key), horizon_world::nbt::Tag::Int(value));
    }
    entity.data = cookpot::data_slots(&m, &st);

    // A block property such as `lit` follows whether the machine is working.
    if let Some(key) = &m.lit_property {
        let want = if step.lit { "true" } else { "false" };
        if game.native_block_property(pos, key).as_deref() != Some(want) {
            game.native_set_block_property(pos, key, want);
        }
    }
}

// Takes `n` items from a slot, emptying it at zero.
fn take(slot: &mut Option<ItemStack>, n: u32) {
    if let Some(s) = slot.as_mut() {
        if s.count <= n {
            *slot = None;
        } else {
            s.count -= n;
        }
    }
}
