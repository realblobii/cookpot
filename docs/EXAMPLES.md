# Examples

Machine files only; add a block, a menu with the same slot count, and recipes as in [QUICKSTART.md](QUICKSTART.md).
Slots are listed in order, and the menu's `results` numbers the output slots by that order.
**None of these has been run in the game.**

## Cooking pot (heat from below, fuel as a backup, sounds, XP)

Menu: `{ "slots": 6, "data_slots": 4, "results": [5] }`

```json
{
  "type": "cookpot:machine", "block": "mypack:cooking_pot", "menu": "mypack:cooking_pot", "data_slots": 4,
  "heat": { "sources": [ "minecraft:campfire[lit=true]", "#mypack:heat_sources", { "block": "minecraft:magma_block", "speed": 2.0 } ] },
  "process": { "default_time": 200, "fuel": { "slot": "fuel", "burn_times": { "minecraft:coal": 1600, "#minecraft:planks": 300 } } },
  "sounds": { "working": { "id": "minecraft:block.campfire.crackle", "interval": 40, "volume": 0.5 }, "done": "minecraft:entity.experience_orb.pickup" },
  "gui": { "slots": [
    { "id": "i0", "role": "input" }, { "id": "i1", "role": "input" }, { "id": "i2", "role": "input" },
    { "id": "fuel", "role": "fuel" }, { "id": "bowl", "role": "container" }, { "id": "out", "role": "output" } ],
    "progress": { "pos": [60, 31] }, "flame": { "pos": [30, 46] } }
}
```

## Kiln (fuel only, furnace-style, lit texture)

Menu: `{ "slots": 3, "data_slots": 4, "results": [2] }`. Use a furnace template for the block so its `lit` property works.

```json
{
  "type": "cookpot:machine", "block": "mypack:kiln", "menu": "mypack:kiln", "data_slots": 4,
  "process": { "default_time": 300, "lit_property": "lit", "keep_progress_when_cold": true,
               "fuel": { "slot": "fuel", "burn_times": {}, "game_table": true } },
  "gui": { "slots": [ { "id": "in", "role": "input" }, { "id": "fuel", "role": "fuel" }, { "id": "out", "role": "output" } ],
           "progress": { "pos": [60, 31] }, "flame": { "pos": [30, 46] } }
}
```

`burn_times` may be empty only with `game_table: true`.

## Cutting board (instant, no fuel or heat)

Menu: `{ "slots": 3, "data_slots": 0, "results": [1, 2] }`

```json
{
  "type": "cookpot:machine", "block": "mypack:cutting_board", "menu": "mypack:cutting_board",
  "mode": "instant",
  "sounds": { "done": "minecraft:block.wood.hit" },
  "gui": { "slots": [ { "id": "in", "role": "input" }, { "id": "out0", "role": "output" }, { "id": "out1", "role": "output" } ] }
}
```

A recipe with `"results": [ {"id": "mypack:sliced"}, {"id": "mypack:scraps"} ]` fills both outputs.

## Still (staged: fuel gives charges, one run makes several results)

Menu: `{ "slots": 5, "data_slots": 4, "results": [2, 3, 4] }`

```json
{
  "type": "cookpot:machine", "block": "mypack:still", "menu": "mypack:still", "data_slots": 4,
  "mode": "staged",
  "process": { "default_time": 400, "fuel": { "slot": "fuel", "burn_times": { "minecraft:blaze_powder": 20 } } },
  "gui": { "slots": [ { "id": "in", "role": "input" }, { "id": "fuel", "role": "fuel" },
                      { "id": "o0", "role": "output" }, { "id": "o1", "role": "output" }, { "id": "o2", "role": "output" } ],
           "flame": { "pos": [20, 50], "size": [60, 4] }, "progress": { "pos": [60, 31] } }
}
```

A recipe with `"charges": 4` spends 4 of the 20 charges per craft.

## Stove-fed machine (heat from the side)

```json
"heat": { "offset": [1, 0, 0], "sources": [ "#mypack:stoves[lit=true]" ] }
```
