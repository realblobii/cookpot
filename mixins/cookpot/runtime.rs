// Cookpot's pure logic: machine definitions read from content mods' `cookpot:machine` recipe files.
// No game types, so it is also a plain crate (`cargo test` at the repository root).
use std::sync::{Arc, Mutex};

/// The recipe `type` of a machine definition file (`data/<ns>/recipe/<name>.json`).
pub const MACHINE_TYPE: &str = "cookpot:machine";
/// The block entity id of every machine.
pub const ENTITY_ID: &str = "cookpot:machine";
/// Phase 1 only: the progress bar fills over this many ticks while the first slot holds an item.
pub const TEST_TOTAL: i32 = 200;

const MAX_SLOTS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Input,
    Fuel,
    Container,
    Output,
}

impl Role {
    pub fn accepts_items(self) -> bool {
        self != Role::Output
    }
    pub fn is_output(self) -> bool {
        self == Role::Output
    }
    fn parse(name: &str) -> Option<Role> {
        match name {
            "input" => Some(Role::Input),
            "fuel" => Some(Role::Fuel),
            "container" => Some(Role::Container),
            "output" => Some(Role::Output),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub id: String,
    pub role: Role,
    /// Top left of the slot inside the screen, GUI pixels.
    pub pos: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub pos: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Machine {
    /// The definition file's id (`<ns>:<name>`).
    pub id: String,
    pub blocks: Vec<String>,
    /// The menu type the container opens; its slot count must equal `slots.len()`.
    pub menu: String,
    pub data_slots: usize,
    pub slots: Vec<Slot>,
    pub width: f32,
    pub height: f32,
    pub inventory_y: f32,
    /// A content mod's GUI texture (`<ns>:gui/<name>`), drawn at the screen's top left.
    pub texture: Option<String>,
    pub progress: Option<Bar>,
}

impl Machine {
    /// The numbers of the slots whose role passes `keep`, in slot order.
    pub fn slot_indices(&self, keep: impl Fn(Role) -> bool) -> Vec<usize> {
        (0..self.slots.len()).filter(|&i| keep(self.slots[i].role)).collect()
    }
}

fn number(v: &serde_json::Value) -> Option<f32> {
    v.as_f64().filter(|n| n.is_finite()).map(|n| n as f32)
}

fn pair(v: Option<&serde_json::Value>) -> Option<[f32; 2]> {
    let a = v?.as_array()?;
    match a.as_slice() {
        [x, y] => Some([number(x)?, number(y)?]),
        _ => None,
    }
}

/// Reads one machine definition. The error names what is wrong, for the log.
pub fn parse(id: &str, json: &serde_json::Value) -> Result<Machine, String> {
    let text = |key: &str| json.get(key).and_then(|v| v.as_str());
    let blocks: Vec<String> = match json.get("block") {
        Some(serde_json::Value::String(b)) => vec![b.clone()],
        Some(serde_json::Value::Array(list)) => list.iter().filter_map(|b| b.as_str()).map(str::to_owned).collect(),
        _ => Vec::new(),
    };
    if blocks.is_empty() || blocks.iter().any(|b| !b.contains(':')) {
        return Err("`block` must be a block id like `mymod:pot` (or a list of them)".into());
    }
    let menu = text("menu").filter(|m| m.contains(':')).ok_or("`menu` must be a menu type id like `mymod:pot`")?;
    let gui = json.get("gui");
    let list = gui
        .and_then(|g| g.get("slots"))
        .and_then(|s| s.as_array())
        .ok_or("`gui.slots` must be a list of slots")?;
    if list.is_empty() || list.len() > MAX_SLOTS {
        return Err(format!("`gui.slots` needs 1 to {MAX_SLOTS} slots, it has {}", list.len()));
    }
    let width = gui.and_then(|g| g.get("size")).and_then(|s| pair(Some(s))).map_or(176.0, |s| s[0]);
    let height = gui.and_then(|g| g.get("size")).and_then(|s| pair(Some(s))).map_or(166.0, |s| s[1]);
    // Slots without a `pos` go in rows of three, centred; that is the built-in layout.
    let columns = 3.0_f32;
    let mut slots: Vec<Slot> = Vec::new();
    for (i, s) in list.iter().enumerate() {
        let sid = s.get("id").and_then(|v| v.as_str()).ok_or(format!("slot {i} has no `id`"))?;
        if slots.iter().any(|o| o.id == sid) {
            return Err(format!("two slots are called `{sid}`"));
        }
        let role_name = s.get("role").and_then(|v| v.as_str()).ok_or(format!("slot `{sid}` has no `role`"))?;
        let role = Role::parse(role_name)
            .ok_or(format!("slot `{sid}`: role `{role_name}` is not input, fuel, container or output"))?;
        let pos = match s.get("pos") {
            Some(p) => pair(Some(p)).ok_or(format!("slot `{sid}`: `pos` must be [x, y]"))?,
            None => {
                let (col, row) = ((i as f32) % columns, (i as f32 / columns).floor());
                [(width - columns * 18.0) / 2.0 + col * 18.0, 18.0 + row * 18.0]
            }
        };
        slots.push(Slot { id: sid.to_owned(), role, pos });
    }
    let progress = match gui.and_then(|g| g.get("progress")) {
        None => None,
        Some(p) => Some(Bar {
            pos: pair(p.get("pos")).ok_or("`gui.progress.pos` must be [x, y]")?,
            size: pair(p.get("size")).unwrap_or([24.0, 4.0]),
        }),
    };
    let data_slots = json.get("data_slots").and_then(|v| v.as_u64()).unwrap_or(0).min(MAX_SLOTS as u64) as usize;
    Ok(Machine {
        id: id.to_owned(),
        blocks,
        menu: menu.to_owned(),
        data_slots,
        slots,
        width,
        height,
        inventory_y: height - 82.0,
        texture: gui.and_then(|g| g.get("texture")).and_then(|t| t.as_str()).map(str::to_owned),
        progress,
    })
}

struct Registry {
    loaded: bool,
    machines: Vec<Arc<Machine>>,
}

static REGISTRY: Mutex<Registry> = Mutex::new(Registry { loaded: false, machines: Vec::new() });

/// The machine whose `block` is `block`. `files` lists the `cookpot:machine` files (id and JSON) and is
/// only called until some are found: it is empty outside a world, and the content mods never change
/// while the game runs. A file that does not parse is skipped with a warning in the log.
pub fn machine_for(
    block: &str,
    files: impl FnOnce() -> Vec<(String, serde_json::Value)>,
) -> Option<Arc<Machine>> {
    let mut reg = REGISTRY.lock().ok()?;
    if !reg.loaded {
        let files = files();
        if files.is_empty() {
            return None;
        }
        for (id, json) in files {
            match parse(&id, &json) {
                Ok(m) => reg.machines.push(Arc::new(m)),
                Err(e) => log::warn!("cookpot: {id}: {e}"),
            }
        }
        reg.loaded = true;
    }
    reg.machines.iter().find(|m| m.blocks.iter().any(|b| b == block)).cloned()
}

/// One tick of the Phase 1 test process: progress rises while `active`, and starts over at `total`.
pub fn advance(progress: i32, active: bool, total: i32) -> i32 {
    if !active || total <= 0 {
        return 0;
    }
    if progress + 1 >= total { 0 } else { progress.max(0) + 1 }
}

/// The values for the menu's data slots: progress and total, then zeros up to `data_slots`.
/// They are 16-bit on the wire, so a larger value is clamped.
pub fn data_slots(m: &Machine, progress: i32, total: i32) -> Vec<i32> {
    let mut out = vec![0; m.data_slots];
    for (slot, value) in out.iter_mut().zip([progress, total]) {
        *slot = value.clamp(0, i16::MAX as i32);
    }
    out
}

/// `value` of `total` as 0 to 1.
pub fn fraction(value: i32, total: i32) -> f32 {
    if total <= 0 { 0.0 } else { (value as f32 / total as f32).clamp(0.0, 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn def() -> serde_json::Value {
        json!({
            "type": "cookpot:machine", "block": "t:pot", "menu": "t:pot", "data_slots": 2,
            "gui": { "slots": [
                { "id": "in", "role": "input" },
                { "id": "out", "role": "output", "pos": [100, 30] } ],
              "progress": { "pos": [60, 30] } }
        })
    }

    #[test]
    fn parses_a_definition_with_default_layout() {
        let m = parse("t:pot_machine", &def()).unwrap();
        assert_eq!(m.slots.len(), 2);
        assert_eq!(m.slots[0].pos, [(176.0 - 54.0) / 2.0, 18.0]);
        assert_eq!(m.slots[1].pos, [100.0, 30.0]);
        assert_eq!(m.slot_indices(Role::is_output), vec![1]);
        assert_eq!(m.slot_indices(Role::accepts_items), vec![0]);
        assert_eq!(m.inventory_y, 84.0);
        assert_eq!(m.progress.as_ref().unwrap().size, [24.0, 4.0]);
    }

    #[test]
    fn rejects_bad_definitions_with_a_reason() {
        let mut d = def();
        d["block"] = json!("pot");
        assert!(parse("x", &d).unwrap_err().contains("block"));
        let mut d = def();
        d["gui"]["slots"][1]["id"] = json!("in");
        assert!(parse("x", &d).unwrap_err().contains("two slots"));
        let mut d = def();
        d["gui"]["slots"][0]["role"] = json!("cauldron");
        assert!(parse("x", &d).unwrap_err().contains("cauldron"));
        let mut d = def();
        d["gui"]["slots"] = json!([]);
        assert!(parse("x", &d).is_err());
    }

    #[test]
    fn progress_runs_and_resets() {
        assert_eq!(advance(0, true, 3), 1);
        assert_eq!(advance(1, true, 3), 2);
        assert_eq!(advance(2, true, 3), 0);
        assert_eq!(advance(2, false, 3), 0);
    }

    #[test]
    fn data_slots_are_padded_and_clamped() {
        let m = parse("x", &def()).unwrap();
        assert_eq!(data_slots(&m, 5, 200), vec![5, 200]);
        let mut one = m.clone();
        one.data_slots = 1;
        assert_eq!(data_slots(&one, 99999, 200), vec![i16::MAX as i32]);
        one.data_slots = 4;
        assert_eq!(data_slots(&one, 1, 2), vec![1, 2, 0, 0]);
    }

    #[test]
    fn registry_finds_machines_once() {
        let f = || vec![("t:pot_machine".to_owned(), def()), ("t:bad".to_owned(), json!({}))];
        assert!(machine_for("t:pot", f).is_some());
        // Loaded: the file list is not asked again.
        assert!(machine_for("t:other", || panic!("asked again")).is_none());
        assert!(machine_for("t:pot", || panic!("asked again")).is_some());
    }
}
