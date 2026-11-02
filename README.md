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

Research is done (`docs/FINDINGS.md`). The engine is **written but untested in the game**: machine, recipe and add-on extension files,
`timed` / `instant` / `staged` modes, heat from author-chosen blocks (ids, properties, tags, any offset), fuel, bars, XP orbs, sounds,
cross-mod machines (recipes and extensions that target another mod's machine by id), and demo mods (`testmod/`). The pure logic has unit tests (`cargo test`); the game hooks have **never been
compiled against Horizon**. v1 targets **singleplayer**. Binding to vanilla blocks (fletching table) is out of v1:
no hook can do it. `docs/AUTHORING.md` is the author guide.

## Files

- `CLAUDE.md`: rules for Claude working in this folder. Read first.
- **Docs for mod authors:** `docs/QUICKSTART.md` (first machine), `docs/AUTHORING.md` (full reference),
  `docs/ADDONS.md` (using machines from other mods), `docs/EXAMPLES.md` (cooking pot, kiln, cutting board, still),
  `docs/TROUBLESHOOTING.md`, and `schemas/*.schema.json` (editor validation).
- `docs/READING_LIST.md`, `docs/OPEN_QUESTIONS.md`, `docs/FINDINGS.md`: the research and what is untested.
- `docs/BREAKDOWN.md`: design, phases, acceptance tests. `docs/LESSONS.md`: build and tooling lessons.
- `mixins/cookpot/`: the native mod. `testmod/`: demo content mods (`cptest` machine, `cpaddon` add-on using it).
