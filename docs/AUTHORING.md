# Cookpot authoring reference (v0.3.0)

> **Status: written, not run in the game.** The pure logic (parsing, matching, heat, fuel, modes, XP, sounds,
> add-on extensions) has unit tests that passed against a stand-in JSON library. The game hooks were never compiled
> against Horizon or run. See `docs/FINDINGS.md` for the untested list and `docs/TROUBLESHOOTING.md` if something
> does not work.

New here? Read [QUICKSTART.md](QUICKSTART.md) first (a working machine in five files). Making an add-on for
someone else's machine? Read [ADDONS.md](ADDONS.md). Copyable machines are in [EXAMPLES.md](EXAMPLES.md). Editors
can validate your JSON with the files in [`../schemas/`](../schemas/).

Cookpot is one native mod. You never write Rust: machines, recipes and add-ons are JSON inside a content mod
(`.hmod`).

## The file types

| You write | Where | `type` | What it is |
|---|---|---|---|
| Block | `data/<ns>/horizon/block/<name>.json` | (no Cookpot type) | The block the player places. |
| Menu | `data/<ns>/horizon/menu/<name>.json` | (no Cookpot type) | Slot count and which slots are results. |
| Machine | `data/<ns>/recipe/<name>.json` | `cookpot:machine` | Slots, screen, heat, fuel, mode, sounds. |
| Recipe | `data/<ns>/recipe/<name>.json` | `cookpot:recipe` | What a machine makes. May be in **any** mod. |
| Extension | `data/<ns>/recipe/<name>.json` | `cookpot:extension` | Adds heat blocks and fuels to a machine from **another** mod. |

Machine, recipe and extension files sit under `recipe/` because the game keeps recipe files of an unknown `type`
as raw JSON; that is how Cookpot finds them. **A machine's id is its file id: `<ns>:<file name>`.** The file
`data/cptest/recipe/pot_machine.json` is the machine `cptest:pot_machine`. Recipes and extensions point at this id.

## 1. The block

```json
{ "template": "minecraft:barrel", "item": true }
```

The template **must have a block entity** (barrel, chest, furnace). To show a lit look, use a template with a `lit`
property (furnace) and set `process.lit_property`.

## 2. The menu

```json
{ "slots": 5, "data_slots": 4, "results": [4] }
```

- `slots` must equal the number of slots in the machine file.
- `results` lists output slot numbers (zero-based, in the machine's slot order).
- `data_slots` is the number of synced numbers. Use **4** (progress, total, burn, burn total) to get both bars.

## 3. The machine

```json
{
  "type": "cookpot:machine",
  "block": "mypack:pot",
  "menu": "mypack:pot",
  "data_slots": 4,
  "mode": "timed",
  "heat": { "offset": [0, -1, 0], "sources": [ "minecraft:campfire[lit=true]", "#mypack:heat_sources" ] },
  "process": {
    "default_time": 200,
    "keep_progress_when_cold": false,
    "lit_property": "lit",
    "fuel": { "slot": "fuel", "burn_times": { "minecraft:coal": 1600, "#minecraft:planks": 300 }, "game_table": true }
  },
  "sounds": {
    "start": "minecraft:block.lava.pop",
    "working": { "id": "minecraft:block.campfire.crackle", "interval": 40, "volume": 0.6 },
    "done": { "id": "minecraft:entity.experience_orb.pickup", "pitch": 0.8 }
  },
  "gui": {
    "size": [176, 166],
    "texture": "mypack:gui/pot",
    "slots": [
      { "id": "in0", "role": "input", "pos": [30, 17] },
      { "id": "fuel", "role": "fuel", "pos": [39, 56] },
      { "id": "bowl", "role": "container", "pos": [92, 55] },
      { "id": "out", "role": "output", "pos": [124, 28] }
    ],
    "progress": { "pos": [60, 31], "size": [40, 6] },
    "flame": { "pos": [30, 46], "size": [40, 4] }
  }
}
```

### Top-level fields

| Field | Required | Meaning |
|---|---|---|
| `block` | yes | A block id, or a list of ids sharing the machine. |
| `menu` | yes | The menu type id from step 2. |
| `data_slots` | no | Synced numbers (0 to the slot limit). Use 4. |
| `mode` | no | `timed` (default), `instant`, `staged`. See [Modes](#modes). |
| `heat` | no | Blocks that power the machine. See [Heat](#heat-from-blocks). |
| `process` | no | Timing, fuel, lit state. |
| `sounds` | no | `start`, `working`, `done`. See [Sounds](#sounds). |
| `gui` | yes | Screen layout. |

### `process`

| Field | Default | Meaning |
|---|---|---|
| `default_time` | 200 | Ticks a recipe takes when it names no `time`. |
| `keep_progress_when_cold` | false | Keep progress when unpowered. Otherwise it drains 2 per tick. |
| `lit_property` | none | A block property set to `true` while the machine works, `false` otherwise. |
| `fuel` | none | Fuel slot. See [Fuel](#fuel). |

### `gui`

| Field | Default | Meaning |
|---|---|---|
| `size` | `[176, 166]` | Panel size in GUI pixels. |
| `texture` | none | A content-mod texture `assets/<ns>/textures/gui/<name>.png`, written `<ns>:gui/<name>`, drawn at the panel's top left. |
| `slots` | required | List of `{ "id", "role", "pos"? }`. `id`s are unique. |
| `progress` | none | Bar for craft progress: `pos`, `size` (default `[24, 4]`). |
| `flame` | none | Bar for fuel burn (timed) or charges (staged): `pos`, `size` (default `[14, 4]`). |

**Roles:** `input`, `fuel`, `container`, `output`. Players cannot fill output slots. A slot without `pos` is placed in
rows of three, centred (the built-in layout), so a machine needs no art to work.

## 4. Recipes

```json
{
  "type": "cookpot:recipe",
  "machine": "mypack:pot_machine",
  "inputs": [ { "item": "mypack:tomato", "count": 2 }, "#c:meats", ["minecraft:carrot", "minecraft:beetroot"] ],
  "container": "minecraft:bowl",
  "results": [ { "id": "mypack:tomato_soup", "count": 1 } ],
  "time": 200,
  "charges": 1,
  "experience": 1.0
}
```

| Field | Meaning |
|---|---|
| `machine` | **Required.** The machine id (`<ns>:<file name>`). Any mod may add recipes to any installed machine. |
| `inputs` | Shapeless; every entry needs its own input slot. An entry is an item id, `#tag`, list (any one), or `{ "item" \| "tag" \| "items", "count" }` (count 1-64). |
| `container` | An item the container slot must hold; one is used up. |
| `result` / `results` | One result or a list. Each is an item id or `{ "id", "count", "components"? }`. They fill output slots, merging into like stacks. If they do not fit, the machine waits. |
| `time` | Ticks; overrides `default_time`. |
| `charges` | Staged machines: charges spent per craft (default 1). |
| `experience` | XP points paid out when the craft finishes. See [Experience](#experience). |

The first matching recipe (in load order) wins. A bare id means `minecraft:`.

## Heat from blocks

`heat.sources` lists blocks that power the machine; the fastest matching source wins. Cookpot looks at the block at
`heat.offset` from the machine (default `[0, -1, 0]`, directly below; `[0, 1, 0]` is above, `[1, 0, 0]` beside).

| Form | Example |
|---|---|
| Block id | `"minecraft:campfire"` |
| With properties | `"minecraft:campfire[lit=true]"` |
| Block tag | `"#mypack:heat_sources"`, `"#mypack:hot[lit=true]"` |
| Object | `{ "block": "...", "tag": "...", "properties": { "lit": true }, "speed": 2.0 }` |

`speed` scales progress: 1.0 normal, 2.0 twice as fast. **Use a tag when you want other packs to add heat blocks**:
they add values to your tag and your machine needs no change. With a heat source present, fuel is not used.

## Fuel

`process.fuel.slot` names the fuel slot. `burn_times` maps an item id or `#tag` to **ticks** (`timed`, `instant`) or
**charges** (`staged`); an exact item id beats a tag. `game_table: true` also accepts anything the game burns in a
furnace (a private game API: a Horizon update may break it). A machine with neither `heat` nor `fuel` always runs.

## Modes

| Mode | Like | Behaviour |
|---|---|---|
| `timed` | furnace, smoker | Progress over `time`; fuel burns for its burn time and carries across crafts. |
| `instant` | cutting board | Crafts on the next tick the inputs match. It does not wait for a click. |
| `staged` | brewing stand | Fuel adds charges, each craft spends the recipe's `charges`, a recipe may have several results. A heat block runs it without spending charges. |

## Sounds

`sounds` has three optional cues. Each is a sound id string or `{ "id", "volume" (0-16), "pitch" (0-4), "interval" }`.

| Cue | Plays |
|---|---|
| `start` | When a craft begins making progress. |
| `working` | Repeatedly while a craft runs, every `interval` ticks of progress (default 40; only for this cue). |
| `done` | When a craft finishes. |

Ids are vanilla (`minecraft:block.campfire.crackle`) or your own (`mypack:pot_bubble`, defined in
`assets/mypack/sounds.json`). Sounds play for the local player. Guests on a LAN game may not hear them (untested).

## Experience

When a craft finishes, the recipe's `experience` is added to the machine's tally, and whole points are paid out as
experience orbs above the machine; fractions carry over to the next craft (0.7 pays 0, then 1, then 0, then 1...).
This is **not** vanilla's furnace behaviour (stored XP paid when you take the item): Horizon has no hook for taking
an item out of a mod's menu. The orbs are collected by nearby players.

## Machines from other mods

Recipes and extensions can target any machine by id. See [ADDONS.md](ADDONS.md).

## Limits you should know

- The menu cannot restrict which items a slot accepts in this Horizon build. Hoppers can put into any non-output slot
  and take from the output; players can place anything anywhere.
- Output slots assume a 64-item limit, whatever the item's real limit.
- Binding to a **vanilla block** (such as the fletching table) is not possible: no hook can attach slots or saved data to
  one. Add your own block that copies its look.
- Content is read once at game start. Editing JSON needs a restart, **not** a rebuild of Cookpot.
- Clients and servers need the same mods.
- A container's leftovers (an empty bucket) are not given back.

## Errors

A file that does not parse is skipped with a warning in `logs/latest.log` that names the file and says why; the game
still starts. A recipe or extension for a machine that is not installed also logs a warning that names the missing
machine. Common messages are in [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
