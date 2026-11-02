# Cookpot — guidance for Claude

You are building **Cookpot**, a **library** native Horizon mod that lets other people define new crafting
machines (a cooking pot, a cutting board, a kiln, ...) with **data files and no Rust**. Cookpot itself adds
no blocks or machines: it ships the engine and the **documentation** (`AUTHORING.md` is a first-class
deliverable). Example machines are demos only.

Decisions already made by the user (do not re-ask):
- Library mod, no machines of its own; examples live in docs or a separate demo mod.
- Machine definitions ship **inside content mods (`.hmod`)**.
- **Singleplayer first**; do not block on multiplayer, but avoid choices that rule it out.
- **If the research shows a required hook is missing, stop and tell the user** with evidence. Do not
  silently fall back or invent a workaround.
- UI: authors may use a **built-in layout** or supply **their own UI** (texture + slot positions).
Read `README.md` for the goal and `docs/BREAKDOWN.md` for the design before doing anything.

## Rule 0: read the docs first, and say what you read

Do not write or edit any code until you have worked through `docs/READING_LIST.md` in order and
written your answers to every question in `docs/OPEN_QUESTIONS.md` (put them in
`docs/FINDINGS.md`, with the URL or file and line each answer came from). The design in
BREAKDOWN.md is a **draft written without having read the hook source**. Parts of it will be
wrong. Fix the draft from what you find, and tell the user what changed.

- Horizon is **unofficial and moves fast** (nightlies, several per day). Docs, hook names and APIs
  change. Never rely on memory or on this folder for a hook name, signature, or file format.
  Look it up at the **pinned commit** (below).
- The docs contradict each other in places (for example `docs/api/README.md` said shader packs were
  "removed" while the game had a working shader options UI). When docs and source disagree, the
  **source at the pinned commit wins**. Say so when you find one.
- Never claim something works unless it was built and run in the game. This project's loop is slow
  (see "Build loop"), so state clearly what is untested.

## Environment (as of 2026-11-02, re-verify)

- Linux, shell is **fish** (`for ... in` loops and globbing differ from bash; use `python3 -I` or
  `bash -c` for anything fancy). A failed glob in fish aborts the command.
- Game folder used for testing: `~/.local/share/horizon/instances/try/` (has `options.txt`,
  `horizon-native-mods/`, `horizon-mods.txt`, `logs/latest.log`, `shaderpacks/`, `crash-reports/`).
- Launcher: `/usr/bin/horizon-launcher`. Its native build log:
  `~/.local/share/horizon-launcher/native/build.log`. Read it first when a build fails.
- Find the game's exact version from the top of `logs/latest.log`:
  `Horizon Nightly 2026-11-01 · c9d474d`. A native mod pins **one full commit hash**; get it with
  `git -C <horizon checkout> rev-parse <short>`. Check this at the start of every session: the user's
  nightly will have moved on, and the pin must match what the launcher builds.
- Docs repo and source: `https://github.com/horizon-rust/horizon` (clone it; `git archive <commit> |
  tar -x -C <dir>` gives a read-only copy of the exact pinned source to grep). Public docs:
  `https://playhorizon.dev/docs/...`. Raw docs: `https://raw.githubusercontent.com/horizon-rust/horizon/main/docs/<file>.md`.
  Note `WebFetch` summarizes with a small model and truncates long pages (it said so for
  `native-mods.md`: 236k chars, only the first 100k read). **Use `offset` to read the rest**, or clone
  the repo and read files directly. Prefer reading real files over summaries for anything you will
  code against.
- A working reference mod from the previous session: `~/horizon-slider-mod/shader_sliders/`
  (built and run in the game by the user). Copy its layout, `tools/package.py` and bundle structure.
  Clones of `horizon` and `horizon-mixins` (pinned) are in `~/horizon-slider-mod/`.

## Mixin rules learned the hard way (horizon-mixins 0.6.0)

Native mods are source patches (`horizon-mixin.toml` + helper `.rs`) applied by `horizon-mixins`
and compiled into the game. See `docs/LESSONS.md` for the full list. The big ones:

1. **`OVERWRITE`, `RETURN` and other body-rewriting hooks reject any target function containing a
   closure, `?`, async, or unsafe/const block** (`E_UNSUPPORTED_CONTROL_FLOW`). Only **`HEAD`**
   (including `cancellable = true`, which returns `ControlFlow<R>`) tolerates those. Many game
   functions contain closures, so plan on cancelling `HEAD` hooks.
2. A helper file is **exactly one** private function. Pure, testable logic goes in the `[[module]]`
   `runtime.rs`, which is injected as `pub(crate) mod <name>` into a source file that has **no
   top-level glob imports**.
3. Hooks are compiled in the target's own module: private fields and functions are visible.
4. Bundle crates (`[[crate]]`) can only use dependencies already in Horizon's `Cargo.lock`.
5. Bump `version` in `horizon-mixin.toml` **and** `Cargo.toml` on every repackage, or the launcher
   may not rebuild.

## Build loop

1. Edit the bundle under `mixins/cookpot/`.
2. `python3 tools/package.py` writes `target/<id>-<version>.hnmod` (source only).
3. Copy it into `~/.local/share/horizon/instances/try/horizon-native-mods/` (delete older versions).
4. In the launcher: Settings > Native mods (experimental) > **Build**. First build is slow
   (docs say 20-30 min; the real time on this machine was not measured), later ones faster. Do not restart a running build.
5. Enable the mod in the game under Options > Mods. Read `logs/latest.log` and
   `crash-reports/` after testing.

The auto-mode permission classifier **blocked running the `horizon-mixins` CLI and its python
staging script locally** ("code from external"). The launcher builds the mod itself, so the loop
above works without it. If you need the local `plan`/`check` loop, ask the user to run it with
`! <command>` or to add a permission rule. Do not try to get around the block.

## Working with the user

- The user tests in the game and reports back with screenshots and build-log errors. Keep each
  iteration small so a failed build costs little. Tell them exactly what to press and what to look for.
- Report outcomes plainly: built / not built, tested / untested.
- Do not modify their packs, options or saves without saying so, and keep backups of anything you edit.
- Do not publish anything or contact external services without asking.
