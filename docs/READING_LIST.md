# Reading list (do this before any code)

For each item: read the real text, not a summary, note the answer to anything in
`OPEN_QUESTIONS.md`, and record the source in `docs/FINDINGS.md`. WebFetch truncates long pages: use
`offset`, or better, clone `https://github.com/horizon-rust/horizon` and read files locally. Get the
pinned source with `git -C horizon archive <commit> | tar -x -C <readonly-dir>`.

## A. Modding docs (public site and repo `docs/`)

1. `https://playhorizon.dev/docs/mods` and repo `docs/mods.md`: overview of content vs native mods,
   install folders (`horizon-mods/`, `horizon-native-mods/`), Options > Mods, settings storage
   (`native_set_setting`, `horizon-mods/.horizon-mod-data/<id>/`, `config/`).
2. `https://playhorizon.dev/docs/make-a-mod`, `/docs/mod-manifest`, `/docs/mod-code`,
   `/docs/content-mods`, `/docs/test-a-mod`, `/docs/publish-a-mod`. These linked pages were **not read** in
   the previous session. They are the detailed content-mod and hook docs.
3. Repo `docs/native-mods.md` (very long: about 236k chars). Read **all** of it. Previously only the first
   100k chars were seen. Look for: hook API numbers, `native_screen*`, `native_block_entity_container`,
   `native_menu_slot_rules`, `native_random_tick`, `native_data!`, `[[crate]]`, `data/` staging.
4. Repo `docs/api/README.md` and `docs/api/stability.md` (versioning policy), `docs/native-hooks-neoforge.md`
   (a proposal for hooks modelled on NeoForge: shows what is missing today).
5. Repo `docs/smp.md`: a big real native mod (economy, market, trades...) and the hooks it needed.
   Good evidence of what is feasible.
6. Repo `docs/resource-packs.md` and any docs on custom blocks, block models, block entities,
   containers/menus, recipes, tags, data packs (`grep -ril "block entity\|menu type\|recipe" docs`).

## B. Source at the pinned commit (primary truth)

1. `native-hooks.json` (root): the catalog of named hooks and versions (108 entries at `c9d474d`).
   Read the whole list, not just names that look relevant.
2. `crates/horizon-client/src/game/native_hooks.rs`: where hooks are defined. Read the container,
   menu, screen, slot-rule and block-entity ones in full.
3. How **content mods declare blocks with containers/menus** (crates `horizon-mods`,
   `horizon-mod-registry`, `cargo-horizon-mod`): can a content mod add a block entity with inventory
   slots? Can it add recipes? Which recipe types?
4. The vanilla furnace, smoker, campfire, brewing stand and crafter implementations in Rust: they are the
   model for slot handling, ticking, progress, and persistence (NBT / block entity data).
5. How recipes are loaded and matched (`horizon-items`, `horizon-core`?), and whether new recipe
   types or recipe JSON from mods are supported.
6. Existing native mods in the registry that add machines or menus: the Recipes mod (hook version 2),
   `smp`, and any in `mod-browser/horizon-index.json` (`~/.local/share/horizon/mod-browser/`). Clone their
   sources and read how they use `native_screen` and the container hooks.

## C. horizon-mixins (the patch tool)

Clone `https://github.com/horizon-rust/horizon-mixins` and read `README.md` and all of `docs/`:
`mixin-api.md`, `advanced-injections.md`, `boundary-head-and-modules.md`, `effects-and-state.md`
(per-instance state and save data: **needed for block entity data**), `scopes-and-bridges.md`
(accessor/invoker bridges for private fields), `crates-data-and-named-hooks.md`.

## D. Reference implementation

`~/horizon-slider-mod/shader_sliders/`: bundle layout, `[[module]]`, cancellable HEAD, packaging. Read
`mixins/shader_sliders/*` and `tools/package.py`. See also `docs/LESSONS.md`.

## E. Before finishing the research

Write `docs/FINDINGS.md`, then tell the user: is the goal feasible, what changed from the draft,
and what you propose to build first. Wait for their go-ahead before coding.
