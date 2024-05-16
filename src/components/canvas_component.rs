use std::{
    any::TypeId,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gamezap::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        concepts::ConceptManager,
        entity::EntityId,
        scene::AllComponents,
    },
    texture::Texture,
    EngineDetails, EngineSystems,
};

use rfd::FileDialog;
use wgpu::{Device, Queue};

#[derive(Debug, Clone)]
struct ImageData {
    path: String,
    id: imgui::TextureId,
    size: [f32; 2],
}

#[derive(Debug, Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
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

#[derive(Debug, Clone, Default)]
struct TileData {
    north_valid_tiles: Vec<TileConnection>,
    south_valid_tiles: Vec<TileConnection>,
    east_valid_tiles: Vec<TileConnection>,
    west_valid_tiles: Vec<TileConnection>,
}

impl ImageData {
    fn new(path: String, id: imgui::TextureId, size: [f32; 2]) -> Self {
        Self { path, id, size }
    }
}

#[derive(Debug, Clone)]
pub struct CanvasComponent {
    parent: EntityId,
    id: ComponentId,
    loaded_images: Vec<ImageData>,
    loaded_tiles: Vec<TileData>,
    tile_being_modified: Option<usize>,
    selected_direction: Option<Direction>,
    some_num: u32,
}

impl Default for CanvasComponent {
    fn default() -> Self {
        CanvasComponent {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            loaded_images: Vec::new(),
            loaded_tiles: Vec::new(),
            tile_being_modified: None,
            selected_direction: None,
            some_num: 0,
        }
    }
}

impl ComponentSystem for CanvasComponent {
    fn initialize(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
    ) {
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        _engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        if engine_systems
            .lock()
            .unwrap()
            .sdl_context
            .mouse()
            .is_cursor_showing()
        {
            let systems = engine_systems.lock().unwrap();
            let mut ui_manager = systems.ui_manager.lock().unwrap();
            ui_manager.set_render_flag();

            let mut imgui_context = ui_manager.imgui_context.lock().unwrap();

            let ui = imgui_context.new_frame();

            let mut images = self.loaded_images.clone();
            let mut tiles = self.loaded_tiles.clone();

            ui.window("Main window")
                .title_bar(false)
                .position([20.0, 20.0], imgui::Condition::Always)
                .resizable(false)
                .size([560.0, 220.0], imgui::Condition::Always)
                .bring_to_front_on_focus(false)
                .focused(false)
                .collapsible(false)
                .build(|| {
                    let style = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
                    let button_style =
                        ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.5, 0.5, 0.5, 0.5]);
                    let button_style_2 =
                        ui.push_style_color(imgui::StyleColor::ButtonActive, [0.8, 0.8, 0.8, 0.5]);
                    for (i, ImageData { path, id, size }) in
                        self.loaded_images.clone().iter().enumerate()
                    {
                        /* images.push(ImageData::new(path.clone(), *id, *size));
                        tiles.push(self.loaded_tiles); */
                        ui.window(format!("Image {i}"))
                            .resizable(false)
                            .always_auto_resize(true)
                            .title_bar(false)
                            .position([110.0 * i as f32 + 30.0, 30.0], imgui::Condition::Always)
                            .always_use_window_padding(false)
                            .build(|| {
                                let aspect_ratio = size[1] / size[0];

                                let text = format!("{i}");
                                let frame_padding = unsafe { ui.style().frame_padding[0] * 2.0 };
                                let size = ui.calc_text_size(&text)[0] + frame_padding;
                                let available = ui.content_region_avail()[0];
                                let off = (available - size) * 0.5;
                                if off > 0.0 {
                                    ui.set_cursor_pos([
                                        ui.cursor_pos()[0] + off,
                                        ui.cursor_pos()[1],
                                    ]);
                                }
                                ui.text(text);
                                ui.separator();
                                if ui.image_button(
                                    "test image button",
                                    *id,
                                    [100.0, 100.0 * aspect_ratio],
                                ) {
                                    self.tile_being_modified = Some(i);
                                }

                                ui.separator();
                                let frame_padding = unsafe { ui.style().frame_padding[0] * 2.0 };
                                let size = ui.calc_text_size("Remove image")[0] + frame_padding;
                                let available = ui.content_region_avail()[0];
                                let off = (available - size) * 0.5;
                                if off > 0.0 {
                                    ui.set_cursor_pos([
                                        ui.cursor_pos()[0] + off,
                                        ui.cursor_pos()[1],
                                    ]);
                                }

                                if ui.button("Remove image") {
                                    images.remove(i);
                                    tiles.remove(i);
                                }
                                ui.spacing()
                            });
                    }
                    style.pop();

                    ui.window("image selector")
                        .title_bar(false)
                        .resizable(false)
                        .movable(false)
                        .draw_background(false)
                        .always_auto_resize(true)
                        .position([30.0, 200.0], imgui::Condition::Always)
                        .build(|| {
                            if ui.button("Load image") {
                                let file = FileDialog::new().pick_file();
                                if let Some(path) = file {
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
                                    tiles.push(TileData::default());
                                }
                            }
                        });
                    button_style.pop();
                    button_style_2.pop();
                });

            if let Some(tile_index) = self.tile_being_modified {
                let ImageData { id, size, .. } = &self.loaded_images[tile_index];

                ui.window("Modifying tile")
                    .collapsible(false)
                    .movable(false)
                    .position([20.0, 240.0], imgui::Condition::Always)
                    .size([560.0, 250.0], imgui::Condition::Always)
                    .build(|| {
                        let padding =
                            unsafe { ui.style().frame_padding[0] + ui.style().window_padding[0] };
                        ui.columns(3, "Editor cols", true);
                        ui.set_column_width(0, 100.0 + padding);
                        ui.set_column_width(1, 350.0);
                        let aspect_ratio = size[1] / size[0];
                        imgui::Image::new(*id, [100.0, 100.0 * aspect_ratio]).build(ui);
                        ui.next_column();
                        let directions_bar = ui.tab_bar("Tile directions").unwrap();
                        for dir in 0..4 {
                            let mut temp_vec = Vec::new();
                            let tab_data = match dir {
                                0 => ("North", &mut tiles[tile_index].north_valid_tiles),
                                1 => ("South", &mut tiles[tile_index].south_valid_tiles),
                                2 => ("East", &mut tiles[tile_index].east_valid_tiles),
                                3 => ("West", &mut tiles[tile_index].west_valid_tiles),
                                _ => ("", &mut temp_vec),
                            };
                            if let Some(dir_tab) = ui.tab_item(tab_data.0) {
                                if tab_data.1.is_empty() {
                                    ui.text("No existing connections for this direction");
                                } else {
                                    let bar = ui.tab_bar("Thing").unwrap();
                                    for (i, (index, direction)) in
                                        tab_data.1.clone().iter().enumerate()
                                    {
                                        if let Some(item) = ui.tab_item(format!("{index}")) {
                                            ui.text(format!("{direction}"));
                                            if ui.button("Remove") {
                                                tab_data.1.remove(i);
                                            }
                                            ui.separator();
                                            ui.spacing();
                                            item.end();
                                        }
                                    }
                                    bar.end();
                                }

                                if ui.button("Add") {
                                    tab_data.1.push((dir, Direction::North));
                                    // existing_connections.push((0, Direction::North));
                                }
                                dir_tab.end();
                            }
                        }
                        directions_bar.end();
                        /* if ui.button("North") {
                            self.selected_direction = Some(Direction::North)
                        }
                        if ui.button("South") {
                            self.selected_direction = Some(Direction::South)
                        }
                        if ui.button("East") {
                            self.selected_direction = Some(Direction::East)
                        }
                        if ui.button("West") {
                            self.selected_direction = Some(Direction::West)
                        }
                        ui.next_column();
                        if let Some(direction) = &self.selected_direction {
                            let TileData {
                                north_valid_tiles,
                                south_valid_tiles,
                                east_valid_tiles,
                                west_valid_tiles,
                            } = &mut tiles[tile_index];

                            let existing_connections = match direction {
                                Direction::North => north_valid_tiles,
                                Direction::South => south_valid_tiles,
                                Direction::East => east_valid_tiles,
                                Direction::West => west_valid_tiles,
                            };
                            // dbg!(&existing_connections);
                        } */
                        if ui.button("Close") {
                            self.tile_being_modified = None;
                            self.selected_direction = None;
                        }
                    });
            }

            self.loaded_images = images;
            self.loaded_tiles = tiles;
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
