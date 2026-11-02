# Authoring a Cookpot machine (v0.1.x, Phase 1)

> **Status: partial and untested in the game.** Everything below is read from Cookpot's source and from
> `docs/FINDINGS.md`. Nothing has been built or run in Horizon yet. Only what the Phase 1 scaffold
> understands is documented here. Recipes, heat, fuel and process modes are **not implemented yet**; they are
> listed under "Not yet" so you do not rely on them.

Cookpot is a native mod that you install once. You write **no Rust**: a machine is a few JSON files inside
your own content mod (`.hmod`). Cookpot discovers them when a world loads.

## What you ship in your content mod

A machine needs three files that must agree with each other.

### 1. The block: `data/<ns>/horizon/block/<name>.json`

```json
{ "template": "minecraft:barrel", "item": true }
```

The template **must have a block entity** (barrel, chest or furnace). The game refuses the block otherwise.
Model and textures are ordinary content-mod assets.

### 2. The menu: `data/<ns>/horizon/menu/<name>.json`

```json
{ "slots": 3, "data_slots": 2, "results": [2] }
```

`slots` must equal the number of slots in the machine file, or the menu is dropped. `results` lists the
output slot numbers (zero-based, in the order you list slots). `data_slots` is the number of synced numbers
(progress bars read these).

### 3. The machine: `data/<ns>/recipe/<name>.json`

It lives under `recipe/` because the game keeps recipe files of an unknown `type` as raw JSON, which is how
Cookpot finds it.

```json
{
  "type": "cookpot:machine",
  "block": "mypack:pot",
  "menu": "mypack:pot",
  "data_slots": 2,
  "gui": {
    "size": [176, 166],
    "texture": "mypack:gui/pot",
    "slots": [
      { "id": "in",    "role": "input" },
      { "id": "extra", "role": "input" },
      { "id": "out",   "role": "output", "pos": [116, 30] }
    ],
    "progress": { "pos": [60, 31], "size": [40, 6] }
  }
}
```

| Field | Meaning |
|---|---|
| `block` | A block id, or a list of ids. Several blocks may share one machine. |
| `menu` | The menu type id from step 2. |
| `data_slots` | Synced numbers, 0 to the slot limit. Should match the menu file. |
| `gui.size` | Panel size in GUI pixels. Default `[176, 166]`. |
| `gui.texture` | Optional. A content-mod texture at `assets/<ns>/textures/gui/<name>.png`, written `<ns>:gui/<name>`, drawn at the panel's top left. |
| `gui.slots` | Required list. Each has a unique `id`, a `role`, and an optional `pos` `[x, y]` (top left of the slot). |
| `gui.progress` | Optional bar: `pos`, and `size` (default `[24, 4]`). |

**Roles:** `input`, `fuel`, `container`, `output`. Output slots cannot be filled by the player.

**Built-in layout:** leave out `pos` on a slot and Cookpot places it in rows of three, centred, so you need no
art to get a working screen. Give every slot a `pos` and a `texture` for a fully custom look.

## Errors

A machine file that does not parse is skipped with a warning in `logs/latest.log` that says why, for example
`two slots are called `in``, or ``slot `x`: role `y` is not input, fuel, container or output``. The game still starts.

## Limits you should know

- The menu cannot restrict which items a slot accepts in this Horizon build. Hoppers are limited by role, and
  the machine's own logic ignores wrong items, but players can still place anything.
- Binding to a **vanilla block** (such as the fletching table) is not possible: no hook can attach slots or saved
  data to one. Add your own block that copies its look instead.
- Content is read once when the game starts. Editing your JSON needs a game restart, **not** a rebuild of Cookpot.
- Clients and servers need the same mods.

## Not yet (do not use)

Recipe files (`cookpot:recipe`), heat sources, fuel and burn times, `instant` and `staged` modes, sounds, items
drawn in the block, comparator output, recipe viewer support. See `docs/BREAKDOWN.md`, section 5.

## Try it

`testmod/cptest-0.1.0.hmod` is a throwaway one-block example (source in `testmod/cptest/`). It is scaffolding
for the Phase 1 check, not part of the library.
