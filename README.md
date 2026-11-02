# Cookpot

**Cookpot is a library mod.** It does not add any blocks or machines of its own. It provides the engine
(slots, menus, timed processing, recipe matching, saving) plus **documentation** so other mod authors can
add new crafting methods from a content mod, with no code mod of their own. Any example machine (such as a
cooking pot) is a demo that lives in the docs or a separate demo mod, not in Cookpot itself.

A native mod for the [Horizon](https://github.com/horizon-rust/horizon) Minecraft client (an unofficial
Rust rewrite of Java Edition 26.3) that lets pack authors add **new kinds of crafting machines**
(Farmer's Delight's cooking pot, a cutting board, a kiln, a loom, a smoker variant) using **data files
only**. The machine's behaviour lives in Cookpot; the machine's definition, recipes and art live
in a content mod (`.hmod`) or in data files.

## Why

Horizon cannot run Java mods. It has two mod kinds:

- **Content mods (`.hmod`)**: blocks, items, data and assets, **no code**. Easy to make, limited.
- **Native mods (`.hnmod`)**: Rust source patches compiled into the game. Powerful, but every author
  needs a Rust toolchain, a 20-30 minute first build, and knowledge of the hook API.

Machines with their own menu and recipes (cooking pot, etc.) currently need a native mod each.
Cookpot is the one native mod that provides the generic machine, so authors never write Rust.

## What kinds of machine it should cover

- **Timed machines, like a furnace**: inputs plus optional fuel, a progress bar, output when done.
- **Multi-slot, brewing-stand style**: several inputs and outputs, a fuel or catalyst slot, one process
  that produces more than one result, or a staged process. The Farmer's Delight cooking pot is this kind.
- **Instant workbenches**: crafting happens when you click (cutting board, fletching bench, loom-like).
- **New blocks or upgraded vanilla ones**: a machine can be bound to a block the author adds, **or to a
  vanilla block** that has no menu today, such as the **fletching table**, so it becomes a real
  workbench. Other candidates: cartography table, smithing-style stations, composter, lectern.

All of these are the same engine with different data, so authors get "all sorts of workbench blocks"
without new code.

## UI: built-in or your own

Authors choose per machine:

- **Built-in UI**: pick a ready-made layout (`furnace`, `brewing`, `grid`, `single`, ...) and Cookpot draws
  it with the game's normal GUI look, so an author needs no art to get a working machine.
- **Custom UI**: the author supplies a GUI texture plus slot positions, progress and flame sprites, for a
  fully custom look.

Both use the same data format; only the `gui` section differs.

## What an author should be able to do (target experience)

1. Make a content mod with a block `mypack:cooking_pot`, its model and textures (or choose a vanilla
   block to upgrade, such as the fletching table).
2. Add `crafters/cooking_pot.json` describing the machine's slots, UI, heat or fuel rules and process.
3. Add `crafter_recipes/*.json` for what it makes.
4. Install Cookpot (a dependency) and their content mod. The block opens the menu and crafts. No Rust.

Definitions ship **inside the author's content mod (`.hmod`)**. That is the supported route for v1.

## Status

Research is done (`docs/FINDINGS.md`) and a **Phase 1 scaffold** exists under `mixins/cookpot/` plus a throwaway
test mod in `testmod/`. **None of it has been built or run in the game yet.** v1 targets **singleplayer**.
Binding to vanilla blocks (fletching table) is **out of v1**: no hook can do it. Next step: build the Phase 1 bundle
and run the acceptance check in `FINDINGS.md` ("Proposed first build"). `docs/AUTHORING.md` covers what exists so far.

## Files

- `CLAUDE.md`: rules for Claude working in this folder. Read first.
- `docs/READING_LIST.md`: what to read, in order, and how.
- `docs/OPEN_QUESTIONS.md`: questions the research must answer before design is final.
- `docs/BREAKDOWN.md`: draft design, data formats, phases, acceptance tests.
- `docs/LESSONS.md`: build and tooling lessons from building the Shader Sliders mod.
- `docs/FINDINGS.md`: research answers with sources.
- `docs/AUTHORING.md`: how to write a machine (partial, untested).
