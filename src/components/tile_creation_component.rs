use std::{
    any::{Any, TypeId},
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gamezap::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        concepts::ConceptManager,
        entity::{Entity, EntityId},
        scene::AllComponents,
    },
    texture::Texture,
    EngineDetails, EngineSystems,
};

use rfd::FileDialog;
use wgpu::{Device, Queue};

#[derive(Debug, Clone)]
pub struct ImageData {
    _path: String,
    pub id: imgui::TextureId,
    size: [f32; 2],
}

impl ImageData {
    pub fn new(path: String, id: imgui::TextureId, size: [f32; 2]) -> Self {
        Self {
            _path: path,
            id,
            size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl From<usize> for Direction {
    fn from(value: usize) -> Self {
        match value {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            3 => Direction::West,
            _ => Direction::North,
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::South => write!(f, "South"),
            Direction::East => write!(f, "East"),
            Direction::West => write!(f, "West"),
        }
    }
}

pub type TileConnection = (usize, Direction);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileData {
    pub image_index: usize,
    pub north_valid_tiles: Vec<TileConnection>,
    pub south_valid_tiles: Vec<TileConnection>,
    pub east_valid_tiles: Vec<TileConnection>,
    pub west_valid_tiles: Vec<TileConnection>,
}

impl TileData {
    pub fn new(image_index: usize) -> Self {
        Self {
            image_index,
            north_valid_tiles: Vec::new(),
            south_valid_tiles: Vec::new(),
            east_valid_tiles: Vec::new(),
            west_valid_tiles: Vec::new(),
        }
    }

    pub fn total_connections(&self) -> usize {
        self.north_valid_tiles.len()
            + self.south_valid_tiles.len()
            + self.east_valid_tiles.len()
            + self.west_valid_tiles.len()
    }
}

#[derive(Debug, Clone)]
pub struct TileCreationComponent {
    parent: EntityId,
    id: ComponentId,
    concept_ids: Vec<String>,
    tile_being_modified: Option<usize>,
    selected_direction: Option<Direction>,
    tile_selected: usize,
    direction_selected: usize,
    run_algorithm: bool,
}

impl TileCreationComponent {
    pub fn new(concept_manager: Rc<Mutex<ConceptManager>>) -> Self {
        let mut comp = Self {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            concept_ids: Vec::new(),
            tile_being_modified: None,
            selected_direction: None,
            tile_selected: 0,
            direction_selected: 0,
            run_algorithm: false,
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert(
            "loaded_images".to_string(),
            Box::<Vec<ImageData>>::default(),
        );
        concepts.insert("loaded_tiles".to_string(), Box::<Vec<TileData>>::default());

        comp.register_component(concept_manager, concepts);

        comp
    }
}

impl ComponentSystem for TileCreationComponent {
    // Registers all of the component concepts
    // This makes it possible to share data between components
    fn register_component(
        &mut self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        data: HashMap<String, Box<dyn Any>>,
    ) {
        self.concept_ids = data.keys().cloned().collect();

        concept_manager
            .lock()
            .unwrap()
            .register_component_concepts(self.id, data);
    }

    // Main update method
    // This is called every frame
    fn update(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        _engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        entities: &mut Vec<Entity>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        entities[1].enabled = !concept_manager
            .get_concept::<Vec<TileData>>(self.id, "loaded_tiles".to_string())
            .unwrap()
            .is_empty()
            && self.run_algorithm;
    }

    // Main UI draw method
    // Called every frame
    // This can definitely be split up into functions
    // (I don't have time)
    fn ui_draw(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        ui_manager: &mut gamezap::ui_manager::UiManager,
        ui_frame: &mut imgui::Ui,
        _component_map: &mut AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        if engine_systems
            .lock()
            .unwrap()
            .sdl_context
            .mouse()
            .is_cursor_showing()
        {
            let mut images = concept_manager
                .get_concept::<Vec<ImageData>>(self.id, "loaded_images".to_string())
                .unwrap()
                .clone();
            let mut tiles = concept_manager
                .get_concept::<Vec<TileData>>(self.id, "loaded_tiles".to_string())
                .unwrap()
                .clone();

            ui_frame
                .window("Main window")
                .title_bar(false)
                .position([20.0, 20.0], imgui::Condition::Always)
                .resizable(false)
                .size([560.0, 220.0], imgui::Condition::Always)
                .scrollable(true)
                .bring_to_front_on_focus(false)
                .focused(false)
                .collapsible(false)
                .build(|| {
                    let style = ui_frame.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
                    let button_style = ui_frame
                        .push_style_color(imgui::StyleColor::ButtonHovered, [0.5, 0.5, 0.5, 0.5]);
                    let button_style_2 = ui_frame
                        .push_style_color(imgui::StyleColor::ButtonActive, [0.8, 0.8, 0.8, 0.5]);
                    let image_table = ui_frame
                        .begin_table_with_flags(
                            "Image table",
                            images.len().max(1),
                            imgui::TableFlags::SIZING_FIXED_FIT
                                | imgui::TableFlags::NO_BORDERS_IN_BODY,
                        )
                        .unwrap();
                    ui_frame.table_next_row();
                    ui_frame.table_set_column_index(0);
                    for (i, ImageData { id, size, .. }) in images.clone().iter().enumerate() {
                        if i % 4 == 0 {
                            ui_frame.table_next_row();
                            ui_frame.table_set_column_index(0);
                        }
                        let aspect_ratio = size[1] / size[0];
                        if aspect_ratio != 1.0 {
                            panic!("Select a square image");
                        }

                        let text = format!("{i}");
                        let frame_padding = unsafe { ui_frame.style().frame_padding[0] * 2.0 };
                        let size = ui_frame.calc_text_size(&text)[0] + frame_padding;
                        let available = ui_frame.content_region_avail()[0];
                        let off = (available - size) * 0.5;
                        if off > 0.0 {
                            ui_frame.set_cursor_pos([
                                ui_frame.cursor_pos()[0] + off,
                                ui_frame.cursor_pos()[1],
                            ]);
                        }
                        ui_frame.text(text);
                        ui_frame.separator();
                        if ui_frame.image_button(format!("Image button {i}"), *id, [100.0, 100.0]) {
                            self.tile_being_modified = Some(i);
                        }

                        ui_frame.separator();
                        let frame_padding = unsafe { ui_frame.style().frame_padding[0] * 2.0 };
                        let size =
                            ui_frame.calc_text_size(format!("Remove image {i}"))[0] + frame_padding;
                        let available = ui_frame.content_region_avail()[0];
                        let off = (available - size) * 0.5;
                        if off > 0.0 {
                            ui_frame.set_cursor_pos([
                                ui_frame.cursor_pos()[0] + off,
                                ui_frame.cursor_pos()[1],
                            ]);
                        }

                        if ui_frame.button(format!("Remove image {i}")) {
                            images.remove(i);
                            tiles.remove(i);
                        }
                        ui_frame.spacing();
                        ui_frame.table_next_column();
                    }
                    style.pop();

                    ui_frame.table_next_row();

                    ui_frame
                        .window("image selector")
                        .title_bar(false)
                        .resizable(false)
                        .movable(false)
                        .draw_background(false)
                        .always_auto_resize(true)
                        .position([450.0, 180.0], imgui::Condition::Always)
                        .build(|| {
                            if ui_frame.button("Load image") {
                                let file = FileDialog::new().pick_files();
                                if let Some(paths) = file {
                                    for path in paths {
                                        let (id, size) = Texture::load_ui_image(
                                            &device,
                                            &queue,
                                            &mut ui_manager.imgui_renderer.lock().unwrap(),
                                            (*path.to_str().unwrap()).to_owned(),
                                        );
                                        images.push(ImageData::new(
                                            path.to_str().unwrap().to_owned(),
                                            id,
                                            size,
                                        ));
                                        tiles.push(TileData::new(images.len() - 1));
                                    }
                                }
                            }
                            ui_frame.checkbox("Run algorithm", &mut self.run_algorithm);
                        });
                    button_style.pop();
                    button_style_2.pop();
                    image_table.end();
                });

            if let Some(tile_index) = self.tile_being_modified {
                let ImageData { id, size, .. } = images[tile_index];

                ui_frame
                    .window("Modifying tile")
                    .collapsible(false)
                    .movable(false)
                    .position([20.0, 240.0], imgui::Condition::Always)
                    .size([560.0, 250.0], imgui::Condition::Always)
                    .build(|| {
                        if let Some(main_table) = ui_frame.begin_table_with_flags(
                            "Main table",
                            2,
                            imgui::TableFlags::NO_BORDERS_IN_BODY
                                | imgui::TableFlags::SIZING_FIXED_FIT,
                        ) {
                            ui_frame.table_next_row();
                            ui_frame.table_set_column_index(0);
                            let aspect_ratio = size[1] / size[0];
                            imgui::Image::new(id, [100.0, 100.0 * aspect_ratio]).build(ui_frame);
                            ui_frame.table_next_column();
                            if let Some(directions_bar) = ui_frame.tab_bar("Tile directions") {
                                for dir in 0..4 {
                                    let mut temp_vec = Vec::new();
                                    let tab_data = match dir {
                                        0 => ("North", &mut tiles[tile_index].north_valid_tiles),
                                        1 => ("South", &mut tiles[tile_index].south_valid_tiles),
                                        2 => ("East", &mut tiles[tile_index].east_valid_tiles),
                                        3 => ("West", &mut tiles[tile_index].west_valid_tiles),
                                        _ => ("", &mut temp_vec),
                                    };
                                    if let Some(dir_tab) = ui_frame.tab_item(tab_data.0) {
                                        if tab_data.1.is_empty() {
                                            ui_frame
                                                .text("No existing connections for this direction");
                                        } else {
                                            let bar = ui_frame.tab_bar("Thing").unwrap();
                                            for (i, (index, direction)) in
                                                tab_data.1.clone().iter().enumerate()
                                            {
                                                if let Some(item) =
                                                    ui_frame.tab_item(format!("{index}"))
                                                {
                                                    ui_frame.text(format!("{direction}"));
                                                    if ui_frame.button("Remove") {
                                                        tab_data.1.remove(i);
                                                    }
                                                    ui_frame.separator();
                                                    ui_frame.spacing();
                                                    item.end();
                                                }
                                            }
                                            bar.end();
                                        }

                                        if let Some(table) = ui_frame.begin_table_with_flags(
                                            "modification table",
                                            3,
                                            imgui::TableFlags::NO_BORDERS_IN_BODY
                                                | imgui::TableFlags::SIZING_FIXED_FIT,
                                        ) {
                                            ui_frame.table_next_row();
                                            ui_frame.table_set_column_index(0);
                                            ui_frame.text("Tile id: ");
                                            ui_frame.table_next_column();
                                            let input_width = ui_frame.push_item_width(50.0);
                                            ui_frame
                                                .input_scalar("|", &mut self.tile_selected)
                                                .build();
                                            input_width.end();
                                            ui_frame.table_next_column();
                                            ui_frame.combo_simple_string(
                                                "Side",
                                                &mut self.direction_selected,
                                                &["North", "South", "East", "West"],
                                            );
                                            // ui.table_next_row();
                                            ui_frame.new_line();

                                            let tile_being_added = (
                                                self.tile_selected,
                                                Direction::from(self.direction_selected),
                                            );
                                            if ui_frame.button("Add")
                                                && !tab_data.1.contains(&tile_being_added)
                                            {
                                                tab_data.1.push(tile_being_added);
                                            }
                                            table.end();
                                        }
                                        dir_tab.end();
                                    }
                                }
                                directions_bar.end();
                            }
                            main_table.end();
                        }
                        if ui_frame.button("Close") {
                            self.tile_being_modified = None;
                            self.selected_direction = None;
                        }
                    });
            }
            *concept_manager
                .get_concept_mut::<Vec<ImageData>>(self.id, "loaded_images".to_string())
                .unwrap() = images;

            *concept_manager
                .get_concept_mut::<Vec<TileData>>(self.id, "loaded_tiles".to_string())
                .unwrap() = tiles;
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32) {
        self.parent = parent;
        self.id.0 = parent;
        self.id.2 = same_component_count;
    }

    fn get_parent_entity(&self) -> EntityId {
        self.parent
    }

    fn get_id(&self) -> ComponentId {
        self.id
    }
}
