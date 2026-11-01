//! The left panel's tools (m0_contrat.md §1.12, §4.2, §4.4, §4.10; M0-T89): Brush, Eraser, Heat and Cool, the brush size, and
//! the nine paintable elements, the three presets (§4.5, M0-T90) and the four views (§4.2, M0-T91), as a view model. `Tools` is the state (the selected tool, element and radius); `view` says what
//! the panel shows; `Command` is what a click or a key asks of the state and `Tools::apply` does it, so the window, the keys
//! and the scripted captures all go through the same door. `frame_edit` is the edit one frame of the held button queues — one
//! per frame, never one per mouse event. The egui layer (desktop.rs) draws the `LeftPanel` as given.

use eframe::egui;
use sr_engine::render::View;
use sr_engine::scene::cloud_shape;
use sr_engine::state::WorldConfig;
use sr_engine::step::edit::{Edit, RADIUS};
use sr_physics::registry::Elements;

use crate::strings;

/// The brush size a new world starts with (§4.4).
pub const DEFAULT_RADIUS: u32 = 8;
/// What `[` and `]` change the radius by (§4.10).
pub const RADIUS_STEP: u32 = 2;

/// The four tools of §4.2, in the panel's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Brush,
    Eraser,
    Heat,
    Cool,
}

impl Tool {
    pub const ALL: [Tool; 4] = [Tool::Brush, Tool::Eraser, Tool::Heat, Tool::Cool];

    /// The §4.1 key of the tool's label.
    fn label_key(self) -> &'static str {
        match self {
            Tool::Brush => "tools.brush",
            Tool::Eraser => "tools.eraser",
            Tool::Heat => "tools.heat",
            Tool::Cool => "tools.cool",
        }
    }

    /// The tool a capture's `{"tool": …}` names (§3.3).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn parse(name: &str) -> Option<Tool> {
        Some(match name {
            "brush" => Tool::Brush,
            "eraser" => Tool::Eraser,
            "heat" => Tool::Heat,
            "cool" => Tool::Cool,
            _ => return None,
        })
    }
}

/// What a control of the panel (or a key) asks of the tools.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// A tool button, B, E, H or C.
    SelectTool(Tool),
    /// An element row: the species id (§1.5).
    SelectElement(usize),
    /// The slider: the radius asked for, cut to 1–40.
    SetRadius(u32),
    /// `[`.
    Smaller,
    /// `]`.
    Larger,
}

/// The tools' state: what the left button does and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tools {
    tool: Tool,
    species: usize,
    radius: u32,
    /// The species ids the element list offers, in the registry's order — never neutron matter (§4.2).
    paintable: Vec<usize>,
}

impl Tools {
    /// A new world's tools: the Brush, Hydrogen (B2: «Paint hydrogen gas with the brush») and 8 cells.
    pub fn new(elements: &Elements) -> Tools {
        let paintable: Vec<usize> =
            elements.species().iter().enumerate().filter(|(_, s)| s.paintable).map(|(id, _)| id).collect();
        Tools { tool: Tool::Brush, species: paintable.first().copied().unwrap_or(0), radius: DEFAULT_RADIUS, paintable }
    }

    #[cfg(test)]
    pub fn tool(&self) -> Tool {
        self.tool
    }

    #[cfg(test)]
    pub fn species(&self) -> usize {
        self.species
    }

    #[cfg(test)]
    pub fn radius(&self) -> u32 {
        self.radius
    }

    /// The species id of the element a capture's `{"element": "<key>"}` names, if it is in the list.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn species_of(&self, elements: &Elements, key: &str) -> Option<usize> {
        elements.index_of(key).filter(|id| self.paintable.contains(id))
    }

    /// Does what `command` asks. An element outside the list (neutron matter) is never selected; the radius stays in 1–40.
    pub fn apply(&mut self, command: Command) {
        let (lo, hi) = (*RADIUS.start(), *RADIUS.end());
        match command {
            Command::SelectTool(tool) => self.tool = tool,
            Command::SelectElement(id) if self.paintable.contains(&id) => self.species = id,
            Command::SelectElement(_) => {}
            Command::SetRadius(r) => self.radius = r.clamp(lo, hi),
            Command::Smaller => self.radius = self.radius.saturating_sub(RADIUS_STEP).clamp(lo, hi),
            Command::Larger => self.radius = (self.radius + RADIUS_STEP).clamp(lo, hi),
        }
    }
}

/// The four views in the panel's order; F1–F4 pick them the same way (§4.10).
pub const VIEWS: [View; 4] = [View::Glow, View::Heat, View::Element, View::Density];

/// The §4.1 key of a view's label.
fn view_label_key(view: View) -> &'static str {
    match view {
        View::Glow => "views.glow",
        View::Heat => "views.heat",
        View::Element => "views.element",
        View::Density => "views.density",
    }
}

/// The view a capture's `{"view": …}` names (§3.3).
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub fn parse_view(name: &str) -> Option<View> {
    Some(match name {
        "glow" => View::Glow,
        "heat" => View::Heat,
        "element" => View::Element,
        "density" => View::Density,
        _ => return None,
    })
}

/// What a view button or F1–F4 asks of the views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewCommand {
    Select(View),
}

/// The active view the renderer draws (§1.11): Glow at first. Apart from `Tools` — a switch touches neither the brush nor the element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Views {
    view: View,
}

impl Default for Views {
    fn default() -> Views {
        Views { view: View::Glow }
    }
}

impl Views {
    pub fn active(&self) -> View {
        self.view
    }

    pub fn apply(&mut self, command: ViewCommand) {
        match command {
            ViewCommand::Select(view) => self.view = view,
        }
    }
}

/// How long `presets.no_room` stays under the world (§4.5).
pub const NO_ROOM_SECONDS: f64 = 3.0;

/// The three presets of §2.10, in the panel's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Sun,
    Massive,
    Giant,
}

impl Preset {
    pub const ALL: [Preset; 3] = [Preset::Sun, Preset::Massive, Preset::Giant];

    /// The engine's name for the preset (`scene::PRESETS`).
    pub fn name(self) -> &'static str {
        match self {
            Preset::Sun => "sun",
            Preset::Massive => "massive",
            Preset::Giant => "giant",
        }
    }

    /// The preset a capture's `{"preset": …}` or the engine's name stands for.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn parse(name: &str) -> Option<Preset> {
        Preset::ALL.into_iter().find(|p| p.name() == name)
    }

    fn label_key(self) -> &'static str {
        match self {
            Preset::Sun => "presets.sun",
            Preset::Massive => "presets.massive",
            Preset::Giant => "presets.giant",
        }
    }

    fn note_key(self) -> &'static str {
        match self {
            Preset::Sun => "presets.sun_note",
            Preset::Massive => "presets.massive_note",
            Preset::Giant => "presets.giant_note",
        }
    }
}

/// What a preset button or Esc asks of the presets (§4.5, §4.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetCommand {
    /// A preset button: arms it (a second press on the armed one keeps it armed).
    Arm(Preset),
    /// Esc.
    Disarm,
}

/// The presets' state: the armed one, and the refusal under the world while it lasts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Presets {
    armed: Option<Preset>,
    /// The egui time (s) `presets.no_room` stops showing at.
    refused_until: Option<f64>,
}

impl Presets {
    pub fn armed(&self) -> Option<Preset> {
        self.armed
    }

    /// Does what `command` asks. Arming or disarming takes the old refusal off the screen.
    pub fn apply(&mut self, command: PresetCommand) {
        self.refused_until = None;
        self.armed = match command {
            PresetCommand::Arm(preset) => Some(preset),
            PresetCommand::Disarm => None,
        };
    }

    /// The armed preset was dropped: the click is spent.
    pub fn placed(&mut self) {
        *self = Presets::default();
    }

    /// The engine refused the drop for lack of room, at egui time `now`: the preset stays armed for another click, and
    /// `presets.no_room` shows for [`NO_ROOM_SECONDS`].
    pub fn refused(&mut self, now: f64) {
        self.refused_until = Some(now + NO_ROOM_SECONDS);
    }

    /// The lines under the world at egui time `now` (§4.2, §4.5): `presets.place_hint` while a preset is armed, then
    /// `presets.no_room` for 3 s after a refusal.
    pub fn notices(&self, now: f64) -> Vec<String> {
        let mut lines = Vec::new();
        if self.armed.is_some() {
            lines.push(strings::get("presets.place_hint").to_string());
        }
        if self.refused_until.is_some_and(|until| now < until) {
            lines.push(strings::get("presets.no_room").to_string());
        }
        lines
    }
}

/// The cloud of a preset of `mass` (sandbox units): the radius, in cells, the engine lays it out with (§2.10: 40 cells per
/// √Sun-mass, capped at 150). The outline under the cursor is drawn at this radius and the engine refuses the drop by the
/// same cloud (`Edit::drop_preset` reads `scene::cloud_shape` too), so what the outline shows is what has to fit.
pub fn cloud_radius(mass: f64) -> f64 {
    cloud_shape(mass).a
}

/// The cloud's outline over the world: the centre of `cell` and `radius` cells, in screen points.
pub fn cloud_outline(rect: egui::Rect, world: WorldConfig, cell: [u32; 2], radius: f64) -> (egui::Pos2, f32) {
    let scale = rect.width() / world.width as f32;
    let centre = rect.min + egui::vec2(cell[0] as f32 + 0.5, cell[1] as f32 + 0.5) * scale;
    (centre, radius as f32 * scale)
}

/// A button: the text on it, and what it sends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolButton {
    pub label: String,
    pub command: Command,
    /// The selected tool is drawn in `accent` (§4.2).
    pub selected: bool,
}

/// `tools.brush_size` with its slider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SizeSlider {
    pub label: String,
    pub value: u32,
    pub min: u32,
    pub max: u32,
}

/// `tools.heading`, Brush, Eraser, Heat, Cool, and the size slider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolsSection {
    pub heading: String,
    pub buttons: Vec<ToolButton>,
    pub size: SizeSlider,
}

/// One row of the element list: its swatch (§1.5's colour) and name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementRow {
    pub label: String,
    pub swatch: [u8; 3],
    pub command: Command,
    pub selected: bool,
}

/// `tools.element_heading` and the paintable elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementsSection {
    pub heading: String,
    pub rows: Vec<ElementRow>,
}

/// One preset button: its name and its note (the mass and the ending it reaches, §4.5), the armed one in `accent`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetButton {
    pub label: String,
    pub note: String,
    pub command: PresetCommand,
    pub armed: bool,
}

/// `presets.heading` and the three buttons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetsSection {
    pub heading: String,
    pub buttons: Vec<PresetButton>,
}

/// One view button, the active one in `accent` (§4.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewButton {
    pub label: String,
    pub command: ViewCommand,
    pub selected: bool,
}

/// `views.heading` and the four buttons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewsSection {
    pub heading: String,
    pub buttons: Vec<ViewButton>,
}

/// The panel, top to bottom (§4.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeftPanel {
    pub tools: ToolsSection,
    pub elements: ElementsSection,
    pub presets: PresetsSection,
    pub views: ViewsSection,
}

/// The panel as the tools, the presets and the views stand.
pub fn view(tools: &Tools, presets: &Presets, views: &Views, elements: &Elements) -> LeftPanel {
    let buttons = Tool::ALL
        .iter()
        .map(|&tool| ToolButton {
            label: strings::get(tool.label_key()).to_string(),
            command: Command::SelectTool(tool),
            selected: tool == tools.tool,
        })
        .collect();
    let rows = tools
        .paintable
        .iter()
        .map(|&id| {
            let species = &elements.species()[id];
            ElementRow {
                label: strings::get(&format!("element.{}", species.key)).to_string(),
                swatch: species.color,
                command: Command::SelectElement(id),
                selected: id == tools.species,
            }
        })
        .collect();
    LeftPanel {
        tools: ToolsSection {
            heading: strings::get("tools.heading").to_string(),
            buttons,
            size: SizeSlider {
                label: strings::get("tools.brush_size").to_string(),
                value: tools.radius,
                min: *RADIUS.start(),
                max: *RADIUS.end(),
            },
        },
        elements: ElementsSection { heading: strings::get("tools.element_heading").to_string(), rows },
        presets: PresetsSection {
            heading: strings::get("presets.heading").to_string(),
            buttons: Preset::ALL
                .iter()
                .map(|&preset| PresetButton {
                    label: strings::get(preset.label_key()).to_string(),
                    note: strings::get(preset.note_key()).to_string(),
                    command: PresetCommand::Arm(preset),
                    armed: presets.armed == Some(preset),
                })
                .collect(),
        },
        views: ViewsSection {
            heading: strings::get("views.heading").to_string(),
            buttons: VIEWS
                .iter()
                .map(|&v| ViewButton {
                    label: strings::get(view_label_key(v)).to_string(),
                    command: ViewCommand::Select(v),
                    selected: v == views.view,
                })
                .collect(),
        },
    }
}

/// The command a key sends (§4.10), none for a key the panel does not bind.
pub fn key_command(key: egui::Key) -> Option<Command> {
    use egui::Key;
    Some(match key {
        Key::B => Command::SelectTool(Tool::Brush),
        Key::E => Command::SelectTool(Tool::Eraser),
        Key::H => Command::SelectTool(Tool::Heat),
        Key::C => Command::SelectTool(Tool::Cool),
        Key::OpenBracket => Command::Smaller,
        Key::CloseBracket => Command::Larger,
        _ => return None,
    })
}

/// The command F1–F4 send (§4.10).
pub fn view_key_command(key: egui::Key) -> Option<ViewCommand> {
    use egui::Key;
    Some(ViewCommand::Select(match key {
        Key::F1 => View::Glow,
        Key::F2 => View::Heat,
        Key::F3 => View::Element,
        Key::F4 => View::Density,
        _ => return None,
    }))
}

/// The command Esc sends (§4.10): it disarms a preset, and nothing else is the presets' key.
pub fn preset_key_command(key: egui::Key) -> Option<PresetCommand> {
    (key == egui::Key::Escape).then_some(PresetCommand::Disarm)
}

/// The edit one frame asks for (§1.12): `cell` is the world cell under the pointer while the left button is held, none when it
/// is not. The window calls this once per frame, so the held button queues one edit per frame whatever the mouse sends.
pub fn frame_edit(tools: &Tools, cell: Option<[u32; 2]>) -> Option<Edit> {
    let cell = cell?;
    let radius = tools.radius;
    Some(match tools.tool {
        Tool::Brush => Edit::Paint { cell, radius, species: tools.species },
        Tool::Eraser => Edit::Erase { cell, radius },
        Tool::Heat => Edit::Heat { cell, radius },
        Tool::Cool => Edit::Cool { cell, radius },
    })
}

/// The world cell under `pos` when the world's image fills `rect` (§1.11), none outside it.
pub fn cell_at(pos: egui::Pos2, rect: egui::Rect, world: WorldConfig) -> Option<[u32; 2]> {
    if !rect.contains(pos) || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }
    let x = ((pos.x - rect.min.x) / rect.width() * world.width as f32).floor() as u32;
    let y = ((pos.y - rect.min.y) / rect.height() * world.height as f32).floor() as u32;
    (x < world.width && y < world.height).then_some([x, y])
}

/// The brush disk's outline over the world (§1.12, §4.2): the centre of `cell` and the radius, in screen points. The engine's
/// disk holds the cells whose indices lie within `radius` of the cell's, so the outline passes half a cell beyond the
/// outermost of them.
pub fn outline(tools: &Tools, rect: egui::Rect, world: WorldConfig, cell: [u32; 2]) -> (egui::Pos2, f32) {
    let scale = rect.width() / world.width as f32;
    let centre = rect.min + egui::vec2(cell[0] as f32 + 0.5, cell[1] as f32 + 0.5) * scale;
    (centre, (tools.radius as f32 + 0.5) * scale)
}

#[cfg(test)]
mod tools_tests {
    use super::*;

    fn elements() -> Elements {
        Elements::shipped().unwrap()
    }

    fn fresh() -> Tools {
        Tools::new(&elements())
    }

    fn panel(tools: &Tools) -> LeftPanel {
        view(tools, &Presets::default(), &Views::default(), &elements())
    }

    #[test]
    fn the_texts_are_byte_exact() {
        let p = panel(&fresh());
        assert_eq!(p.tools.heading, "Tools");
        let labels: Vec<&str> = p.tools.buttons.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, ["Brush", "Eraser", "Heat", "Cool"]);
        assert_eq!(p.tools.size.label, "Brush size");
        assert_eq!(p.elements.heading, "Element");
    }

    #[test]
    fn one_tool_is_selected_and_it_follows_the_commands() {
        let mut tools = fresh();
        let selected = |tools: &Tools| -> Vec<String> {
            panel(tools).tools.buttons.iter().filter(|b| b.selected).map(|b| b.label.clone()).collect()
        };
        assert_eq!(selected(&tools), ["Brush"]);
        for (tool, name) in [(Tool::Eraser, "Eraser"), (Tool::Heat, "Heat"), (Tool::Cool, "Cool"), (Tool::Brush, "Brush")] {
            let button = panel(&tools).tools.buttons.iter().find(|b| b.label == name).unwrap().command;
            assert_eq!(button, Command::SelectTool(tool));
            tools.apply(button);
            assert_eq!(selected(&tools), [name]);
        }
    }

    #[test]
    fn the_keys_are_mapped_and_nothing_else_is() {
        use egui::Key;
        assert_eq!(key_command(Key::B), Some(Command::SelectTool(Tool::Brush)));
        assert_eq!(key_command(Key::E), Some(Command::SelectTool(Tool::Eraser)));
        assert_eq!(key_command(Key::H), Some(Command::SelectTool(Tool::Heat)));
        assert_eq!(key_command(Key::C), Some(Command::SelectTool(Tool::Cool)));
        assert_eq!(key_command(Key::OpenBracket), Some(Command::Smaller));
        assert_eq!(key_command(Key::CloseBracket), Some(Command::Larger));
        for key in [Key::Space, Key::Period, Key::Num1, Key::Escape, Key::F1, Key::A, Key::Delete] {
            assert_eq!(key_command(key), None, "{key:?}");
        }
    }

    #[test]
    fn the_slider_runs_1_to_40_from_8() {
        let mut tools = fresh();
        let size = panel(&tools).tools.size;
        assert_eq!((size.value, size.min, size.max), (8, 1, 40));
        for (asked, got) in [(0, 1), (1, 1), (17, 17), (40, 40), (41, 40), (u32::MAX, 40)] {
            tools.apply(Command::SetRadius(asked));
            assert_eq!(tools.radius(), got, "asked {asked}");
            assert_eq!(panel(&tools).tools.size.value, got);
        }
    }

    #[test]
    fn the_brackets_step_by_two_and_stop_at_the_ends() {
        let mut tools = fresh();
        tools.apply(Command::Larger);
        assert_eq!(tools.radius(), 10);
        tools.apply(Command::Smaller);
        tools.apply(Command::Smaller);
        assert_eq!(tools.radius(), 6);
        tools.apply(Command::SetRadius(2));
        tools.apply(Command::Smaller);
        assert_eq!(tools.radius(), 1);
        tools.apply(Command::Smaller);
        assert_eq!(tools.radius(), 1);
        tools.apply(Command::SetRadius(39));
        tools.apply(Command::Larger);
        assert_eq!(tools.radius(), 40);
        tools.apply(Command::Larger);
        assert_eq!(tools.radius(), 40);
    }

    #[test]
    fn the_list_is_the_nine_in_order_and_never_neutron_matter() {
        let rows = panel(&fresh()).elements.rows;
        let names: Vec<&str> = rows.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(names, ["Hydrogen", "Helium", "Carbon", "Oxygen", "Neon", "Magnesium", "Silicon", "Sulfur", "Iron"]);
        assert!(rows.iter().all(|r| r.label != "Neutron matter"));
        let ids: Vec<Command> = rows.iter().map(|r| r.command).collect();
        assert_eq!(ids, (0..9).map(Command::SelectElement).collect::<Vec<_>>());
    }

    #[test]
    fn the_swatches_are_the_registry_colours() {
        let rows = panel(&fresh()).elements.rows;
        assert_eq!(rows[0].swatch, [0x7F, 0xB2, 0xFF], "Hydrogen");
        assert_eq!(rows[1].swatch, [0xFF, 0xE3, 0x8A], "Helium");
        assert_eq!(rows[8].swatch, [0xD9, 0x57, 0x2B], "Iron");
        let registry = elements();
        for (row, species) in rows.iter().zip(registry.species()) {
            assert_eq!(row.swatch, species.color, "{}", row.label);
        }
    }

    #[test]
    fn hydrogen_is_selected_at_first_and_neutron_matter_never_is() {
        let mut tools = fresh();
        let selected = |tools: &Tools| -> Vec<String> {
            panel(tools).elements.rows.iter().filter(|r| r.selected).map(|r| r.label.clone()).collect()
        };
        assert_eq!(selected(&tools), ["Hydrogen"]);
        tools.apply(Command::SelectElement(8));
        assert_eq!(selected(&tools), ["Iron"]);
        // Neutron matter is species 9: not in the list, so no command reaches it — a script's included.
        tools.apply(Command::SelectElement(9));
        tools.apply(Command::SelectElement(99));
        assert_eq!((tools.species(), selected(&tools)), (8, vec!["Iron".to_string()]));
        assert_eq!(tools.species_of(&elements(), "Fe"), Some(8));
        assert_eq!(tools.species_of(&elements(), "n"), None);
        assert_eq!(tools.species_of(&elements(), "Xx"), None);
    }

    #[test]
    fn a_change_of_tool_keeps_the_element_and_the_size() {
        let mut tools = fresh();
        tools.apply(Command::SelectElement(3));
        tools.apply(Command::SetRadius(21));
        for tool in Tool::ALL {
            tools.apply(Command::SelectTool(tool));
            assert_eq!((tools.species(), tools.radius()), (3, 21), "{tool:?}");
        }
        tools.apply(Command::SelectElement(0));
        assert_eq!((tools.tool(), tools.radius()), (Tool::Cool, 21));
    }

    #[test]
    fn a_held_button_queues_one_edit_per_frame_with_the_radius_and_species_of_that_frame() {
        let mut tools = fresh();
        tools.apply(Command::SelectElement(1));
        tools.apply(Command::SetRadius(5));
        // Frames, not events: a frame is one call, whatever the mouse did in it. Held on frames 0, 1, 3; released on 2.
        let mut queued = Vec::new();
        for (frame, cell) in [Some([10, 20]), Some([11, 20]), None, Some([30, 40])].into_iter().enumerate() {
            if frame == 1 {
                tools.apply(Command::SetRadius(7));
                tools.apply(Command::SelectElement(2));
            }
            queued.extend(frame_edit(&tools, cell));
        }
        assert_eq!(
            queued,
            [
                Edit::Paint { cell: [10, 20], radius: 5, species: 1 },
                Edit::Paint { cell: [11, 20], radius: 7, species: 2 },
                Edit::Paint { cell: [30, 40], radius: 7, species: 2 },
            ]
        );
    }

    #[test]
    fn each_tool_queues_its_own_edit_at_the_radius() {
        let mut tools = fresh();
        tools.apply(Command::SetRadius(12));
        let c = Some([5, 6]);
        let want = [
            (Tool::Brush, Edit::Paint { cell: [5, 6], radius: 12, species: 0 }),
            (Tool::Eraser, Edit::Erase { cell: [5, 6], radius: 12 }),
            (Tool::Heat, Edit::Heat { cell: [5, 6], radius: 12 }),
            (Tool::Cool, Edit::Cool { cell: [5, 6], radius: 12 }),
        ];
        for (tool, edit) in want {
            tools.apply(Command::SelectTool(tool));
            assert_eq!(frame_edit(&tools, c), Some(edit), "{tool:?}");
        }
        assert_eq!(frame_edit(&tools, None), None);
    }

    #[test]
    fn the_pointer_maps_to_cells_and_the_outline_follows_the_cell() {
        let world = WorldConfig { width: 600, height: 400 };
        let rect = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(1200.0, 800.0));
        assert_eq!(cell_at(egui::pos2(100.0, 50.0), rect, world), Some([0, 0]));
        assert_eq!(cell_at(egui::pos2(101.9, 51.9), rect, world), Some([0, 0]));
        assert_eq!(cell_at(egui::pos2(102.0, 52.0), rect, world), Some([1, 1]));
        assert_eq!(cell_at(egui::pos2(1299.9, 849.9), rect, world), Some([599, 399]));
        assert_eq!(cell_at(egui::pos2(1300.0, 850.0), rect, world), None);
        assert_eq!(cell_at(egui::pos2(99.0, 60.0), rect, world), None);
        let mut tools = fresh();
        tools.apply(Command::SetRadius(10));
        let (centre, radius) = outline(&tools, rect, world, [300, 200]);
        assert_eq!((centre, radius), (egui::pos2(100.0 + 601.0, 50.0 + 401.0), 21.0));
    }

    fn views_of(views: &Views) -> ViewsSection {
        view(&fresh(), &Presets::default(), views, &elements()).views
    }

    #[test]
    fn the_views_texts_are_byte_exact() {
        let section = views_of(&Views::default());
        assert_eq!(section.heading, "View");
        let labels: Vec<&str> = section.buttons.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, ["Glow", "Heat", "Element", "Density"]);
    }

    #[test]
    fn glow_is_active_at_first_and_one_view_is_selected_after_each_command() {
        assert_eq!(Views::default().active(), View::Glow);
        let mut views = Views::default();
        for (i, button) in views_of(&views).buttons.iter().enumerate() {
            assert_eq!(button.selected, i == 0);
        }
        for want in VIEWS {
            views.apply(ViewCommand::Select(want));
            let section = views_of(&views);
            let selected: Vec<View> = section
                .buttons
                .iter()
                .filter(|b| b.selected)
                .map(|b| match b.command {
                    ViewCommand::Select(v) => v,
                })
                .collect();
            assert_eq!((selected, views.active()), (vec![want], want));
        }
    }

    #[test]
    fn each_button_sends_its_own_view() {
        let sent: Vec<View> = views_of(&Views::default()).buttons.iter().map(|b| match b.command {
            ViewCommand::Select(v) => v,
        }).collect();
        assert_eq!(sent, [View::Glow, View::Heat, View::Element, View::Density]);
    }

    #[test]
    fn f1_to_f4_pick_the_views_and_nothing_else_does() {
        use egui::Key;
        let picked: Vec<_> = [Key::F1, Key::F2, Key::F3, Key::F4].map(view_key_command).to_vec();
        let want = [View::Glow, View::Heat, View::Element, View::Density].map(|v| Some(ViewCommand::Select(v)));
        assert_eq!(picked, want);
        for key in [Key::F5, Key::Num1, Key::B, Key::H, Key::E, Key::Escape, Key::Space] {
            assert_eq!(view_key_command(key), None, "{key:?}");
        }
        assert_eq!(parse_view("density"), Some(View::Density));
        assert_eq!(parse_view("Glow"), None);
    }

    #[test]
    fn a_view_switch_leaves_the_brush_the_element_and_the_size_alone() {
        let mut tools = fresh();
        tools.apply(Command::SelectTool(Tool::Heat));
        tools.apply(Command::SetRadius(21));
        let iron = tools.species_of(&elements(), "Fe").unwrap();
        tools.apply(Command::SelectElement(iron));
        let before = tools.clone();
        let mut views = Views::default();
        for v in VIEWS {
            views.apply(ViewCommand::Select(v));
            assert_eq!(tools, before, "{v:?}");
            let panel = view(&tools, &Presets::default(), &views, &elements());
            assert_eq!(panel.tools, view(&before, &Presets::default(), &Views::default(), &elements()).tools);
            assert_eq!(panel.elements, view(&before, &Presets::default(), &Views::default(), &elements()).elements);
        }
        assert_eq!((tools.tool(), tools.species(), tools.radius()), (Tool::Heat, iron, 21));
    }

    #[test]
    fn a_tool_or_element_change_leaves_the_view_alone() {
        let mut views = Views::default();
        views.apply(ViewCommand::Select(View::Density));
        let mut tools = fresh();
        tools.apply(Command::SelectTool(Tool::Cool));
        tools.apply(Command::Larger);
        assert_eq!(views.active(), View::Density);
    }
}

#[cfg(test)]
mod presets_tests {
    use super::*;
    use sr_engine::step::edit::EditError;
    use sr_physics::calibration;
    use sr_physics::registry::Physics;

    fn elements() -> Elements {
        Elements::shipped().unwrap()
    }

    fn panel(tools: &Tools, presets: &Presets) -> LeftPanel {
        view(tools, presets, &Views::default(), &elements())
    }

    fn armed(presets: &Presets) -> Vec<String> {
        panel(&Tools::new(&elements()), presets).presets.buttons.iter().filter(|b| b.armed).map(|b| b.label.clone()).collect()
    }

    fn masses() -> calibration::PresetMasses {
        calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("the shipped calibration").preset_mass_sb
    }

    fn mass_of(preset: Preset) -> f64 {
        let m = masses();
        match preset {
            Preset::Sun => m.sun,
            Preset::Massive => m.massive,
            Preset::Giant => m.giant,
        }
    }

    #[test]
    fn the_texts_are_byte_exact() {
        let section = panel(&Tools::new(&elements()), &Presets::default()).presets;
        assert_eq!(section.heading, "Drop a star");
        let rows: Vec<(&str, &str)> = section.buttons.iter().map(|b| (b.label.as_str(), b.note.as_str())).collect();
        assert_eq!(
            rows,
            [
                ("Sun-like star", "1 Sun · ends as a white dwarf"),
                ("Massive star", "15 Suns · ends in a supernova"),
                ("Giant star", "40 Suns · ends as a black hole"),
            ]
        );
        let mut presets = Presets::default();
        presets.apply(PresetCommand::Arm(Preset::Sun));
        assert_eq!(presets.notices(0.0), ["Click in the world to drop the star. Esc cancels."]);
        presets.refused(0.0);
        assert_eq!(presets.notices(1.0)[1], "Not enough room here for this star.");
    }

    #[test]
    fn a_button_arms_its_preset_and_only_one_is_armed() {
        let mut presets = Presets::default();
        assert_eq!((presets.armed(), armed(&presets)), (None, Vec::<String>::new()));
        let commands: Vec<PresetCommand> = panel(&Tools::new(&elements()), &presets).presets.buttons.iter().map(|b| b.command).collect();
        assert_eq!(commands, Preset::ALL.map(PresetCommand::Arm));
        for (preset, name) in [(Preset::Massive, "Massive star"), (Preset::Giant, "Giant star"), (Preset::Sun, "Sun-like star")] {
            presets.apply(PresetCommand::Arm(preset));
            assert_eq!((presets.armed(), armed(&presets)), (Some(preset), vec![name.to_string()]));
        }
        // The names are the engine's, so a script's `{"preset": …}` and a button reach the same cloud.
        assert_eq!(Preset::ALL.map(Preset::name), sr_engine::scene::PRESETS);
        assert_eq!(Preset::parse("giant"), Some(Preset::Giant));
        assert_eq!(Preset::parse("Giant"), None);
    }

    #[test]
    fn arming_leaves_the_tools_alone() {
        let tools = Tools::new(&elements());
        let (before, after) = (panel(&tools, &Presets::default()), panel(&tools, &Presets { armed: Some(Preset::Sun), ..Presets::default() }));
        assert_eq!((&before.tools, &before.elements), (&after.tools, &after.elements));
    }

    #[test]
    fn the_hint_shows_while_armed_and_a_drop_spends_the_click() {
        let mut presets = Presets::default();
        assert!(presets.notices(0.0).is_empty());
        presets.apply(PresetCommand::Arm(Preset::Massive));
        assert_eq!(presets.notices(100.0).len(), 1, "the hint has no time limit");
        presets.placed();
        assert_eq!((presets.armed(), presets.notices(100.0)), (None, vec![]));
    }

    #[test]
    fn a_refusal_shows_for_three_seconds_and_the_preset_stays_armed() {
        let mut presets = Presets::default();
        presets.apply(PresetCommand::Arm(Preset::Giant));
        presets.refused(10.0);
        assert_eq!(presets.armed(), Some(Preset::Giant));
        for (now, lines) in [(10.0, 2), (12.9, 2), (13.0, 1), (60.0, 1)] {
            assert_eq!(presets.notices(now).len(), lines, "at {now}");
        }
        // A second refusal starts its own 3 s; arming another preset takes the line away at once.
        presets.refused(20.0);
        assert_eq!(presets.notices(22.9).len(), 2);
        presets.apply(PresetCommand::Arm(Preset::Sun));
        assert_eq!(presets.notices(20.5).len(), 1);
        // Refused with nothing armed (a cleared preset under a late reply): the line still ends at 3 s, without the hint.
        let mut late = Presets::default();
        late.refused(5.0);
        assert_eq!((late.notices(6.0).len(), late.notices(8.0).len()), (1, 0));
    }

    #[test]
    fn esc_disarms_and_no_other_key_is() {
        use egui::Key;
        assert_eq!(preset_key_command(Key::Escape), Some(PresetCommand::Disarm));
        for key in [Key::Space, Key::Period, Key::Num1, Key::F1, Key::B, Key::Delete, Key::Enter] {
            assert_eq!(preset_key_command(key), None, "{key:?}");
        }
        assert_eq!(key_command(Key::Escape), None, "Esc is not a tool's key");
        let mut presets = Presets::default();
        presets.apply(PresetCommand::Arm(Preset::Giant));
        presets.refused(1.0);
        presets.apply(preset_key_command(Key::Escape).unwrap());
        assert_eq!((presets.armed(), armed(&presets), presets.notices(1.5)), (None, vec![], vec![]));
        // Esc with nothing armed changes nothing.
        presets.apply(PresetCommand::Disarm);
        assert_eq!(presets, Presets::default());
    }

    /// The outline and the refusal read one cloud, at the radius the engine lays it out with (§2.10: 40 cells × √(M / M₀),
    /// capped at 150) — so a cloud the outline shows inside the world is accepted, and one it shows sticking out is refused
    /// with `NoRoom`. The shipped masses stay under the cap (39, 56 and 93 cells), so a mass above it joins the grid: an
    /// outline drawn at the uncapped 253 cells would call a cloud that fits «no room».
    #[test]
    fn the_outline_is_the_clouds_radius_and_agrees_with_the_refusal() {
        use sr_engine::scene::{MAX_RADIUS, UNIT_MASS};
        let (physics, elements) = (Physics::shipped().unwrap(), elements());
        let world = WorldConfig { width: 600, height: 400 };
        let rect = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(1200.0, 800.0));
        let uncapped = |mass: f64| 40.0 * (mass / UNIT_MASS).sqrt();
        let heavy = 40.0 * UNIT_MASS;
        assert!(uncapped(heavy) > 200.0, "the cap matters for the heavy cloud: {}", uncapped(heavy));
        let clouds: Vec<(Preset, f64)> = Preset::ALL.iter().map(|&p| (p, mass_of(p))).chain([(Preset::Giant, heavy)]).collect();
        for &(preset, mass) in &clouds {
            assert_eq!(cloud_radius(mass), uncapped(mass).min(MAX_RADIUS), "{preset:?} at {mass}");
        }
        assert_eq!(cloud_radius(heavy), 150.0);
        let cells = [[300, 200], [140, 200], [160, 200], [300, 100], [300, 260], [20, 20], [599, 399], [60, 200], [100, 200], [200, 120]];
        let mut outcomes = (0, 0);
        for &(preset, mass) in &clouds {
            let radius = cloud_radius(mass);
            for cell in cells {
                let (centre, r) = cloud_outline(rect, world, cell, radius);
                assert_eq!(r, radius as f32 * 2.0, "{preset:?} at {cell:?}: the outline is drawn at the cloud's radius");
                let inside = centre.x - r >= rect.min.x && centre.x + r <= rect.max.x && centre.y - r >= rect.min.y && centre.y + r <= rect.max.y;
                match Edit::drop_preset(preset.name(), mass, cell, world, &physics, &elements) {
                    Ok(_) => {
                        assert!(inside, "{preset:?} ({mass}) at {cell:?} dropped though its outline leaves the world");
                        outcomes.0 += 1;
                    }
                    Err(EditError::NoRoom) => {
                        assert!(!inside, "{preset:?} ({mass}) at {cell:?} refused though its outline is inside the world");
                        outcomes.1 += 1;
                    }
                    Err(e) => panic!("{preset:?} at {cell:?}: {e}"),
                }
            }
        }
        assert!(outcomes.0 >= 8 && outcomes.1 >= 8, "both arms are seen: {outcomes:?}");
    }
}
