// A machine's texture (when it has one) and its progress bar.
fn cookpot_draw(game: &crate::game::Game<'_>, block: &str, data: &[i32], batch: &mut HudBatch, origin: [f32; 2]) {
    let Some(m) = cookpot::machine_for(block, || game.native_recipes_of_type(cookpot::MACHINE_TYPE).iter().map(|r| (r.id.clone(), (*r.json).clone())).collect()) else {
        return;
    };
    if let Some(texture) = &m.texture {
        game.native_sprite(batch, texture, [0.0, 0.0, m.width, m.height], [origin[0], origin[1], m.width, m.height]);
    }
    // Progress: data slots 0 and 1. Burn time or charges: data slots 2 and 3.
    let bars = [(&m.progress, 0xFF55_FF55_u32, 0), (&m.flame, 0xFFFF_9922_u32, 2)];
    for (bar, color, at_data) in bars {
        let Some(bar) = bar else { continue };
        let at = [origin[0] + bar.pos[0], origin[1] + bar.pos[1]];
        game.native_fill(batch, [at[0], at[1], bar.size[0], bar.size[1]], 0xFF55_5555);
        let done = cookpot::fraction(data.get(at_data).copied().unwrap_or(0), data.get(at_data + 1).copied().unwrap_or(0));
        game.native_fill(batch, [at[0], at[1], (bar.size[0] * done).floor(), bar.size[1]], color);
    }
}
