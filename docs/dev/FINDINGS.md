# Findings (Phase 0 research)

Pinned source: Horizon `c9d474dd9116e84c243f8992d75c11314df56f06` (`NATIVE_HOOKS_API = 80`,
`crates/horizon-client/src/game/native_hooks.rs:335`). This matches the launcher's last build
(`~/.local/share/horizon-launcher/native/built.json`, `release.horizon`), so the pin is right for the user's
game today. A read-only copy of that commit was used for every line number below. In this file
`hooks.rs` means `crates/horizon-client/src/game/native_hooks.rs` and `docs/...` means the repo's `docs/`
at the pinned commit.

**Verdict: feasible for `new_block` machines (timed, instant, staged), singleplayer first. Not feasible as
drafted for `vanilla_block` binding (fletching table): a hook is missing. See Q15.** Nothing has been built or
run yet.

## What was and was not read

Read in full or in the relevant part: `docs/native-mods.md` lines 1-680 (every API note 2-80, bundle, data
files, post-build) and the "Server menus" section; `docs/mods.md` 1-350 (content mods, menu types);
`native-hooks.json` (whole list of names); `hooks.rs` for every block-entity, container, screen, menu,
sprite, data, recipe and click hook plus their helpers; `horizon-client-render/src/chunk/block_entity.rs`
(`ModBlockEntity`, `ModContainer`, `MOD_MENUS`); `horizon-mods/src/content.rs` (`MenuType`);
`game/block_entity_ticks.rs`, `game/spill.rs`, `hud.rs` (GUI atlas); horizon-mixins `crates-data-and-named-hooks.md`,
`boundary-head-and-modules.md`; the shader-sliders reference bundle; the mod browser index (45 projects).

**Not read** (so treat related claims as unchecked): `docs/native-mods.md` lines 680-1330 beyond the
sections quoted here (launcher behaviour, the helper catalog "draft" sections), `docs/smp.md`,
`docs/native-hooks-neoforge.md`, `docs/api/*`, `docs/resource-packs.md`, `docs/modpacks.md` beyond block
templates, the public site pages (`playhorizon.dev/docs/*`), horizon-mixins `mixin-api.md`,
`advanced-injections.md`, `scopes-and-bridges.md`, and the source of the Recipes and JEI mods. I did not find a
single native mod in the index that uses the machine hooks, so **there is no working reference to copy**.

## Blockers

**1. Block entity type / inventory from a native mod: yes, for mod blocks.**
`native_block_entity(block, entity)` makes a block entity when the block is a mod's (hooks.rs:1550-1561);
`native_block_entity_container(block, container)` gives it item slots, hopper faces and a menu
(hooks.rs:1587-1602). `ModContainer { slots, faces, inputs, outputs, menu }` and
`ModBlockEntity { id, nbt, items, data }` are in `horizon-client-render/src/chunk/block_entity.rs:97-122`.
The block's **template must itself have a block entity** (copy a chest, barrel or furnace), otherwise the game
refuses with "no block entity ... its template has none (copy a chest or furnace)"
(hooks.rs:4196-4210, `game/mods.rs:319`). A native mod does not create the container; it names the slots and
the game owns the `WorldlyContainer`. Breaking the block drops the slots (`game/spill.rs:480-487`).

**2. Custom menu with arbitrary slot positions and GUI texture: yes, but the slot count lives in a content-mod
file.** The block's container opens either one of 11 vanilla menus (`MOD_MENUS`: furnace, blast furnace,
smoker, hopper, 3x3, 9x1..9x6) or a content mod's own menu type, declared in
`data/<id>/horizon/menu/<name>.json` as `{ "slots": N, "data_slots": M, "results": [..] }` (`docs/mods.md:223-242`,
`horizon-mods/src/content.rs:285-299`). The container's slot count **must equal** the menu's, otherwise the menu is
dropped (hooks.rs:4234-4241, `mod_menu` at 4782). `native_screen(block, screen)` lays it out: `NativeScreen
{ width, height, slots: Vec<Option<[f32;2]>>, inventory_x, inventory_y, own_labels }` (hooks.rs:802-820).
It is called with only the block name and `&self`, so the layout **can be built at runtime from data**. A slot
left `None` is not in the menu (shift-click skips it). `native_screen_draw(block, data, batch, origin)` draws
over a plain panel with slot frames, using `native_sprite`, `native_fill`, `native_text`, `native_item`
(hooks.rs:1716-1738, 3539-3575, 3718). Not yet in the game: per-slot insert rules beyond `results`, a
`stillValid` hook (docs/mods.md:240-242). The slot rule "container slot accepts only the recipe's bowl" cannot
be enforced by the menu; it can only be enforced by hopper `inputs`/`faces` and by the machine's own tick.

**3. Per-block persistence: yes.** `ModBlockEntity.nbt` and `items` are saved with the chunk
(`save.rs:622, 903`; `docs/native-mods.md:86-87`). `data: Vec<i32>` are the menu's synced data slots, not saved
(`block_entity.rs:97-101`), so progress must be kept in `nbt` and copied to `data` every tick. mixins `[[state]]`
is not needed.

**4. Reading data from content mods at runtime (no recompile): yes, two routes.**
(a) Recipe files whose `type` is outside `minecraft`, in `data/<ns>/recipe/*.json` of a content mod, are kept
as raw JSON and returned by `native_recipes_of_type(kind)` as `CustomRecipe { id, json }`
(hooks.rs:3846-3862, `crafting.rs:1168`, `docs/native-mods.md:209-213`). Plus `native_recipe_ingredient` (items,
tags, lists) and `native_recipe_result` (hooks.rs:3864-3900). This is **a discovery mechanism**: one file per
machine (`"type": "cookpot:machine"`) and one per recipe (`"type": "cookpot:recipe"`). Empty outside a world.
(b) `native_data("data/<ns>/anything.json")` reads any file under the resources `data/` including content mods
(hooks.rs:3671-3678), but cannot list a folder. Route (a) finds files, (b) is the fallback for extra files.
Caveats: the content mod's recipe folder is the author's namespace, not `cookpot`; whether
`cargo horizon-mod check` accepts an unknown recipe `type` is **untested**; the `recipe_filter` and
`neoforge:conditions` features (docs/native-mods.md:360-399) apply to these files too.

**5. Rebuild behaviour: no rebuild for content.** The game reads content once at start
(`docs/mods.md:48-49`). Authors' definitions are plain data in their `.hmod`, so only the player's game restart is
needed. A native mod itself is rebuilt by the launcher when installed or changed (build.log shows 5m10s for the
last build here). Cookpot's own code changes cost a rebuild; author data does not.

## Design-shaping

**6. Content-mod features** (`docs/mods.md:71-335`): blocks and items from a vanilla template, `state_properties`
(enum/boolean, e.g. `lit`), creative tabs, mob effects, entity types, menu types, tags (merge as data packs
do), loot tables/recipes in the mod's namespace, `recipe_filter`, `neoforge:conditions`. Block entity data is not
a content feature; it comes from the native mod.

**7. Ticking:** `native_block_entity_ticks(block, &mut ticks)` then `native_block_entity_tick(pos, entity)` every
server tick, in vanilla order, with `&mut self` (hooks.rs:1562-1566, 1618-1630). The table is built once on the
first block-entity tick of a world (`block_entity_ticks.rs:24-31`), when a player and the recipes exist, so the
data route in Q4 is available then. `native_set_block_property` can drive a `lit` state (API 18/33).

**8. Automation:** `ModContainer.faces/inputs/outputs` are exactly `getSlotsForFace`, `canPlace...`, `canTake...`
(`block_entity.rs:109-122`). Comparator: `native_has_analog_output` / `native_analog_output` (API 64).

**9. Server vs client:** `native_block_entity_tick`, `native_menu_button`, the container and `native_screen`
(server also asks it) run on the server thread; `native_screen_draw/click/tooltip` run on the client. Singleplayer
has both. Clients and servers need the same mods (`docs/mods.md:77-78, 238-240`). Multiplayer is not ruled out.

**10. Drops, comparator, rendering, sound, particles:** drops yes (Q1). Comparator yes (API 64). Items drawn in the
block: `native_block_entity_submit` + `native_draw_item` (API 71, hooks.rs:1654-1675). Sounds:
`native_play_sound` (own sounds API 61). Particles: `native_send_particles` (API 11).

**11. Recipe viewer:** custom recipe types are "not crafted by the game and not shown in the recipe book"
(`docs/native-mods.md:212`). The panel API (`native_panel_recipes`, hooks.rs:3836-3846) belongs to one mod id at
a time. Showing Cookpot recipes in the JEI port is **not free**; later phase, unverified.

**12. Compatibility:** the hooks Cookpot needs are all `HEAD` hooks into empty hook functions that take
`&mut Option<..>` out-parameters (no OVERWRITE, no closures problem). Several mods can answer; each must answer
only for its own blocks. Cookpot must answer only for blocks that appear in a loaded machine definition.

**13. Nightly:** user's built game is `c9d474d`; pin matches. Re-check `built.json` at the start of each session.

**14. Licensing:** original art/recipes only; no assets from Farmer's Delight etc.

## Machine types and binding

**15. Vanilla block (fletching table): NOT possible with an existing hook. Stop and decide.**
- `native_block_entity` is only meaningful for mod blocks; for a block whose template has no block entity (the
  fletching table) the game warns and drops it (hooks.rs:4196-4210). `native_containers` skips every `minecraft:`
  block (hooks.rs:4229-4231). So Cookpot cannot attach slots or saved data to a vanilla block.
- `native_right_click_block(pos, face, canceled)` can cancel the click (hooks.rs:2434-2441) but nothing then opens
  a container menu with persistent slots. `native_open_menu` (API 10, `docs/native-mods.md:1287-1306`) opens a stock
  chest-looking menu filled with plain items, with no texture of ours and no storage.
- The closest working option: **an author's own block** that copies the fletching table's look through a model and
  uses a chest/barrel template (`new_block`), i.e. a *replacement* block, not an upgrade of the vanilla one.
- Possible workaround (not tried, and invented by me, so **not** doing it without your say-so): intercept the click,
  cancel it, open a `native_open_menu` chest and keep the state in a Cookpot-owned file keyed by position.
  It would have no custom texture, a stock chest layout, and its own save code.
- Per `CLAUDE.md` I am stopping here on this item and asking.

**16. Furnace timing and fuel:** vanilla's fuel time is `Providers::burn_time(items, stack, fast)`
(`horizon-client-render/src/providers.rs:108`), used by `furnace.rs:88`. It is not a documented hook helper, but
hooks compile inside `game::native_hooks`, where `self.providers()` and `self.items()` (private on `Game`,
`game.rs:1789-1810`) are visible. Reading the game's own fuel table is possible but **depends on private API**, so a
nightly can break it. Fallback is `burn_times` in the definition (as drafted).

**17. Brewing stand template:** not read in detail. A `staged` machine can be built on the same container; the
brewing logic is ours (charges in `nbt`).

**18. One definition, several blocks:** yes. Hooks are keyed by block name, so a definition may list several
`blocks`.

## UI

**19. Built-in UI:** Horizon already draws the plain panel and slot frames for a mod screen; the author only
supplies slot positions. So a "built-in layout" is a table of positions in Cookpot (`furnace`, `grid`, `single`,
`brewing`), drawn with vanilla sprites by `native_sprite`. For vanilla layouts a block can use the stock
`minecraft:furnace` menu: its 3 slots and 4 data slots draw the flame and arrow with no art
(`block_entity.rs:138-141`). Vanilla sprite names for the arrow/flame were **not** looked up yet.

**20. Custom UI:** `native_sprite(batch, name, src, dst)` with a content mod's `assets/<ns>/textures/gui/x.png`
as `<ns>:gui/x` (hooks.rs:3539-3555, `horizon-client-render/src/hud.rs:561-564`). Slots, progress and tooltips all
come from our own data via `native_screen`, `native_screen_draw`, `native_screen_click`, `native_screen_tooltip`
(hooks.rs:2111-2135).

**21. Textures from a content mod found at runtime:** yes (Q20).

## Corrections to `docs/dev/BREAKDOWN.md`

1. **Definition and recipe format**: files go in `data/<ns>/recipe/` with a `type` of `cookpot:machine` or
   `cookpot:recipe`, not `crafters/` and `crafter_recipes/` (the game only keeps unknown-type recipes; arbitrary
   folders cannot be listed).
2. **Menu**: an author must also ship `data/<ns>/horizon/menu/<name>.json` whose `slots` equals the definition's
   slot count, plus a block JSON whose template has a block entity (barrel/chest/furnace). Cookpot should validate
   all three agree and log a clear error.
3. **`vanilla_block` binding is removed from v1** (Q15). `new_block` is the only binding until you decide.
4. **Slot rules** (container slot accepts only a bowl) cannot be enforced by the menu in this build. Enforce on
   hopper `inputs` and in the tick; document that players can still place anything.
5. **Heat/"block below"** uses `native_block(pos)`; tags via `native_block_in_tag`/`native_item_in_tag` (API 28)
   inside tick hooks. Not yet exercised.
6. **Cookpot is a `[[crate]]` plus thin helpers**: pure logic (definition parsing, validation, recipe matching,
   progress) in a crate that depends only on crates already in Horizon's lock (`serde_json`, checked at build);
   it can be unit-tested with plain `cargo test`. Game glue stays in helpers (docs/native-mods.md:336-358).
7. **Untested assumption list** (everything above is read, not run): recipe `type` accepted by the content-mod
   checker; a chest/barrel template block opening our menu rather than its own; screen hooks firing with our
   layout; `native_data` reading from a packed `.hmod`; fuel table via private API.

## Proposed first build (Phase 1, after your go-ahead)

A throwaway test `.hmod` (one block with a barrel template, one menu JSON with 3 slots + 2 data slots, one
`cookpot:machine` file) and the smallest Cookpot: `block_entity`, `block_entity_container`, `screen`,
`screen_draw`, `block_entity_ticks`/`tick` (counter in `nbt`). Acceptance: open the block, items persist across
reopen and world reload, the counter draws on the screen. That proves Q1-Q5 in the real game.

## Phase 2-3 build (2026-11-02, written the same evening; NOT run in the game)

Implemented in `mixins/cookpot/`: recipe parsing and shapeless matching with tags, counts and a container slot;
`timed`, `instant`, `staged` modes; heat from blocks at an offset (block id, `[property=value]`, block tags,
per-source speed); fuel by item id or tag, optionally the game's own table; `lit` property; progress and burn bars.

**Verified:** the pure logic compiles and 11 unit tests pass, but against a hand-written stand-in for `serde_json`
(crates.io was unreachable here), so real `serde_json` behaviour (for example `as_i64` on `2.0`) is unchecked.
**Not verified:** `tick.rs` and `draw.rs` were written against the pinned source (`native_block`,
`native_block_property`, `native_set_block_property`, `native_recipe_result`, `items().item_tags/block_tags`,
`providers().burn_time`, `ItemStack` fields) but never compiled, and nothing ran in Horizon. Likely first failures:
a borrow in `cookpot_tick`'s closures, `game.items()`/`providers()` visibility, tag key format (with or without a
namespace), whether the `tags/block/` folder name is right for content mods, and whether the unknown recipe `type`s
pass `cargo horizon-mod check`.

## Phase 4 (XP, sounds, cross-mod machines; same evening, NOT run in the game)

- **XP:** `Game::award_orbs(at, amount)` (`game/spill.rs:449`) spawns orbs; used above the machine when a craft finishes.
  Fractions carry in the block entity (`xp`, thousandths). Vanilla's pay-on-take is not possible: no hook for taking from a
  mod menu was found (`native_give_experience` needs a player uuid; the tick hook has none).
- **Sounds:** `Game::native_play_local_sound` (`native_hooks.rs:3504`) plays for this window's player only and returns false
  on a server. `GameLevel::native_play_sound` (server queue, API 61) is not reachable from a block-entity tick hook as written,
  so LAN guests likely hear nothing. Custom sound ids work if the mod ships `assets/<ns>/sounds.json` (docs line 580).
- **Cross-mod:** `native_recipes_of_type` returns every installed content mod's files of a type, so a recipe or
  `cookpot:extension` in any mod can name another mod's machine by id (`<ns>:<file name>`). Extensions are applied after all
  machines are read. Missing targets log a warning. The content manifest has no documented dependency field, so none is used.
- **Verified:** 14 unit tests pass against a stand-in JSON library; the demo mods' files validate against `schemas/` and parse.
- **Untested guesses:** `award_orbs` visibility and effect from a tick hook, `native_play_local_sound` firing in singleplayer
  from a server-thread tick, whether block entities tick while unloaded neighbours exist, `tags/block/` path for content mods.
