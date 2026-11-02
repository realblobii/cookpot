// Cookpot's pure logic: machine definitions read from content mods' `cookpot:machine` recipe files.
// No game types, so it is also a plain crate (`cargo test` at the repository root).
use std::sync::{Arc, Mutex};

/// The recipe `type` of a machine definition file (`data/<ns>/recipe/<name>.json`).
pub const MACHINE_TYPE: &str = "cookpot:machine";
/// The block entity id of every machine.
pub const ENTITY_ID: &str = "cookpot:machine";
/// The `type` of a file that adds heat sources or fuels to a machine another mod made.
pub const EXTENSION_TYPE: &str = "cookpot:extension";
/// Every file type Cookpot reads that describes a machine (definitions, then extensions).
pub const FILE_TYPES: [&str; 2] = [MACHINE_TYPE, EXTENSION_TYPE];
/// The recipe `type` of a machine's recipe file (`data/<ns>/recipe/<name>.json`).
pub const RECIPE_TYPE: &str = "cookpot:recipe";
/// The most items one output slot holds. The game's real per-item limit is not consulted.
pub const STACK_MAX: u32 = 64;

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
    /// Burn or charge bar (`gui.flame`), drawn from data slots 2 and 3.
    pub flame: Option<Bar>,
    pub mode: Mode,
    /// Ticks a recipe takes when it names no `time`.
    pub default_time: i32,
    /// Where heat can come from (blocks near the machine). None: no block heat.
    pub heat: Option<Heat>,
    pub fuel: Option<Fuel>,
    /// Progress is kept (not lost) while the machine has no heat or fuel.
    pub keep_progress: bool,
    /// A block property (such as `lit`) set to `true` while the machine works.
    pub lit_property: Option<String>,
    pub sounds: Sounds,
}

/// One sound a machine makes.
#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
    /// A sound event id: a vanilla one or a mod's own (`mypack:pot_bubble`).
    pub id: String,
    pub volume: f32,
    pub pitch: f32,
    /// For the working sound: ticks of progress between plays.
    pub interval: i32,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Sounds {
    /// When a craft begins.
    pub start: Option<Sound>,
    /// Repeatedly while a craft runs.
    pub working: Option<Sound>,
    /// When a craft finishes.
    pub done: Option<Sound>,
}

/// Which of a machine's sounds to play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Start,
    Working,
    Done,
}

impl Sounds {
    pub fn get(&self, cue: Cue) -> Option<&Sound> {
        match cue {
            Cue::Start => self.start.as_ref(),
            Cue::Working => self.working.as_ref(),
            Cue::Done => self.done.as_ref(),
        }
    }
}

/// Whether `id` is a sound event id the game accepts: `namespace:path`, lowercase.
pub fn valid_sound_id(id: &str) -> bool {
    let Some((ns, path)) = id.split_once(':') else { return false };
    let ok = |c: char, slash: bool| c.is_ascii_lowercase() || c.is_ascii_digit() || "_.-".contains(c) || (slash && c == '/');
    !ns.is_empty() && !path.is_empty() && ns.chars().all(|c| ok(c, false)) && path.chars().all(|c| ok(c, true))
}

/// A sound: `"ns:event"` or `{ "id", "volume", "pitch", "interval" }`.
fn parse_sound(what: &str, v: &serde_json::Value) -> Result<Sound, String> {
    let (id, volume, pitch, interval) = match v {
        serde_json::Value::String(s) => (s.as_str(), 1.0, 1.0, 40),
        serde_json::Value::Object(_) => (
            v.get("id").and_then(|x| x.as_str()).ok_or(format!("`sounds.{what}` needs an `id`"))?,
            v.get("volume").and_then(number).unwrap_or(1.0),
            v.get("pitch").and_then(number).unwrap_or(1.0),
            v.get("interval").and_then(|x| x.as_i64()).unwrap_or(40).clamp(1, 1_000_000) as i32,
        ),
        _ => return Err(format!("`sounds.{what}` is a sound id or an object")),
    };
    if !valid_sound_id(id) {
        return Err(format!("`sounds.{what}`: `{id}` is not a sound id like `minecraft:block.campfire.crackle`"));
    }
    if !(0.0..=16.0).contains(&volume) || !(0.0..=4.0).contains(&pitch) {
        return Err(format!("`sounds.{what}`: volume must be 0 to 16 and pitch 0 to 4"));
    }
    Ok(Sound { id: id.to_owned(), volume, pitch, interval })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Furnace-like: progress over time, fuel burns for a while.
    Timed,
    /// Crafts on the next tick it can: no progress bar.
    Instant,
    /// Fuel gives charges, each craft spends them, and a recipe may give several results.
    Staged,
}

/// One kind of block that heats the machine.
#[derive(Clone, Debug, PartialEq)]
pub struct HeatSource {
    pub block: Option<String>,
    /// A block tag, without the `#`.
    pub tag: Option<String>,
    /// Block properties that must all match, such as `lit=true`.
    pub properties: Vec<(String, String)>,
    /// How fast progress runs on this heat (1.0 is normal).
    pub speed: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Heat {
    /// The block position to look at, relative to the machine (default: directly below).
    pub offset: [i32; 3],
    pub sources: Vec<HeatSource>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fuel {
    /// Index of the slot that holds fuel.
    pub slot: usize,
    /// Item id or `#tag` (tag key without `#`) and its burn time (Timed) or charges (Staged).
    pub burn_times: Vec<(String, i32)>,
    /// Also use the game's own fuel table for items not listed. Relies on a private game API.
    pub game_table: bool,
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
    let mode = match json.get("mode").and_then(|v| v.as_str()) {
        None | Some("timed") => Mode::Timed,
        Some("instant") => Mode::Instant,
        Some("staged") => Mode::Staged,
        Some(other) => return Err(format!("`mode` `{other}` is not timed, instant or staged")),
    };
    let process = json.get("process");
    let default_time = process.and_then(|p| p.get("default_time")).and_then(|v| v.as_i64()).unwrap_or(200).clamp(0, 1_000_000) as i32;
    let keep_progress = process.and_then(|p| p.get("keep_progress_when_cold")).and_then(|v| v.as_bool()).unwrap_or(false);
    let lit_property = process.and_then(|p| p.get("lit_property")).and_then(|v| v.as_str()).map(str::to_owned);
    let heat = match json.get("heat") {
        None => None,
        Some(h) => Some(parse_heat(h)?),
    };
    let fuel = match process.and_then(|p| p.get("fuel")) {
        None => None,
        Some(f) => {
            let slot_name = f.get("slot").and_then(|v| v.as_str()).ok_or("`process.fuel.slot` must name a slot")?;
            let slot = slots
                .iter()
                .position(|s| s.id == slot_name)
                .ok_or(format!("`process.fuel.slot` `{slot_name}` is not a slot"))?;
            let mut burn_times = Vec::new();
            if let Some(map) = f.get("burn_times").and_then(|v| v.as_object()) {
                for (k, v) in map.iter() {
                    let n = v.as_i64().ok_or(format!("`burn_times` `{k}` must be a whole number"))?;
                    let key = match k.strip_prefix('#') {
                        Some(tag) => format!("#{}", qualify(tag)),
                        None => qualify(k),
                    };
                    burn_times.push((key, n.clamp(0, 1_000_000) as i32));
                }
            }
            let game_table = f.get("game_table").and_then(|v| v.as_bool()).unwrap_or(false);
            if burn_times.is_empty() && !game_table {
                return Err("`process.fuel` needs `burn_times` or `game_table: true`".into());
            }
            Some(Fuel { slot, burn_times, game_table })
        }
    };
    let mut sounds = Sounds::default();
    if let Some(list) = json.get("sounds") {
        if let Some(v) = list.get("start") { sounds.start = Some(parse_sound("start", v)?); }
        if let Some(v) = list.get("working") { sounds.working = Some(parse_sound("working", v)?); }
        if let Some(v) = list.get("done") { sounds.done = Some(parse_sound("done", v)?); }
    }
    let flame = match gui.and_then(|g| g.get("flame")) {
        None => None,
        Some(p) => Some(Bar {
            pos: pair(p.get("pos")).ok_or("`gui.flame.pos` must be [x, y]")?,
            size: pair(p.get("size")).unwrap_or([14.0, 4.0]),
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
        flame,
        mode,
        default_time,
        heat,
        fuel,
        keep_progress,
        lit_property,
        sounds,
    })
}

fn parse_heat(h: &serde_json::Value) -> Result<Heat, String> {
    let offset = match h.get("offset").and_then(|v| v.as_array()) {
        None => [0, -1, 0],
        Some(a) => match a.as_slice() {
            [x, y, z] => [x.as_i64().ok_or("`heat.offset` must be three whole numbers")? as i32,
                          y.as_i64().ok_or("`heat.offset` must be three whole numbers")? as i32,
                          z.as_i64().ok_or("`heat.offset` must be three whole numbers")? as i32],
            _ => return Err("`heat.offset` must be [x, y, z]".into()),
        },
    };
    let list = h.get("sources").and_then(|v| v.as_array()).ok_or("`heat.sources` must be a list")?;
    if list.is_empty() {
        return Err("`heat.sources` is empty".into());
    }
    let mut sources = Vec::new();
    for entry in list {
        sources.push(parse_source(entry)?);
    }
    Ok(Heat { offset, sources })
}

/// A heat source: `"ns:block"`, `"ns:block[lit=true]"`, `"#ns:tag"`, `"#ns:tag[lit=true]"`, or an object
/// `{ "block" | "tag": "...", "properties": { "lit": "true" }, "speed": 1.5 }`.
pub fn parse_source(v: &serde_json::Value) -> Result<HeatSource, String> {
    let (mut block, mut tag, mut properties, mut speed) = (None, None, Vec::new(), 1.0_f32);
    let name_with_props = |text: &str, block: &mut Option<String>, tag: &mut Option<String>, props: &mut Vec<(String, String)>| -> Result<(), String> {
        let (name, rest) = match text.split_once('[') {
            Some((n, r)) => (n, Some(r.strip_suffix(']').ok_or(format!("`{text}`: missing `]`"))?)),
            None => (text, None),
        };
        if let Some(rest) = rest {
            for pair in rest.split(',').filter(|p| !p.is_empty()) {
                let (k, val) = pair.split_once('=').ok_or(format!("`{text}`: `{pair}` is not key=value"))?;
                props.push((k.trim().to_owned(), val.trim().to_owned()));
            }
        }
        match name.strip_prefix('#') {
            Some(t) => *tag = Some(qualify(t)),
            None => *block = Some(qualify(name)),
        }
        Ok(())
    };
    match v {
        serde_json::Value::String(text) => name_with_props(text, &mut block, &mut tag, &mut properties)?,
        serde_json::Value::Object(_) => {
            if let Some(text) = v.get("block").and_then(|x| x.as_str()) {
                name_with_props(text, &mut block, &mut tag, &mut properties)?;
            }
            if let Some(text) = v.get("tag").and_then(|x| x.as_str()) {
                tag = Some(qualify(text.trim_start_matches('#')));
            }
            if let Some(map) = v.get("properties").and_then(|x| x.as_object()) {
                for (k, val) in map.iter() {
                    let text = match val {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        other => other.as_f64().map(|n| n.to_string()).ok_or(format!("property `{k}` must be a string, boolean or number"))?,
                    };
                    properties.push((k.clone(), text));
                }
            }
            if let Some(sp) = v.get("speed") {
                speed = number(sp).filter(|s| *s > 0.0).ok_or("`speed` must be a number above 0")?;
            }
        }
        _ => return Err("a heat source is a string or an object".into()),
    }
    if block.is_none() && tag.is_none() {
        return Err("a heat source needs a `block` or a `tag`".into());
    }
    Ok(HeatSource { block, tag, properties, speed })
}

/// `name` as `minecraft:name` when it has no namespace.
pub fn qualify(name: &str) -> String {
    if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") }
}

struct Registry {
    loaded: bool,
    machines: Vec<Arc<Machine>>,
}

static REGISTRY: Mutex<Registry> = Mutex::new(Registry { loaded: false, machines: Vec::new() });

/// Adds an extension's heat sources and fuels to a machine. The error says what is wrong, for the log.
pub fn extend(machines: &mut [Machine], id: &str, json: &serde_json::Value) -> Result<(), String> {
    let target = json.get("machine").and_then(|v| v.as_str()).ok_or("`machine` must name the machine to extend")?;
    let m = machines
        .iter_mut()
        .find(|m| m.id == target)
        .ok_or(format!("machine `{target}` is not installed (is its mod missing?)"))?;
    let mut added = 0;
    if let Some(list) = json.get("heat_sources").and_then(|v| v.as_array()) {
        let heat = m.heat.get_or_insert(Heat { offset: [0, -1, 0], sources: Vec::new() });
        for entry in list {
            heat.sources.push(parse_source(entry).map_err(|e| format!("{id}: {e}"))?);
            added += 1;
        }
    }
    if let Some(map) = json.get("burn_times").and_then(|v| v.as_object()) {
        let fuel = m.fuel.as_mut().ok_or(format!("machine `{target}` has no fuel slot to add burn times to"))?;
        for (k, v) in map.iter() {
            let n = v.as_i64().ok_or(format!("`burn_times` `{k}` must be a whole number"))?;
            let key = match k.strip_prefix('#') {
                Some(tag) => format!("#{}", qualify(tag)),
                None => qualify(k),
            };
            fuel.burn_times.push((key, n.clamp(0, 1_000_000) as i32));
            added += 1;
        }
    }
    if added == 0 {
        return Err(format!("{id}: an extension needs `heat_sources` or `burn_times`"));
    }
    Ok(())
}

/// The machine whose `block` is `block`. `files` lists the machine definition files and extension files
/// (id and JSON; `FILE_TYPES`) and is only called until some are found: it is empty outside a world, and
/// the content mods never change while the game runs. A file that does not parse is skipped with a warning
/// in the log. Extensions apply after every definition is read, so load order between mods does not matter.
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
        reg.machines = load_machines(files);
        reg.loaded = true;
    }
    reg.machines.iter().find(|m| m.blocks.iter().any(|b| b == block)).cloned()
}

/// Parses every definition, then applies the extensions.
pub fn load_machines(files: Vec<(String, serde_json::Value)>) -> Vec<Arc<Machine>> {
    let mut machines = Vec::new();
    let mut extensions = Vec::new();
    for (id, json) in files {
        if json.get("type").and_then(|t| t.as_str()) == Some(EXTENSION_TYPE) {
            extensions.push((id, json));
            continue;
        }
        match parse(&id, &json) {
            Ok(m) => machines.push(m),
            Err(e) => log::warn!("cookpot: {id}: {e}"),
        }
    }
    for (id, json) in extensions {
        if let Err(e) = extend(&mut machines, &id, &json) {
            log::warn!("cookpot: extension {id}: {e}");
        }
    }
    machines.into_iter().map(Arc::new).collect()
}

/// The ids of every loaded machine.
pub fn machine_ids() -> Vec<String> {
    REGISTRY.lock().map(|r| r.machines.iter().map(|m| m.id.clone()).collect()).unwrap_or_default()
}

/// The values for the menu's data slots: progress, total, burn (or charges), burn total, then zeros up to
/// `data_slots`. They are 16-bit on the wire, so a larger value is clamped.
pub fn data_slots(m: &Machine, st: &State) -> Vec<i32> {
    let mut out = vec![0; m.data_slots];
    for (slot, value) in out.iter_mut().zip([st.progress, st.total, st.burn, st.burn_total]) {
        *slot = value.clamp(0, i16::MAX as i32);
    }
    out
}


// ---------------------------------------------------------------------------------------------------------
// Recipes
// ---------------------------------------------------------------------------------------------------------

/// What an input may be: any of some item ids or any item in some tags.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Ingredient {
    pub items: Vec<String>,
    /// Tag ids without the `#`.
    pub tags: Vec<String>,
}

impl Ingredient {
    /// `in_tag(item, tag)` says whether an item is in an item tag.
    pub fn matches(&self, item: &str, in_tag: &dyn Fn(&str, &str) -> bool) -> bool {
        self.items.iter().any(|i| i == item) || self.tags.iter().any(|t| in_tag(item, t))
    }

    fn add(&mut self, text: &str) {
        match text.strip_prefix('#') {
            Some(tag) => self.tags.push(qualify(tag)),
            None => self.items.push(qualify(text)),
        }
    }
}

/// An ingredient and the count it needs: a string (`"mypack:tomato"`, `"#c:meats"`), a list of those (any
/// one), or `{ "item" | "tag" | "items": ..., "count": 2 }`.
fn parse_ingredient(v: &serde_json::Value) -> Result<(Ingredient, u32), String> {
    let mut ing = Ingredient::default();
    let mut count = 1;
    match v {
        serde_json::Value::String(s) => ing.add(s),
        serde_json::Value::Array(list) => {
            for e in list {
                ing.add(e.as_str().ok_or("an ingredient list holds item ids and #tags")?);
            }
        }
        serde_json::Value::Object(_) => {
            if let Some(s) = v.get("item").and_then(|x| x.as_str()) {
                ing.items.push(qualify(s));
            }
            if let Some(s) = v.get("tag").and_then(|x| x.as_str()) {
                ing.tags.push(qualify(s.trim_start_matches('#')));
            }
            if let Some(list) = v.get("items").and_then(|x| x.as_array()) {
                for e in list {
                    ing.add(e.as_str().ok_or("`items` holds item ids and #tags")?);
                }
            }
            if let Some(c) = v.get("count") {
                count = c.as_u64().filter(|c| (1..=STACK_MAX as u64).contains(c)).ok_or("`count` must be 1 to 64")? as u32;
            }
        }
        _ => return Err("an ingredient is a string, a list or an object".into()),
    }
    if ing.items.is_empty() && ing.tags.is_empty() {
        return Err("an ingredient names no item or tag".into());
    }
    Ok((ing, count))
}

/// One thing a recipe makes. `json` is `{ "id", "count", "components"? }`, what the game reads a result from.
#[derive(Clone, Debug, PartialEq)]
pub struct ResultSpec {
    pub id: String,
    pub count: u32,
    pub json: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    pub id: String,
    /// The machine definition's id (`<ns>:<name>` of its file).
    pub machine: String,
    pub inputs: Vec<(Ingredient, u32)>,
    /// What the container slot must hold (one is used up).
    pub container: Option<Ingredient>,
    pub results: Vec<ResultSpec>,
    pub time: Option<i32>,
    /// Experience points paid out as orbs when a craft finishes (fractions carry over).
    pub experience: f32,
    /// Charges a staged machine spends per craft.
    pub charges: i32,
}

fn parse_result(v: &serde_json::Value) -> Result<ResultSpec, String> {
    let mut map = serde_json::Map::new();
    let (id, count) = match v {
        serde_json::Value::String(s) => (qualify(s), 1),
        serde_json::Value::Object(_) => {
            let id = v.get("id").or_else(|| v.get("item")).and_then(|x| x.as_str()).ok_or("a result needs an `id`")?;
            let count = match v.get("count") {
                None => 1,
                Some(c) => c.as_u64().filter(|c| (1..=STACK_MAX as u64).contains(c)).ok_or("result `count` must be 1 to 64")? as u32,
            };
            if let Some(c) = v.get("components") {
                map.insert("components".to_owned(), c.clone());
            }
            (qualify(id), count)
        }
        _ => return Err("a result is an item id or an object".into()),
    };
    map.insert("id".to_owned(), serde_json::Value::String(id.clone()));
    map.insert("count".to_owned(), serde_json::Value::from(count));
    Ok(ResultSpec { id, count, json: serde_json::Value::Object(map) })
}

pub fn parse_recipe(id: &str, json: &serde_json::Value) -> Result<Recipe, String> {
    let machine = json.get("machine").and_then(|v| v.as_str()).filter(|m| m.contains(':')).ok_or("`machine` must be a machine file id like `mymod:pot`")?;
    let list = json.get("inputs").and_then(|v| v.as_array()).ok_or("`inputs` must be a list")?;
    let mut inputs = Vec::new();
    for e in list {
        inputs.push(parse_ingredient(e)?);
    }
    let container = match json.get("container") {
        None => None,
        Some(c) => Some(parse_ingredient(c)?.0),
    };
    let mut results = Vec::new();
    match (json.get("result"), json.get("results").and_then(|v| v.as_array())) {
        (Some(r), _) => results.push(parse_result(r)?),
        (None, Some(list)) => {
            for r in list {
                results.push(parse_result(r)?);
            }
        }
        _ => {}
    }
    if inputs.is_empty() && container.is_none() {
        return Err("a recipe needs at least one input or a container".into());
    }
    if results.is_empty() {
        return Err("a recipe needs a `result` or a `results` list".into());
    }
    Ok(Recipe {
        id: id.to_owned(),
        machine: machine.to_owned(),
        inputs,
        container,
        results,
        time: json.get("time").and_then(|v| v.as_i64()).map(|t| t.clamp(0, 1_000_000) as i32),
        experience: json.get("experience").and_then(number).unwrap_or(0.0).max(0.0),
        charges: json.get("charges").and_then(|v| v.as_i64()).unwrap_or(1).clamp(0, 1000) as i32,
    })
}

struct RecipeRegistry {
    loaded: bool,
    recipes: Vec<Arc<Recipe>>,
}

static RECIPES: Mutex<RecipeRegistry> = Mutex::new(RecipeRegistry { loaded: false, recipes: Vec::new() });

/// The recipes of the machine `machine_id`. `files` lists the `cookpot:recipe` files and is only called
/// until the first time; a file that does not parse is skipped with a warning in the log.
pub fn recipes_for(
    machine_id: &str,
    files: impl FnOnce() -> Vec<(String, serde_json::Value)>,
) -> Vec<Arc<Recipe>> {
    let Ok(mut reg) = RECIPES.lock() else { return Vec::new() };
    if !reg.loaded {
        reg.recipes = load_recipes(files(), &machine_ids());
        reg.loaded = true;
    }
    reg.recipes.iter().filter(|r| r.machine == machine_id).cloned().collect()
}

/// Parses every recipe, warning about those for a machine not in `known`.
pub fn load_recipes(files: Vec<(String, serde_json::Value)>, known: &[String]) -> Vec<Arc<Recipe>> {
    let mut out = Vec::new();
    for (id, json) in files {
        match parse_recipe(&id, &json) {
            Ok(r) => {
                if !known.contains(&r.machine) {
                    log::warn!("cookpot: recipe {id} is for machine `{}`, which is not installed (is its mod missing?)", r.machine);
                }
                out.push(Arc::new(r));
            }
            Err(e) => log::warn!("cookpot: recipe {id}: {e}"),
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------------------
// Heat and fuel
// ---------------------------------------------------------------------------------------------------------

/// The speed of the best heat source at the block described by `block` (its id), or 0.0 for none.
/// `property(key)` reads a property of that block; `block_in_tag(block, tag)` tests a block tag.
pub fn heat_speed(
    m: &Machine,
    block: &str,
    property: &dyn Fn(&str) -> Option<String>,
    block_in_tag: &dyn Fn(&str, &str) -> bool,
) -> f32 {
    let Some(heat) = &m.heat else { return 0.0 };
    let mut best = 0.0_f32;
    for s in &heat.sources {
        let named = s.block.as_deref().is_some_and(|b| b == block);
        let tagged = s.tag.as_deref().is_some_and(|t| block_in_tag(block, t));
        if !(named || tagged) {
            continue;
        }
        if s.properties.iter().all(|(k, v)| property(k).as_deref() == Some(v.as_str())) {
            best = best.max(s.speed);
        }
    }
    best
}

/// The burn time (or charges) of `item` from the definition's `burn_times`: an exact item id first, then
/// tags. None when it is not listed. `in_tag(item, tag)` tests an item tag.
pub fn fuel_value(m: &Machine, item: &str, in_tag: &dyn Fn(&str, &str) -> bool) -> Option<i32> {
    let fuel = m.fuel.as_ref()?;
    if let Some((_, n)) = fuel.burn_times.iter().find(|(k, _)| k == item) {
        return Some(*n);
    }
    fuel.burn_times
        .iter()
        .filter_map(|(k, n)| k.strip_prefix('#').map(|tag| (tag, *n)))
        .find(|(tag, _)| in_tag(item, tag))
        .map(|(_, n)| n)
}

// ---------------------------------------------------------------------------------------------------------
// The process
// ---------------------------------------------------------------------------------------------------------

/// An item stack as the process sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stack {
    pub item: String,
    pub count: u32,
}

/// What a machine remembers between ticks (saved in its block entity).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct State {
    pub progress: i32,
    pub total: i32,
    /// Timed: ticks of fuel left. Staged: charges.
    pub burn: i32,
    pub burn_total: i32,
    /// Leftover thousandths of a tick, for heat that is faster or slower than normal.
    pub acc: i32,
    /// 0 for none, else the index of the recipe in progress plus one.
    pub recipe: i32,
    /// Experience earned but not yet paid, in thousandths of a point.
    pub xp: i32,
}

/// A craft to apply to the machine's slots.
#[derive(Clone, Debug, PartialEq)]
pub struct Craft {
    /// `(slot, count)` to take away.
    pub consume: Vec<(usize, u32)>,
    /// Index of the recipe whose `results` to add to the output slots.
    pub recipe: usize,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Step {
    pub craft: Option<Craft>,
    /// One item of this slot was used as fuel: take one away.
    pub fuel_slot: Option<usize>,
    /// The machine is working: a `lit` property follows this.
    pub lit: bool,
    /// Whole experience points to pay out now (as orbs above the machine).
    pub xp: u32,
    /// Sounds to play, in order.
    pub cues: Vec<Cue>,
}

/// The input slots a recipe uses, or None when it does not match. Slots are tried in order; an ingredient
/// may need several slots only by naming a count that fits one slot.
fn assign(
    m: &Machine,
    r: &Recipe,
    slots: &[Option<Stack>],
    in_tag: &dyn Fn(&str, &str) -> bool,
) -> Option<Vec<(usize, u32)>> {
    let input_slots = m.slot_indices(|role| role == Role::Input);
    fn go(
        k: usize,
        r: &Recipe,
        input_slots: &[usize],
        slots: &[Option<Stack>],
        used: &mut Vec<usize>,
        in_tag: &dyn Fn(&str, &str) -> bool,
    ) -> bool {
        let Some((ing, need)) = r.inputs.get(k) else { return true };
        for &i in input_slots {
            if used.contains(&i) {
                continue;
            }
            if let Some(Some(s)) = slots.get(i) {
                if s.count >= *need && ing.matches(&s.item, in_tag) {
                    used.push(i);
                    if go(k + 1, r, input_slots, slots, used, in_tag) {
                        return true;
                    }
                    used.pop();
                }
            }
        }
        false
    }
    let mut used = Vec::new();
    if !go(0, r, &input_slots, slots, &mut used, in_tag) {
        return None;
    }
    let mut consume: Vec<(usize, u32)> = used.iter().enumerate().map(|(k, &i)| (i, r.inputs[k].1)).collect();
    if let Some(ing) = &r.container {
        let slot = m
            .slot_indices(|role| role == Role::Container)
            .into_iter()
            .find(|&i| matches!(slots.get(i), Some(Some(s)) if s.count >= 1 && ing.matches(&s.item, in_tag)))?;
        consume.push((slot, 1));
    }
    Some(consume)
}

/// Whether every result fits the output slots (merging into like stacks or filling empty ones).
fn has_room(m: &Machine, r: &Recipe, slots: &[Option<Stack>]) -> bool {
    let outs = m.slot_indices(Role::is_output);
    let mut space: Vec<Option<Stack>> = outs.iter().map(|&i| slots.get(i).cloned().flatten()).collect();
    'next: for res in &r.results {
        let mut left = res.count;
        for s in space.iter_mut().flatten() {
            if s.item == res.id && s.count < STACK_MAX {
                let take = left.min(STACK_MAX - s.count);
                s.count += take;
                left -= take;
                if left == 0 {
                    continue 'next;
                }
            }
        }
        for slot in space.iter_mut() {
            if slot.is_none() {
                let take = left.min(STACK_MAX);
                *slot = Some(Stack { item: res.id.clone(), count: take });
                left -= take;
                if left == 0 {
                    continue 'next;
                }
            }
        }
        return false;
    }
    true
}

/// The first recipe that matches and has room for its results, with the slots it takes from.
pub fn find_recipe(
    m: &Machine,
    recipes: &[Arc<Recipe>],
    slots: &[Option<Stack>],
    in_tag: &dyn Fn(&str, &str) -> bool,
) -> Option<(usize, Vec<(usize, u32)>)> {
    recipes.iter().enumerate().find_map(|(i, r)| {
        let consume = assign(m, r, slots, in_tag)?;
        has_room(m, r, slots).then_some((i, consume))
    })
}

/// Rotates `acc` thousandths of a tick by `rate` and returns the whole ticks that came due.
fn pace(acc: &mut i32, rate: f32) -> i32 {
    *acc += (rate.max(0.0) * 1000.0).round() as i32;
    let whole = *acc / 1000;
    *acc %= 1000;
    whole
}

/// One server tick of a machine. `slots` is what the machine holds (empty slots `None`); `heat` is
/// `heat_speed` for the block it looks at; `fuel_of(item)` is the burn time or charges of an item (0 for
/// not a fuel); `in_tag(item, tag)` tests an item tag. Changes `st`, and returns what to do to the slots.
pub fn step(
    m: &Machine,
    recipes: &[Arc<Recipe>],
    st: &mut State,
    slots: &[Option<Stack>],
    heat: f32,
    fuel_of: &dyn Fn(&str) -> i32,
    in_tag: &dyn Fn(&str, &str) -> bool,
) -> Step {
    let mut out = Step::default();
    if m.mode == Mode::Timed || m.mode == Mode::Instant {
        st.burn = (st.burn - 1).max(0);
    }
    let Some((ri, consume)) = find_recipe(m, recipes, slots, in_tag) else {
        st.recipe = 0;
        st.total = 0;
        if !m.keep_progress {
            st.progress = (st.progress - 2).max(0);
        }
        out.lit = st.burn > 0 && m.mode != Mode::Staged;
        return out;
    };
    let r = &recipes[ri];
    if st.recipe != ri as i32 + 1 {
        st.recipe = ri as i32 + 1;
        st.progress = 0;
        st.acc = 0;
    }
    st.total = if m.mode == Mode::Instant { 0 } else { r.time.unwrap_or(m.default_time) };
    let needs_power = m.heat.is_some() || m.fuel.is_some();
    let mut rate = if heat > 0.0 { heat } else { 1.0 };
    let mut powered = !needs_power || heat > 0.0;
    let mut paid_by_charges = false;
    if !powered {
        match m.mode {
            Mode::Timed | Mode::Instant => {
                if st.burn == 0 {
                    if let Some((slot, value)) = take_fuel(m, slots, fuel_of) {
                        st.burn = value;
                        st.burn_total = value;
                        out.fuel_slot = Some(slot);
                    }
                }
                powered = st.burn > 0;
                rate = 1.0;
            }
            Mode::Staged => {
                if st.burn < r.charges {
                    if let Some((slot, value)) = take_fuel(m, slots, fuel_of) {
                        st.burn += value;
                        st.burn_total = st.burn_total.max(st.burn);
                        out.fuel_slot = Some(slot);
                    }
                }
                powered = st.burn >= r.charges;
                paid_by_charges = true;
                rate = 1.0;
            }
        }
    }
    let before = st.progress;
    if powered {
        st.progress += pace(&mut st.acc, rate);
        if before == 0 && st.progress > 0 {
            out.cues.push(Cue::Start);
        } else if let Some(w) = &m.sounds.working {
            if st.progress / w.interval > before / w.interval {
                out.cues.push(Cue::Working);
            }
        }
    } else if !m.keep_progress {
        st.progress = (st.progress - 2).max(0);
    }
    out.lit = powered;
    if powered && st.progress >= st.total {
        st.progress = 0;
        st.acc = 0;
        st.recipe = 0;
        if paid_by_charges {
            st.burn -= r.charges;
        }
        st.xp += (r.experience * 1000.0).round() as i32;
        out.xp = (st.xp / 1000) as u32;
        st.xp %= 1000;
        out.cues.push(Cue::Done);
        out.craft = Some(Craft { consume, recipe: ri });
    }
    out
}

/// The first fuel-slot item with a fuel value above zero: its slot and value.
fn take_fuel(m: &Machine, slots: &[Option<Stack>], fuel_of: &dyn Fn(&str) -> i32) -> Option<(usize, i32)> {
    let slot = m.fuel.as_ref()?.slot;
    let s = slots.get(slot)?.as_ref()?;
    let value = fuel_of(&s.item);
    (value > 0 && s.count > 0).then_some((slot, value))
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

    fn pot() -> Machine {
        parse("t:pot_machine", &json!({
            "type": "cookpot:machine", "block": "t:pot", "menu": "t:pot", "data_slots": 4,
            "heat": { "sources": ["minecraft:campfire[lit=true]", "#t:hot", { "block": "minecraft:magma_block", "speed": 2.0 }] },
            "process": { "default_time": 4, "lit_property": "lit",
                "fuel": { "slot": "fuel", "burn_times": { "minecraft:coal": 10, "#minecraft:planks": 3 } } },
            "gui": { "slots": [
                { "id": "a", "role": "input" }, { "id": "b", "role": "input" },
                { "id": "fuel", "role": "fuel" }, { "id": "bowl", "role": "container" },
                { "id": "out", "role": "output" } ] }
        })).unwrap()
    }

    fn soup() -> Vec<Arc<Recipe>> {
        vec![Arc::new(parse_recipe("t:soup", &json!({
            "type": "cookpot:recipe", "machine": "t:pot_machine",
            "inputs": [ { "item": "t:tomato", "count": 2 }, "#t:meats" ],
            "container": "bowl", "result": { "id": "t:soup" }, "time": 3
        })).unwrap())]
    }

    fn stack(item: &str, count: u32) -> Option<Stack> {
        Some(Stack { item: item.to_owned(), count })
    }

    fn meat_tag(item: &str, tag: &str) -> bool {
        tag == "t:meats" && item == "t:beef"
    }

    #[test]
    fn parses_a_recipe_and_rejects_a_bad_one() {
        let r = &soup()[0];
        assert_eq!(r.inputs.len(), 2);
        assert_eq!(r.inputs[0].1, 2);
        assert_eq!(r.container.as_ref().unwrap().items, vec!["minecraft:bowl".to_owned()]);
        assert_eq!(r.results[0].id, "t:soup");
        assert!(parse_recipe("x", &json!({ "machine": "t:pot", "inputs": [], "result": "a:b" })).is_err());
        assert!(parse_recipe("x", &json!({ "machine": "pot", "inputs": ["a:b"], "result": "a:b" })).is_err());
        assert!(parse_recipe("x", &json!({ "machine": "t:pot", "inputs": ["a:b"] })).is_err());
    }

    #[test]
    fn heat_comes_from_the_blocks_future_authors_name() {
        let m = pot();
        let lit = |k: &str| (k == "lit").then(|| "true".to_owned());
        let unlit = |k: &str| (k == "lit").then(|| "false".to_owned());
        let tag = |b: &str, t: &str| t == "t:hot" && b == "t:stove";
        assert_eq!(heat_speed(&m, "minecraft:campfire", &lit, &tag), 1.0);
        assert_eq!(heat_speed(&m, "minecraft:campfire", &unlit, &tag), 0.0);
        assert_eq!(heat_speed(&m, "t:stove", &unlit, &tag), 1.0);
        assert_eq!(heat_speed(&m, "minecraft:magma_block", &unlit, &tag), 2.0);
        assert_eq!(heat_speed(&m, "minecraft:stone", &lit, &tag), 0.0);
    }

    #[test]
    fn fuel_values_by_item_then_tag() {
        let m = pot();
        let planks = |i: &str, t: &str| t == "minecraft:planks" && i == "minecraft:oak_planks";
        assert_eq!(fuel_value(&m, "minecraft:coal", &planks), Some(10));
        assert_eq!(fuel_value(&m, "minecraft:oak_planks", &planks), Some(3));
        assert_eq!(fuel_value(&m, "minecraft:dirt", &planks), None);
    }

    #[test]
    fn matches_tags_counts_containers_and_room() {
        let m = pot();
        let recipes = soup();
        let full = vec![stack("t:tomato", 2), stack("t:beef", 1), None, stack("minecraft:bowl", 1), None];
        let (ri, consume) = find_recipe(&m, &recipes, &full, &meat_tag).unwrap();
        assert_eq!(ri, 0);
        assert_eq!(consume, vec![(0, 2), (1, 1), (3, 1)]);
        let few = vec![stack("t:tomato", 1), stack("t:beef", 1), None, stack("minecraft:bowl", 1), None];
        assert!(find_recipe(&m, &recipes, &few, &meat_tag).is_none());
        let no_bowl = vec![stack("t:tomato", 2), stack("t:beef", 1), None, None, None];
        assert!(find_recipe(&m, &recipes, &no_bowl, &meat_tag).is_none());
        let blocked = vec![stack("t:tomato", 2), stack("t:beef", 1), None, stack("minecraft:bowl", 1), stack("t:soup", 64)];
        assert!(find_recipe(&m, &recipes, &blocked, &meat_tag).is_none());
        let stacking = vec![stack("t:tomato", 2), stack("t:beef", 1), None, stack("minecraft:bowl", 1), stack("t:soup", 10)];
        assert!(find_recipe(&m, &recipes, &stacking, &meat_tag).is_some());
    }

    #[test]
    fn a_heat_block_powers_the_process_without_fuel() {
        let m = pot();
        let recipes = soup();
        let slots = vec![stack("t:tomato", 2), stack("t:beef", 1), None, stack("minecraft:bowl", 1), None];
        let mut st = State::default();
        let fuel = |_: &str| 0;
        for _ in 0..2 {
            let out = step(&m, &recipes, &mut st, &slots, 1.0, &fuel, &meat_tag);
            assert!(out.craft.is_none() && out.lit && out.fuel_slot.is_none());
        }
        let out = step(&m, &recipes, &mut st, &slots, 1.0, &fuel, &meat_tag);
        assert_eq!(out.craft.unwrap().recipe, 0);
        assert_eq!(st.progress, 0);
        // Faster heat finishes sooner.
        let mut fast = State::default();
        step(&m, &recipes, &mut fast, &slots, 2.0, &fuel, &meat_tag);
        let out = step(&m, &recipes, &mut fast, &slots, 2.0, &fuel, &meat_tag);
        assert!(out.craft.is_some());
    }

    #[test]
    fn fuel_burns_and_runs_out() {
        let m = pot();
        let recipes = soup();
        let slots = vec![stack("t:tomato", 2), stack("t:beef", 1), stack("minecraft:coal", 3), stack("minecraft:bowl", 1), None];
        let mut st = State::default();
        let fuel = |i: &str| if i == "minecraft:coal" { 10 } else { 0 };
        let out = step(&m, &recipes, &mut st, &slots, 0.0, &fuel, &meat_tag);
        assert_eq!(out.fuel_slot, Some(2));
        assert_eq!((st.burn, st.burn_total, st.progress), (10, 10, 1));
        // No fuel and no heat: nothing burns, progress drains.
        let dry = vec![stack("t:tomato", 2), stack("t:beef", 1), None, stack("minecraft:bowl", 1), None];
        let mut cold = State { progress: 2, ..State::default() };
        let out = step(&m, &recipes, &mut cold, &dry, 0.0, &fuel, &meat_tag);
        assert!(out.craft.is_none() && !out.lit);
        assert_eq!(cold.progress, 0);
        let mut keep = m.clone();
        keep.keep_progress = true;
        let mut held = State { progress: 2, recipe: 1, ..State::default() };
        step(&keep, &recipes, &mut held, &dry, 0.0, &fuel, &meat_tag);
        assert_eq!(held.progress, 2);
    }

    #[test]
    fn staged_machines_spend_charges_per_craft() {
        let mut m = pot();
        m.mode = Mode::Staged;
        m.heat = None;
        let recipes = soup();
        let slots = vec![stack("t:tomato", 2), stack("t:beef", 1), stack("minecraft:coal", 1), stack("minecraft:bowl", 1), None];
        let fuel = |i: &str| if i == "minecraft:coal" { 5 } else { 0 };
        let mut st = State::default();
        let first = step(&m, &recipes, &mut st, &slots, 0.0, &fuel, &meat_tag);
        assert_eq!(first.fuel_slot, Some(2));
        assert_eq!(st.burn, 5);
        let mut crafted = false;
        for _ in 0..2 {
            crafted |= step(&m, &recipes, &mut st, &slots, 0.0, &fuel, &meat_tag).craft.is_some();
        }
        assert!(crafted);
        assert_eq!(st.burn, 4);
    }

    #[test]
    fn instant_machines_craft_on_the_first_tick() {
        let mut m = pot();
        m.mode = Mode::Instant;
        m.heat = None;
        m.fuel = None;
        let recipes = soup();
        let slots = vec![stack("t:tomato", 2), stack("t:beef", 1), None, stack("minecraft:bowl", 1), None];
        let mut st = State::default();
        let out = step(&m, &recipes, &mut st, &slots, 0.0, &|_| 0, &meat_tag);
        assert!(out.craft.is_some());
    }

    #[test]
    fn xp_pays_whole_points_and_carries_fractions() {
        let m = pot();
        let mut r = (*soup()[0]).clone();
        r.experience = 0.7;
        r.time = Some(0);
        let recipes = vec![Arc::new(r)];
        let slots = vec![stack("t:tomato", 2), stack("t:beef", 1), None, stack("minecraft:bowl", 1), None];
        let mut st = State::default();
        let first = step(&m, &recipes, &mut st, &slots, 1.0, &|_| 0, &meat_tag);
        assert!(first.craft.is_some());
        assert_eq!((first.xp, st.xp), (0, 700));
        let second = step(&m, &recipes, &mut st, &slots, 1.0, &|_| 0, &meat_tag);
        assert_eq!((second.xp, st.xp), (1, 400));
    }

    #[test]
    fn sounds_parse_and_cue() {
        let mut m = pot();
        let d = json!({ "type": "cookpot:machine", "block": "t:pot", "menu": "t:pot",
            "sounds": { "start": "minecraft:block.lava.pop", "working": { "id": "t:bubble", "interval": 2, "volume": 0.5 },
                        "done": { "id": "minecraft:entity.player.levelup", "pitch": 2.0 } },
            "gui": { "slots": [ { "id": "a", "role": "input" }, { "id": "o", "role": "output" } ] } });
        let parsed = parse("t:s", &d).unwrap();
        assert_eq!(parsed.sounds.working.as_ref().unwrap().interval, 2);
        assert_eq!(parsed.sounds.done.as_ref().unwrap().pitch, 2.0);
        let mut bad = d.clone();
        bad["sounds"]["start"] = json!("Not A Sound");
        assert!(parse("t:s", &bad).unwrap_err().contains("sounds.start"));
        m.sounds = parsed.sounds;
        let recipes = soup();
        let slots = vec![stack("t:tomato", 2), stack("t:beef", 1), None, stack("minecraft:bowl", 1), None];
        let mut st = State::default();
        let cues: Vec<Vec<Cue>> = (0..3).map(|_| step(&m, &recipes, &mut st, &slots, 1.0, &|_| 0, &meat_tag).cues).collect();
        assert_eq!(cues, vec![vec![Cue::Start], vec![Cue::Working], vec![Cue::Done]]);
    }

    #[test]
    fn another_mod_can_extend_a_machine_and_add_recipes() {
        let machine = json!({ "type": "cookpot:machine", "block": "base:pot", "menu": "base:pot",
            "process": { "fuel": { "slot": "f", "burn_times": { "minecraft:coal": 100 } } },
            "gui": { "slots": [ { "id": "a", "role": "input" }, { "id": "f", "role": "fuel" }, { "id": "o", "role": "output" } ] } });
        let ext = json!({ "type": "cookpot:extension", "machine": "base:pot_machine",
            "heat_sources": ["addon:stove[lit=true]"], "burn_times": { "#addon:fuels": 50 } });
        let orphan = json!({ "type": "cookpot:extension", "machine": "gone:pot_machine", "heat_sources": ["a:b"] });
        // Extensions listed before the machine still apply.
        let files = vec![("addon:more".to_owned(), ext), ("addon:orphan".to_owned(), orphan), ("base:pot_machine".to_owned(), machine)];
        let machines = load_machines(files);
        let m = machines.iter().find(|m| m.blocks.contains(&"base:pot".to_owned())).unwrap().clone();
        let heat = m.heat.as_ref().unwrap();
        assert_eq!(heat.sources[0].block.as_deref(), Some("addon:stove"));
        assert_eq!(heat.sources[0].properties, vec![("lit".to_owned(), "true".to_owned())]);
        assert!(m.fuel.as_ref().unwrap().burn_times.iter().any(|(k, n)| k == "#addon:fuels" && *n == 50));
        // A recipe from a third mod points at the first mod's machine by id.
        let known: Vec<String> = machines.iter().map(|m| m.id.clone()).collect();
        let recipes = load_recipes(vec![
            ("addon:bread".to_owned(), json!({ "machine": "base:pot_machine", "inputs": ["addon:dough"], "result": "addon:bread" })),
            ("addon:lost".to_owned(), json!({ "machine": "gone:pot_machine", "inputs": ["a:b"], "result": "a:c" })),
        ], &known);
        // Both load (the second warns in the log); a machine only runs the recipes that name it.
        assert_eq!(recipes.iter().filter(|r| r.machine == "base:pot_machine").count(), 1);
        assert_eq!(recipes[0].id, "addon:bread");
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
