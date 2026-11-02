# Open questions (answer each in FINDINGS.md with a source)

The draft design depends on these. If the answer to a "blocker" is no, stop and tell the user before
designing further.

## Blockers

1. **Can a native mod create a new block entity type / inventory** that a content-mod block uses, with
   per-block slots that persist in the world? Or can it only hook existing ones? (`native_block_entity_container`
   "lets a mod block's container open a menu type": what does that mean exactly, and who creates the
   container?)
2. **Can a native mod define a new menu type with arbitrary slot positions and its own GUI texture?**
   (`native_screen` API 19, `native_screen_draw`, `native_menu_slot_rules`.) Can the layout be built at
   runtime from data, not hard-coded Rust?
3. **Where can a native mod store per-block data** so it saves and loads with the chunk? (mixins `[[state]]`
   is per-instance on existing structs: does it reach block entities? Or is there a native data API?)
4. **Can a native mod read data files (definitions inside content mods `.hmod`)** from a content mod, datapack or `native-data/<id>` at runtime
   (`native_data!` is compile-time per docs; is there runtime loading from `horizon-mods/`)? Crafter
   definitions must be editable without recompiling the game, otherwise the whole point is lost.
5. **Is a native mod rebuilt by the launcher every time its content changes?** If authors' definitions
   are plain data in a content mod, they must not trigger a recompile. Confirm.

## Design-shaping

6. Do content mods already support recipes (which types), tags, loot tables, block states and block
   entity data? What does the content manifest (`api` 0.2) allow?
7. How do vanilla machines tick (random tick vs scheduled vs block entity tick) and what hook gives a
   native mod a per-block tick? (`native_random_tick` only fires for random-tick blocks.)
8. How does the game decide hopper/automation interaction with a block's inventory (`WorldlyContainer`
   side rules)? Is there a hook?
9. Are item stacks on a custom menu synced to the server in multiplayer? Is the logic server-side or
   client-side in Horizon's rewrite? (The client crate is `horizon-client`, the server is `horizon-server`;
   native mods may be client-only or also server-side. Find out which crate a hook target lives in.)
10. Which hooks exist for **drops when broken**, **comparator output**, **rendering the block entity**
    (items visible in the pot), **sounds** and **particles**?
11. How is a recipe viewer (the JEI port in the mod index) fed? If Cookpot adds recipes, can they
    show there?
12. Compatibility: how many hooks does Cookpot need, and do they collide with other mods the user
    runs (two mods hooking the same target at HEAD run in priority order; OVERWRITE conflicts with everything)?

## Added: machine types and binding

15. **Can a native mod intercept right-click ("use") on a vanilla block** such as the fletching table and
    open its own menu? Which hook, which function, and can it be cancelled so the vanilla behaviour does
    not also run? Does the same work for blocks with no block entity (the table has none), i.e. where is
    the machine's data stored when the vanilla block has no block entity? (Possibly per-position state in
    the chunk; check how Cookpot could attach data to a block that lacks a block entity.)
16. How does vanilla model **furnace-style timing**: lit time, cook progress, fuel burn values (data or
    code)? Can fuel burn times be read from the game's own table so authors do not repeat them?
17. How does the vanilla **brewing stand** hold charges, process three outputs at once and sync progress?
    That is the template for the `staged` mode.
18. Can the same menu serve several bound blocks (one definition, many blocks)?

## Added: UI

19. What is the **simplest reusable UI** the game offers for container menus (furnace, brewing stand,
    generic 9xN container, hopper)? Can a native mod open one of those layouts with a different slot
    count or role, giving the "built-in UI" mode almost for free?
20. For the **custom UI** mode: how does `native_screen` / `native_screen_draw` load a texture from a
    content mod, and can slot positions, sprites and tooltips all come from data?
21. Can a content mod ship GUI textures and have them found by resource id at runtime?

## Practical

13. Which Nightly does the user have now, and does it still contain every hook the design needs?
    (Hooks are versioned: `hook_version`. A hook "never changes", a new version is added instead.)
14. What is the licensing situation for anything reused from other mods (Farmer's Delight etc.)? Prefer
    original art and recipes; do not copy assets.
