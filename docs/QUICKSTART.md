# Quickstart: your first machine

You need: Cookpot installed (a native mod, built once by the launcher), and a content mod folder. Everything below is
JSON. Replace `mypack` with your mod id. A larger finished example is [Delightful](https://github.com/realblobii/delightful).

> Nothing here has been run in the game yet; if a step fails, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

## 1. Make a mod

`horizon-mod.toml`:

```toml
id = "mypack"
name = "My Pack"
version = "0.1.0"
api = "0.2"
side = "both"
capabilities = ["registries"]
```

## 2. The block

`data/mypack/horizon/block/pot.json`

```json
{ "template": "minecraft:barrel", "item": true }
```

Add `assets/mypack/lang/en_us.json` with `{ "block.mypack.pot": "Cooking Pot" }`.

## 3. The menu

`data/mypack/horizon/menu/pot.json` (two inputs, one fuel, one output, so 4 slots; slot 3 is the result)

```json
{ "slots": 4, "data_slots": 4, "results": [3] }
```

## 4. The machine

`data/mypack/recipe/pot_machine.json` (its id is `mypack:pot_machine`)

```json
{
  "type": "cookpot:machine",
  "block": "mypack:pot",
  "menu": "mypack:pot",
  "data_slots": 4,
  "heat": { "sources": [ "minecraft:campfire[lit=true]" ] },
  "process": { "default_time": 200, "fuel": { "slot": "fuel", "burn_times": { "minecraft:coal": 1600 } } },
  "gui": {
    "slots": [
      { "id": "a", "role": "input" },
      { "id": "b", "role": "input" },
      { "id": "fuel", "role": "fuel" },
      { "id": "out", "role": "output" }
    ],
    "progress": { "pos": [60, 31] },
    "flame": { "pos": [30, 46] }
  }
}
```

No `pos` on the slots means the built-in layout, so you need no GUI art.

## 5. A recipe

`data/mypack/recipe/stew.json`

```json
{
  "type": "cookpot:recipe",
  "machine": "mypack:pot_machine",
  "inputs": [ "minecraft:potato", "minecraft:carrot" ],
  "result": "minecraft:rabbit_stew",
  "experience": 1.0
}
```

## 6. Try it

1. Zip the mod folder's contents (not the folder itself) as `mypack-0.1.0.hmod`, put it in `horizon-mods/`.
2. Enable Cookpot and your mod in Options > Mods, and start a world.
3. Place the pot with a lit campfire **underneath**, put a potato and a carrot in, and wait 10 seconds.

Next: add sounds, a custom screen, or more machines in [AUTHORING.md](AUTHORING.md), or let other mods add recipes
to yours in [ADDONS.md](ADDONS.md).
