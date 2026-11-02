# Cookpot documentation

Cookpot lets you add new crafting machines (a cooking pot, a kiln, a cutting board, a still) to Horizon with
JSON files only, and lets other mods add recipes and fuels to your machines.

> **Status:** written from the source, **not yet run in the game**. Each page says what is untested.

## For mod authors

| Page | Read it when |
|---|---|
| [Quickstart](QUICKSTART.md) | You want a working machine in five small files. Start here. |
| [Authoring reference](AUTHORING.md) | You need every field: machine, menu, recipes, heat, fuel, modes, sounds, XP. |
| [Examples](EXAMPLES.md) | You want to copy a cooking pot, kiln, cutting board, still or stove-fed machine. |
| [Add-ons](ADDONS.md) | You are adding recipes, heat blocks or fuels to a machine **another mod** made, or sharing yours. |
| [Shipping](SHIPPING.md) | You are releasing a mod that needs Cookpot. |
| [Troubleshooting](TROUBLESHOOTING.md) | Something does not work, or the log shows a `cookpot` warning. |
| [JSON schemas](../schemas/) | You want your editor to check your files (`machine`, `recipe`, `extension`). |

Working demos: [`testmod/cptest`](../testmod/cptest) (a machine) and [`testmod/cpaddon`](../testmod/cpaddon)
(an add-on that extends it).

## Which file do I write?

| I want to... | Write | `type` |
|---|---|---|
| Make a new machine | a machine file | `cookpot:machine` |
| Add a recipe to any machine | a recipe file | `cookpot:recipe` |
| Add heat blocks or fuels to another mod's machine | an extension file | `cookpot:extension` |

## For people working on Cookpot itself

| Page | Contents |
|---|---|
| [Findings](dev/FINDINGS.md) | Research answers with sources, and the list of untested assumptions. |
| [Design breakdown](dev/BREAKDOWN.md) | Design, phases and acceptance tests. |
| [Open questions](dev/OPEN_QUESTIONS.md), [Reading list](dev/READING_LIST.md) | The research brief. |
| [Lessons](dev/LESSONS.md) | Build and tooling lessons. |
