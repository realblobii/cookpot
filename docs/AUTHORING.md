# Authoring a Cookpot machine (v0.2.0)

> **Status: written, not run in the game.** The pure logic (parsing, recipe matching, heat, fuel, the three
> process modes) has unit tests that passed against a stand-in JSON library, not the real `serde_json`. The
> game hooks (`tick.rs`, `draw.rs`, `container.rs`...) were **never compiled against Horizon and never run**.
> Expect kinks. See `docs/FINDINGS.md` for the list of untested assumptions.

Cookpot is a native mod that you install once. You write **no Rust**: a machine is a few JSON files inside
your own content mod (`.hmod`). Cookpot finds them when a world loads. `testmod/cptest/` is a complete example.

## What you ship in your content mod

### 1. The block: `data/<ns>/horizon/block/<name>.json`

```json
{ "template": "minecraft:barrel", "item": true }
```

The template **must have a block entity** (barrel, chest or furnace). To show a lit state, copy a template with a
`lit` property (furnace) and name it in `process.lit_property`.

### 2. The menu: `data/<ns>/horizon/menu/<name>.json`

```json
{ "slots": 5, "data_slots": 4, "results": [4] }
```

`slots` must equal the number of slots in the machine file. `results` lists the output slot numbers
(zero-based, in the order the machine lists its slots). Use `data_slots: 4` to get the progress and burn bars.

### 3. The machine: `data/<ns>/recipe/<name>.json`

It lives under `recipe/` because the game keeps recipe files of an unknown `type` as raw JSON; that is how Cookpot
finds it. Its id (`<ns>:<name>`, here `cptest:pot_machine`) is what recipes point at.

```json
{
  "type": "cookpot:machine",
  "block": "mypack:pot",
  "menu": "mypack:pot",
  "data_slots": 4,
  "mode": "timed",
  "heat": {
    "offset": [0, -1, 0],
    "sources": [
      "minecraft:campfire[lit=true]",
      { "block": "minecraft:magma_block", "speed": 2.0 },
      "#mypack:heat_sources"
    ]
  },
  "process": {
    "default_time": 200,
    "keep_progress_when_cold": false,
    "lit_property": "lit",
    "fuel": { "slot": "fuel", "burn_times": { "minecraft:coal": 1600, "#minecraft:planks": 300 }, "game_table": true }
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

| Field | Meaning |
|---|---|
| `block` | A block id, or a list. Several blocks may share one machine. |
| `menu`, `data_slots` | The menu type from step 2 and its synced numbers (use 4). |
| `mode` | `timed` (default), `instant` or `staged`. See below. |
| `heat` | Optional. Blocks that power the machine. See "Heat from blocks". |
| `process.default_time` | Ticks a recipe takes when it has no `time` (default 200). |
| `process.keep_progress_when_cold` | Keep progress when unpowered (default: it drains 2 per tick). |
| `process.lit_property` | A block property set to `true` while working, `false` otherwise. |
| `process.fuel` | Optional fuel slot. See "Fuel". |
| `gui.*` | Size, optional `texture` (`<ns>:gui/<name>`), slots, `progress` and `flame` bars. |

**Roles:** `input`, `fuel`, `container`, `output`. A slot without `pos` is placed in rows of three (the built-in
layout), so a machine needs no art to work.

### 4. Recipes: `data/<ns>/recipe/<name>.json`

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

- **Inputs** are shapeless: each needs its own input slot. An input is an item id, a `#tag`, a list (any one of
  them) or `{ "item" | "tag" | "items", "count" }`.
- **`container`**: an item the container slot must hold; one is used up.
- **`result`** (one) or **`results`** (several; they fill output slots, merging into like stacks). A result may carry
  `components`. A recipe whose results do not fit stays paused.
- **`time`** overrides `default_time`. **`charges`** is what a staged machine spends per craft (default 1).
- **`experience`** is read but **not paid out yet**.
- The first matching recipe in load order wins.

## Heat from blocks

`heat.sources` is a list; any matching source powers the machine, the fastest wins. Look at the block at `offset`
from the machine (default `[0, -1, 0]`, directly below; any offset works, such as the side or above).

- `"minecraft:campfire"`: a block id.
- `"minecraft:campfire[lit=true]"`: with block properties that must all match.
- `"#mypack:heat_sources"` or `"#mypack:hot[lit=true]"`: a block tag, so other mods and packs can add heat blocks
  by adding to your tag, with no change to your machine.
- `{ "block": "...", "tag": "...", "properties": { "lit": "true" }, "speed": 2.0 }`: `speed` scales progress
  (1.0 normal, 2.0 twice as fast).

With a heat source present, fuel is not used. With none, the machine falls back to the fuel slot.

## Fuel

`process.fuel.slot` names the fuel slot. `burn_times` maps an item id or `#tag` to ticks (`timed`, `instant`) or
charges (`staged`); an exact item id beats a tag. `game_table: true` also accepts anything the game burns in a
furnace (this uses a private game API, so a Horizon update may break it). A machine with neither `heat` nor `fuel`
always runs.

## Modes

- **`timed`**: furnace-like. Fuel burns for its burn time and power continues across crafts.
- **`instant`**: crafts on the next tick the inputs match (no waiting). It does not wait for a click.
- **`staged`**: brewing-stand-like. Fuel adds charges, each craft spends the recipe's `charges`, and a recipe can
  have several results. A heat source runs it without spending charges.

## Errors

A machine file that does not parse is skipped with a warning in `logs/latest.log` that says why, for example
`two slots are called `in``, or ``slot `x`: role `y` is not input, fuel, container or output``. The game still starts.

## Limits you should know

- The menu cannot restrict which items a slot accepts in this Horizon build. Hoppers can put into any non-output
  slot and take from the output; players can still place anything anywhere.
- Output slots are assumed to hold 64 items, whatever the item's real limit.
- Binding to a **vanilla block** (such as the fletching table) is not possible: no hook can attach slots or saved
  data to one. Add your own block that copies its look instead.
- Content is read once when the game starts. Editing your JSON needs a game restart, **not** a rebuild of Cookpot.
- Clients and servers need the same mods.

## Not yet

Experience payout, sounds, items drawn inside the block, comparator output, recipe viewer support, taking a
container's leftovers (such as an empty bucket) back, fuel remainders.
