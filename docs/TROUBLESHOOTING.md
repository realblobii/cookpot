# Troubleshooting

Always start with `logs/latest.log` and search for `cookpot`. For a failed Cookpot build, read
`~/.local/share/horizon-launcher/native/build.log`. **Nothing here has been run in the game**, so some entries are
predictions.

## Log messages

| Message (shortened) | Meaning and fix |
|---|---|
| `block must be a block id like mymod:pot` | `block` is missing or has no namespace. |
| `menu must be a menu type id` | `menu` is missing or has no namespace. |
| `gui.slots needs 1 to 256 slots` | No slots, or too many. |
| `two slots are called X` | Slot `id`s must be unique. |
| `slot X: role Y is not input, fuel, container or output` | Fix the role spelling. |
| `mode X is not timed, instant or staged` | Fix `mode`. |
| `process.fuel.slot X is not a slot` | The fuel slot name must be a slot `id`. |
| `process.fuel needs burn_times or game_table: true` | Give a fuel list or turn on the game's table. |
| `sounds.X: Y is not a sound id` | Sound ids are `namespace:path`, lowercase. |
| `heat.sources is empty` / `a heat source needs a block or a tag` | Fix the source list. |
| `recipe X is for machine Y, which is not installed` | The machine's mod is missing or the id is misspelled. A machine id is `<ns>:<file name>`. |
| `extension X: machine Y has no fuel slot to add burn times to` | The base machine has no `process.fuel`. Use heat sources or ask its author. |
| `result X is not an item the game knows` | Misspelled item id in a recipe `result`. |

## Symptoms

**The block does nothing / no menu.** The menu's `slots` must equal the number of slots in the machine file, or the game
drops the menu. The block's template must have a block entity (barrel, chest, furnace).

**The machine opens but never crafts.** Check, in order: the recipe's `machine` matches the machine's id exactly; the inputs
are in input-role slots and counts are met; a container item is in a `container` slot; the output slot has room; and the
machine has power (a heat block at the right offset with matching properties, or fuel in the fuel slot).

**Heat block is ignored.** `campfire[lit=true]` needs the block's `lit` property to be exactly `true`. Look at the offset: the
default is directly below. A tag source needs the tag file to exist (`data/<ns>/tags/block/<name>.json`).

**Bars do not move.** The menu and machine must both say `data_slots: 4`.

**No sound.** Sounds play for the local player only; a LAN guest may hear nothing. Custom sound ids need
`assets/<ns>/sounds.json` entries. Check the id in the log.

**Edits to JSON have no effect.** Content is read once at game start: restart the game. Cookpot itself only needs a rebuild
when its own source changes.

**The native build fails.** Read the launcher's `build.log`. Cookpot's code was written without being compiled against
Horizon, so first builds may fail; send the error text.
