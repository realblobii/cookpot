# Add-ons: using machines made by other mods

Cookpot's machines are shared. Any mod can add recipes to, or extend, a machine another mod made, with no code and
no Cookpot-specific dependency field. This is how an add-on mod can grow a base mod's cooking pot.

> Written from the source, **not run in the game**. The demo is `testmod/cpaddon/` (needs `testmod/cptest/`).

## The machine id

A machine's id is its file's id: `<namespace>:<file name without .json>`. The base mod's file
`data/base/recipe/cooking_pot.json` makes the machine **`base:cooking_pot`**. Base mod authors: treat that id as public
API and do not rename it, since add-ons point at it. (The block id may change; the machine id is what add-ons use.)

## 1. Add recipes to someone else's machine

In your own mod, write an ordinary recipe and put the other mod's machine id in `machine`:

`data/addon/recipe/mash.json`

```json
{
  "type": "cookpot:recipe",
  "machine": "base:cooking_pot",
  "inputs": [ { "item": "minecraft:baked_potato", "count": 2 }, "minecraft:egg" ],
  "container": "minecraft:bowl",
  "result": "minecraft:mushroom_stew",
  "experience": 0.5
}
```

Your recipe may use your own items and tags freely. If the base mod is not installed, Cookpot logs
`recipe addon:mash is for machine base:cooking_pot, which is not installed` and ignores it; the game still starts.

## 2. Add heat blocks and fuels

Use an **extension** when your mod adds a stove, a new fuel or a heat block the base machine should accept:

`data/addon/recipe/cooking_pot_extras.json`

```json
{
  "type": "cookpot:extension",
  "machine": "base:cooking_pot",
  "heat_sources": [
    "addon:stove[lit=true]",
    { "block": "addon:forge", "properties": { "lit": true }, "speed": 1.5 }
  ],
  "burn_times": { "addon:charcoal_briquette": 3200, "#addon:fuels": 800 }
}
```

- `heat_sources` use the same forms as a machine's `heat.sources`, and join the base machine's list. If the base
  machine has no `heat` block yet, it gets one that looks directly below.
- `burn_times` join the machine's fuels. The machine must already have a fuel slot or the extension is rejected with a
  warning. The base mod's own entries win for the same item.
- Extensions apply after every machine is read, so the order mods load in does not matter.

**Prefer tags for base mods.** If the base mod lists `#base:heat_sources` as a heat source, an add-on can instead add its
block to `data/base/tags/block/heat_sources.json`. No extension is needed and it also helps every other add-on.

## 3. Share your own machine

If you make a machine other mods should build on:

1. Keep its file name stable (`data/<ns>/recipe/<name>.json`) and say its id in your mod description.
2. Use tags for heat sources and, where it makes sense, fuels.
3. Put ingredients' categories in tags (`#base:vegetables`) so add-ons can add items to them.
4. List any `lit_property` or slot ids you promise to keep.

## Checking it worked

Start a world and open `logs/latest.log`. Search for `cookpot:`. A line per problem is written; no lines means every
recipe and extension found its machine.

## What add-ons cannot do

An add-on cannot add or remove slots, change the screen, or change a machine's mode. Ask the base author, or make your
own machine. Overriding a base recipe is not supported: use a different recipe with a more specific input list, since
the first matching recipe in load order wins.
