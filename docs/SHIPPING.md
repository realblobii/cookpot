# Shipping a mod that uses Cookpot

1. Put machine, recipe and extension files in your content mod (`.hmod`) at `data/<namespace>/recipe/<name>.json` with `type`
   `cookpot:machine`, `cookpot:recipe` or `cookpot:extension`. The machine id is the machine file's id.
2. Declare the machine's block with a template that has a block entity (`minecraft:barrel`, `chest` or `furnace`), as in the
   [Quickstart](QUICKSTART.md).
3. **Tell players Cookpot must be installed** and enabled under Options > Mods. Horizon's manifest has no documented dependency
   field, so say it in your description and README. Without Cookpot the block exists but does nothing.
4. Add-ons that extend another mod's machine only need that machine's id: see [Add-ons](ADDONS.md). Treat published machine ids as public
   API and never rename them.
5. Content is read once at game start: JSON edits need a game restart, not a Cookpot rebuild.
6. If something fails, search `logs/latest.log` for `cookpot` and read [Troubleshooting](TROUBLESHOOTING.md). The demos in
   [`testmod/`](../testmod) tell a Cookpot fault from a pack fault.
7. Check your files against the [schemas](../schemas/) in your editor.
