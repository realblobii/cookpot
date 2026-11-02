# Cookpot

Cookpot lets [Horizon](https://github.com/horizon-rust/horizon) content mods add **new crafting machines** (a cooking pot, a cutting
board, a kiln, a still) with JSON files only, no Rust. It adds no machines of its own: it is the engine, and your mod brings the
blocks, art and recipes. Other mods can then add recipes, heat sources and fuels to your machines.

- **Three machine kinds:** `timed` (like a furnace), `staged` (brewing-stand style, several outputs) and `instant` (click to craft).
- **Heat and fuel from any blocks you choose:** ids, properties or tags, at any offset (a campfire under a pot).
- **Built-in or custom UI:** pick a ready-made layout, or supply your own texture and slot positions.
- **Experience, sounds and progress bars** per machine and per recipe.
- **Cross-mod machines:** recipes and extensions can target another mod's machine by its id.

```json
{ "type": "cookpot:recipe", "machine": "mypack:pot_machine",
  "inputs": ["minecraft:potato", "minecraft:carrot"], "result": "minecraft:rabbit_stew", "experience": 1.0 }
```

## Documentation

Start at the **[documentation home](docs/README.md)**:

- [Quickstart](docs/QUICKSTART.md): your first machine in five files
- [Authoring reference](docs/AUTHORING.md): every field
- [Examples](docs/EXAMPLES.md): cooking pot, kiln, cutting board, still
- [Add-ons](docs/ADDONS.md): recipes and fuels for another mod's machine
- [Shipping your mod](docs/SHIPPING.md) and [Troubleshooting](docs/TROUBLESHOOTING.md)
- [JSON schemas](schemas/) for editor validation

Working demos: [`testmod/cptest`](testmod/cptest) (a machine) and [`testmod/cpaddon`](testmod/cpaddon) (an add-on that extends it).
Real-world use: [Delightful](https://github.com/realblobii/delightful), a Farmer's Delight-style mod, builds its cooking pot and
cutting board with Cookpot.

## Status

**Untested in the game.** The logic has unit tests, but the game hooks have not yet been run in Horizon. Singleplayer is the target.

## Contributing

`mixins/cookpot/` is the native mod. Design notes, research and build lessons for people working on Cookpot itself are in
[docs/dev/](docs/dev/).
