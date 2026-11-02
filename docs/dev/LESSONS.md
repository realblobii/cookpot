# Lessons from building the Shader Sliders mod (2026-11-02)

Source: `~/horizon-slider-mod/shader_sliders/`. It hooked `OptionsUi::click` and a row builder in
`crates/horizon-client-render/src/options_ui.rs` and was built and run by the launcher.

## Tooling

- A native mod is a bundle: `horizon-mixin.toml` + helper `.rs` files. `tools/package.py` (from
  `horizon-native-mod-template`) zips it to `<id>-<version>.hnmod` with a `horizon-native.json`
  (`schema`, `id`, `version`, `game`, `horizon` = pinned commit, `mixins` = mixins commit).
- The template pins an old Nightly. **Check the pin against the user's game** (top of
  `logs/latest.log`). The template's pin (2026-10-30) had no shader-pack code at all; the user's game
  (2026-11-01, `c9d474d`) did. A wrong pin means hooks target code that does not exist.
- `compatibility.json` in the template also holds `source_sha256` of target files; only the local
  staging script uses it. Not checked whether the launcher reads it.
- The launcher installs its own `horizon-mixins` (a git rev) and Rust. Its failures print the structured
  diagnostic (code, file, `injection_id`, `target`) in `native/build.log`. **The `injection_id` tells you which
  hook failed.** The on-screen error hides it; read the log.
- The first build compiled about 290 crates; `horizon-client-render` then `horizon-client` (the largest, one
  core) dominate. Rebuilds after changing the mod recompile those two plus dependents.
- Pure logic in `runtime.rs` (module name injected as `crate::<file module>::<name>`) can be unit-tested
  standalone with `cargo test` because it has no game types.

## horizon-mixins constraints

- `E_UNSUPPORTED_CONTROL_FLOW`: `OVERWRITE` and `RETURN` (and other body-rewriting kinds) reject target
  functions that contain closures, `?`, async, unsafe or const blocks. We hit it twice: an `OVERWRITE` on a
  function with a closure, and a `RETURN` on a function using `.is_some_and(|c| ...)`.
  **Fix: use `HEAD`, with `cancellable = true` to replace a result:**
  ```toml
  [[injection]]
  id = "rows"
  kind = "HEAD"
  cancellable = true
  target = "horizon_client_render::options_ui::native_pack_option_rows"
  helper = "rows.rs"
  bindings = [{ name = "s", ty = "&Settings", value = "s" }]
  ```
  ```rust
  fn rows(s: &Settings) -> std::ops::ControlFlow<Vec<Row>> {
      std::ops::ControlFlow::Break(build(s)) // Break(value) skips the original body
  }
  ```
- Target ids are `crate_name::module::Type::method` (hyphens become underscores, lifetimes `'_`).
- `self` binds as `{ name = "this", ty = "&mut OptionsUi", value = "&mut *self" }`; other params bind by
  name; `bindings = []` is valid.
- Because hooks compile inside the target module, they can call that module's private functions and read
  private fields. That is how the sliders mod reached `layout()`, `shader_dragging`, `Widget`, `WKind`.
- A HEAD hook cannot see the function's result. To act "after" something, stash state in a static (a
  `Mutex<Option<Box<dyn Any + Send>>>` in `runtime.rs`) and consume it from another hook (we saved on mouse
  release from a stash written during drag).
- Per-instance state: see `docs/effects-and-state.md` in horizon-mixins (`[[state]]`).

## Game facts verified in source at `c9d474d`

- Options UI is `crates/horizon-client-render/src/options_ui.rs` (7k lines). Game folder = working
  directory; shader packs live in `<game dir>/shaderpacks/`, values in `shaderpacks/<pack>.txt`.
- Horizon-native shader packs (`horizon-shaders.toml`) only got click-to-cycle options; OptiFine-style
  packs got sliders. The mod bridged them.
- The game's own handling of a slider drag only updated the handle when `shader_options` was `Some`
  (OptiFine packs). Mismatches like this between two code paths are common: read both.

## Process

- Documentation was incomplete and sometimes stale. Each doc page said "see page X" for the detail. The
  source (`native-hooks.json`, the crates) was the reliable reference.
- The user's feedback loop is a screenshot plus the launcher's error. Keep changes small and say what to
  look for after each build.
- Subjective tuning (fog, brightness) was done by editing the user's pack config; back files up first.
