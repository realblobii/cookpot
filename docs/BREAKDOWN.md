# Cookpot: design breakdown (DRAFT)

> **Research done 2026-11-02: see `docs/FINDINGS.md`, section "Corrections to BREAKDOWN.md".** Corrections applied below:
> definitions and recipes are `type: cookpot:*` files under `data/<ns>/recipe/`, the author also ships a menu JSON, and
> `vanilla_block` binding is **dropped from v1** (no hook can attach slots or saved data to a vanilla block; user decision
> 2026-11-02). Everything here is still **unbuilt/untested in the game** except the Phase 1 scaffold, which is unrun.
>
> **Status: draft.** Written before reading the hook source or `native-mods.md` in full. Every
> technical claim here is a hypothesis to check against `OPEN_QUESTIONS.md`. Where the research
> contradicts this file, change this file and tell the user.

## 0. Decisions from the user

- Cookpot is a **library mod**: no blocks or machines of its own, only the engine and docs. A demo
  machine may exist as a separate optional demo mod or only in the docs.
- Definitions ship inside **content mods (`.hmod`)**.
- **Singleplayer first.** If a needed hook is missing: **stop and tell the user**.
- Two UI modes: **built-in layouts** or **author-supplied UI** (see section 2).

## 1. Concept

One native mod, `cookpot` (mod id: lowercase letters, digits, `_` only), provides a **generic
machine**: a block with an inventory, a menu, a tick process and a recipe lookup. Everything specific
to a machine is data:

- **Machine definition** (`data/<ns>/recipe/<name>.json`, `type: cookpot:machine`): which block, slot layout, GUI, heat rule, process.
- **Recipes** (`data/<ns>/recipe/*.json`, `type: cookpot:recipe`): inputs, container item, result, time, XP.
- **Block, model, textures, lang**: ordinary content-mod (`.hmod`) assets.

So the three actors are: Cookpot (native, written once), the author's content mod (no code), and
the player (installs both).

## 1b. Machine families and binding (added after the first draft)

**Process modes** (the engine must support all of these from one schema):

| Mode | Like | Behaviour |
|---|---|---|
| `instant` | cutting board, fletching bench | crafts on click or on taking the output; no ticking |
| `timed` | furnace, smoker, Farmer's Delight pot | inputs + optional fuel, a progress bar, output after N ticks |
| `staged` | brewing stand | a catalyst/fuel slot with charges, several input and output slots processed together, one run turns many items |

Timed machines need: per-recipe time, a fuel slot with burn times (or a heat block below), progress
that survives save and reload, and a rule for what happens when heat or fuel runs out (pause, reset or
decay like a furnace). A `staged` machine adds "fuel charges" (brewing stand blaze powder) and
multi-output recipes.

**Binding** (what block the machine attaches to):

1. `new_block`: a block from the author's content mod (the common case, e.g. the cooking pot).
2. ~~`vanilla_block`~~ **Out of v1** (Q15 in FINDINGS.md): no existing hook attaches slots or saved data to a
   vanilla block. Authors can instead add a replacement block that copies the fletching table's look.
3. `vanilla_block_replace`: replace the behaviour of a vanilla machine entirely (smoker with a
   different menu). Probably out of scope until 1 and 2 work; note conflicts with other mods.

The same definition file selects the mode and the binding.

## 2. Data formats (proposal, not final)

### Machine definition `data/<ns>/recipe/cooking_pot.json` with `"type": "cookpot:machine"`

```json
{
  "title": "Cooking Pot",
  "gui": {
    "texture": "mypack:textures/gui/cooking_pot.png",
    "size": [176, 166],
    "slots": [
      { "id": "in0", "role": "input", "pos": [30, 17] },
      { "id": "in1", "role": "input", "pos": [48, 17] },
      { "id": "in2", "role": "input", "pos": [30, 35] },
      { "id": "in3", "role": "input", "pos": [48, 35] },
      { "id": "fuel", "role": "fuel", "pos": [39, 56] },
      { "id": "container", "role": "container", "pos": [92, 55] },
      { "id": "out", "role": "output", "pos": [124, 28] }
    ],
    "progress": { "pos": [89, 25], "sprite": [176, 0, 24, 17], "direction": "right" },
    "heat_indicator": { "pos": [55, 56], "sprite": [176, 17, 14, 14] }
  },
  "heat": { "required": true, "check": "below", "sources": ["#mypack:heat_sources", "minecraft:campfire[lit=true]"] },
  "binding": { "type": "new_block", "block": "mypack:cooking_pot" },
  "process": { "mode": "timed", "default_time": 200, "keep_progress_when_cold": false,
               "fuel": { "slot": "fuel", "burn_times": { "minecraft:coal": 1600, "#minecraft:planks": 300 } } },
  "recipe_type": "mypack:cooking",
  "automation": { "insert_into": ["in0", "in1", "in2", "in3", "fuel", "container"], "extract_from": ["out"] },
  "sounds": { "working": "minecraft:block.campfire.crackle", "done": "minecraft:entity.experience_orb.pickup" }
}
```

(The sample shows both a `heat` block-below rule and a `fuel` slot to illustrate the options. A real
definition would normally use one or the other.)

### UI modes (`gui`)

```json
"gui": { "builtin": "furnace" }                      // a ready-made layout, no art needed
"gui": { "builtin": "brewing", "title": "Still" }    // same, with a custom title

"gui": {                                             // the author's own UI
  "texture": "mypack:textures/gui/cooking_pot.png",
  "size": [176, 166],
  "slots": [ { "id": "in0", "role": "input", "pos": [30, 17] } ],
  "progress": { "pos": [89, 25], "sprite": [176, 0, 24, 17], "direction": "right" }
}
```

Built-in layouts are named templates inside Cookpot that fix slot positions and roles (for example
`furnace`: input, fuel, output; `brewing`: three outputs, one ingredient, one fuel; `grid`: N inputs and
one output; `single`: one input and one output). They are drawn with the game's normal container
look. A machine definition using a built-in layout only supplies recipes, timing and binding. A custom UI
must name every slot and may mix built-in widgets. The loader validates both forms the same way.

### Recipe `data/<ns>/recipe/<id>.json` with `"type": "cookpot:recipe"`

```json
{
  "crafter": "mypack:cooking_pot",
  "inputs": [ { "item": "mypack:tomato", "count": 2 }, { "tag": "minecraft:meats" } ],
  "shapeless": true,
  "container": "minecraft:bowl",
  "result": { "item": "mypack:tomato_soup", "count": 1 },
  "time": 200,
  "experience": 1.0
}
```

Other machines are expressed in the same schema. A **cutting board** is `mode: "instant"` with one input
slot, a `tool` requirement and result drops. A **kiln** is `timed` with a `fuel` slot. A **brewing-stand
style** machine is `mode: "staged"` with several input and output slots plus a charge-giving fuel slot.
A machine with no heat uses `heat.required: false`.
Keep the schema small first; add fields only when a real machine needs them.

### Validation

The mod must reject bad definitions with a clear message in the log and in chat: unknown block, slot
id clash, recipe referencing a missing crafter, input count above slot count, overlapping slots, missing
texture. A definition that fails must not crash the game or silently do nothing.

## 3. Mod layers

| Layer | Who | What |
|---|---|---|
| Cookpot native mod | us | machine logic, menu/GUI generation, recipe matching, persistence |
| Example content mod ("Pot Demo") | us | a pot block, textures, a few recipes, as the proof and template for authors |
| Author content mods | others | blocks, assets, definitions, recipes |

## 4. Components (to confirm against the source)

1. **Definition loader**: finds `cookpot:machine` and `cookpot:recipe` files via `native_recipes_of_type` in installed content mods
   / data files at game load and on reload; builds an in-memory registry; validates. (Open Q 4, 5, 6.)
2. **Block binding**: connects a content-mod block id to a definition: placing it creates the machine's
   block entity with the right number of slots. (Open Q 1, 3.)
3. **Menu**: opens a menu with the definition's slots; slot rules per role (input accepts any, container
   accepts only the recipe's container item, output takes only). (Open Q 2; `native_menu_slot_rules`.)
4. **Screen**: draws the GUI texture, slots, progress arrow, heat flame, tooltips; from data, not
   hard-coded. (`native_screen`, `_draw`, `_tooltip`, `_click`.)
5. **Process**: per-tick or scheduled; check heat, find the matching recipe, advance progress, consume
   inputs, fill output, keep state across save/load. (Open Q 3, 7.)
6. **Recipe matching**: shapeless multiset match with tags and counts; optional container item; first
   match wins by a documented priority. Pure Rust in `runtime.rs` with unit tests (no game types).
7. **Persistence**: slots, progress, current recipe, saved with the chunk. (Open Q 3.)
8. **Drops and cleanup**: items drop when the block breaks; the machine's data is removed.
9. **Automation (later)**: hoppers insert/extract by slot role; comparator output. (Open Q 8, 10.)
10. **Rendering extras (later)**: items visible in the pot, particles. (Open Q 10.)
11. **Recipe viewer integration (later)**: expose recipes to the JEI port. (Open Q 11.)

## 5. Phases

- **Phase 0, research.** Read everything in `READING_LIST.md`, write `FINDINGS.md`, update this file.
  Output: go/no-go and a corrected design. **No code.**
- **Phase 1, vertical slice (written, not run).** Prove the hard hooks with the smallest possible library core: a throwaway
  test content mod supplies one block, and Cookpot opens a menu with 3 slots for it, with contents saved
  across reload. The test block and its hard-coded slots are scaffolding, not part of the shipped library.
  Proves open questions 1-3. Smallest possible build.
- **Phase 2, data-driven layout.** Replace the hard-coded slots with a definition read from a file.
  Proves 4 and 5: edit the JSON, reload, no rebuild.
- **Phase 3, recipes and process.** Matching, heat, progress, output, XP, persistence of progress.
  Build `timed` first, then `instant`, then `staged` (brewing-style multi-output with charges).
- **Phase 3b, vanilla binding.** Dropped from v1 (missing hook, FINDINGS Q15). Revisit only if Horizon adds one.
- **Phase 4, polish.** GUI details, tooltips, sounds, drops, validation messages.
- **Phase 5, automation and integration.** Hoppers, comparator, item display, recipe viewer.
- **Phase 6, docs and proof.** `docs/AUTHORING.md` is the main deliverable: how to write a definition
  and recipes, built-in layout reference, custom UI guide, binding options, error messages, worked
  examples (cooking pot, furnace-style kiln). A small demo `.hmod` made with zero Rust
  proves it; it is separate from the Cookpot library. Package Cookpot as `.hnmod`.

Each phase ends with a build the user runs in the game and a short checklist of what to look for.

## 6. Acceptance tests (the user runs these)

1. Place the pot, open it, put items in all slots, close, reopen: items are still there.
2. Quit to title, reload the world: items and progress are still there.
3. With heat below and valid inputs plus a bowl: progress bar fills, output appears, inputs consumed.
4. Remove heat: progress pauses or resets as defined.
5. Break the block: all contents drop.
6. Edit a recipe JSON, reload data: new behaviour with **no rebuild of the game**.
7. A malformed definition produces a readable error and the game still starts.
8. Disable the mod in Options > Mods: the world loads without the machine (document what happens to
   the block and its items).
9. A furnace-style definition smelts over time with fuel burn and a visible flame/progress, and keeps
   going correctly after save and reload.
10. A brewing-style definition processes several slots at once and uses up its charges.
11. (Dropped with vanilla binding.)
12. (Multiplayer, if supported) Two players see consistent contents; no duplication via shift-click.

## 7. Risks

- **Hooks may not exist** to create block entities or menus from a native mod; the feature may need
  upstream changes to Horizon (a PR or a request for new hook points; see `native-hooks-neoforge.md`).
- **Closure-heavy targets** force cancelling `HEAD` hooks everywhere and limit where logic can be inserted.
- **Nightly churn**: hook contracts are versioned and "never change", but targets can move when the game is
  split into crates. Pin an exact nightly and expect updates.
- **Runtime data loading** may not be possible for native mods (`native_data!` is compile-time per the
  docs). If so, the "no code" goal needs another route: investigate before building anything else.
- **Persistence** of custom block data in saves may be unsupported; unsaved machines would be useless.
- **Server vs client**: logic that only runs client-side breaks multiplayer.
- **Licensing**: use original art and recipes; do not copy assets from other mods.

## 8. If the goal turns out not to be feasible

Say so early, with evidence. Useful fallbacks to propose, in order: (a) a smaller kit limited to
instant "station" crafting (cutting-board style) if persistence is the blocker; (b) a request or
contribution to Horizon for the missing hook points; (c) a single-purpose native mod for one machine
(a Cooking Pot) that proves the hooks, then generalize.
