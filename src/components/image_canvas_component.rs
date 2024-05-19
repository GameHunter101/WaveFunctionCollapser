use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::{Arc, Mutex},
    time::Instant,
};

use gamezap::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        concepts::ConceptManager,
        entity::{Entity, EntityId},
        scene::AllComponents,
    },
    EngineDetails, EngineSystems,
};

use rand::Rng;
use wgpu::{Device, Queue};

use super::tile_creation_component::{
    Direction, ImageData, TileConnection, TileCreationComponent, TileData,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PossibleConnections {
    north_connections: Vec<TileConnection>,
    south_connections: Vec<TileConnection>,
    east_connections: Vec<TileConnection>,
    west_connections: Vec<TileConnection>,
}

impl PossibleConnections {
    fn total_len(&self) -> usize {
        self.north_connections.len()
            + self.south_connections.len()
            + self.east_connections.len()
            + self.west_connections.len()
    }

    pub fn guaranteed_connections<'a>(
        &'a self,
        tiles: &'a [TileData],
    ) -> HashMap<&'a TileData, usize> {
        let mut guaranteed = HashMap::new();
        let mut indices = Vec::new();
        if self.north_connections.len() == 1 {
            let index = tiles
                .iter()
                .position(|tile| tile.image_index == self.north_connections[0].0)
                .unwrap();
            indices.push(index);
        }
        if self.south_connections.len() == 1 {
            let index = tiles
                .iter()
                .position(|tile| tile.image_index == self.south_connections[0].0)
                .unwrap();
            indices.push(index);
        }
        if self.east_connections.len() == 1 {
            let index = tiles
                .iter()
                .position(|tile| tile.image_index == self.east_connections[0].0)
                .unwrap();
            indices.push(index);
        }
        if self.west_connections.len() == 1 {
            let index = tiles
                .iter()
                .position(|tile| tile.image_index == self.west_connections[0].0)
                .unwrap();
            indices.push(index);
        }
        for index in indices {
            let tile = &tiles[index];
            if let Some(val) = guaranteed.get_mut(tile) {
                *val += 1;
            } else {
                guaranteed.insert(&tiles[index], 1);
            }
        }
        guaranteed
    }

    fn random_tile<'a>(&'a self, tiles: &'a [TileData]) -> &'a TileData {
        if self == &Self::default() {
            return &tiles[0];
        }
        let mut rng = rand::thread_rng();
        loop {
            let dir_f: f32 = rng.gen();
            let dir = (dir_f * 4.0) as usize;
            let temp_vec = Vec::new();
            let arr = match dir {
                0 => &self.north_connections,
                1 => &self.south_connections,
                2 => &self.east_connections,
                3 => &self.west_connections,
                _ => &temp_vec,
            };

            if arr.is_empty() {
                continue;
            }
            let index_f: f32 = rng.gen();
            let index = (index_f * arr.len() as f32) as usize;
            return &tiles[arr[index].0];
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageCanvasComponent {
    parent: EntityId,
    id: ComponentId,
    current_tile_set: Vec<TileData>,
    canvas_connections: [[PossibleConnections; 10]; 10],
    canvas_representation: [[Option<TileData>; 10]; 10],
    last_update: Instant,
}

impl ImageCanvasComponent {
    fn remove_dupes(arr: &[TileConnection]) -> Vec<TileConnection> {
        let mut vec = Vec::with_capacity(arr.len());
        for elem in arr {
            if !vec.contains(elem) {
                vec.push(*elem);
            }
        }
        vec.shrink_to_fit();
        vec
    }

    fn fill_representation_array(&mut self, tiles: &[TileData]) {
        let all_north_connections = Self::remove_dupes(
            &tiles
                .iter()
                .flat_map(|tile| tile.north_valid_tiles.clone())
                .collect::<Vec<_>>(),
        );
        let all_south_connections = Self::remove_dupes(
            &tiles
                .iter()
                .flat_map(|tile| tile.south_valid_tiles.clone())
                .collect::<Vec<_>>(),
        );
        let all_east_connections = Self::remove_dupes(
            &tiles
                .iter()
                .flat_map(|tile| tile.east_valid_tiles.clone())
                .collect::<Vec<_>>(),
        );
        let all_west_connections = Self::remove_dupes(
            &tiles
                .iter()
                .flat_map(|tile| tile.west_valid_tiles.clone())
                .collect::<Vec<_>>(),
        );

        for (row_index, row) in self.canvas_connections.iter_mut().enumerate() {
            for (col_index, slot) in row.iter_mut().enumerate() {
                let mut valid_connections: PossibleConnections = PossibleConnections::default();
                if row_index != 0 {
                    valid_connections
                        .north_connections
                        .append(&mut all_north_connections.clone());
                } else {
                    /* valid_connections
                    .north_connections
                    .push((0, Direction::South)); */
                }
                if row_index != 9 {
                    valid_connections
                        .south_connections
                        .append(&mut all_south_connections.clone());
                } else {
                    /* valid_connections
                    .south_connections
                    .push((0, Direction::North)); */
                }
                if col_index != 0 {
                    valid_connections
                        .west_connections
                        .append(&mut all_west_connections.clone());
                } else {
                    /* valid_connections
                    .west_connections
                    .push((0, Direction::East)); */
                }
                if col_index != 9 {
                    valid_connections
                        .east_connections
                        .append(&mut all_east_connections.clone());
                } else {
                    /* valid_connections
                    .east_connections
                    .push((0, Direction::West)); */
                }

                *slot = valid_connections;
            }
        }
    }

    fn append_hash_set(set: &mut HashSet<TileConnection>, slice: &[TileConnection]) {
        for con in slice {
            set.insert(*con);
        }
    }

    fn get_lowest_entropy(&self) -> Option<(usize, usize)> {
        let mut rng = rand::thread_rng();
        let x_f: f32 = rng.gen();
        let y_f: f32 = rng.gen();

        let x = (x_f * 10.0) as usize;
        let y = (y_f * 10.0) as usize;

        let mut lowest_position = (x, y);
        let mut lowest_val = &self.canvas_connections[x][y];
        for (row_index, row) in self.canvas_connections.iter().enumerate() {
            for (col_index, val) in row.iter().enumerate() {
                if val.total_len() < lowest_val.total_len()
                    && self.canvas_representation[row_index][col_index].is_none()
                {
                    lowest_val = val;
                    lowest_position = (row_index, col_index);
                }
            }
        }
        // println!("{lowest_position:?}");
        if lowest_position == (x, y) && self.canvas_representation[x][y].is_some() {
            return None;
        }
        Some(lowest_position)
    }

    fn do_tile_arrs_overlap(vec1: &[TileConnection], vec2: &[TileConnection]) -> bool {
        for elem in vec1 {
            if vec2.contains(elem) {
                return true;
            }
        }
        false
    }

    fn hashmap_max<'a>(map: &'a HashMap<&'a TileData, usize>) -> (&'a TileData, &'a usize) {
        let mut max_index = 0;
        let mut max_val = map.values().collect::<Vec<_>>()[0];

        for (i, val) in map.values().enumerate() {
            if val > max_val {
                max_index = i;
                max_val = val;
            }
        }

        let key = *map.keys().collect::<Vec<_>>()[max_index];
        let val = map.values().collect::<Vec<_>>()[max_index];

        (key, val)
    }

    fn tile_confidence(tile: &TileData, connections: &PossibleConnections) -> f32 {
        let mut confidence = 0.0;
        if Self::do_tile_arrs_overlap(&tile.north_valid_tiles, &connections.north_connections) {
            confidence += 0.25;
        }
        if Self::do_tile_arrs_overlap(&tile.south_valid_tiles, &connections.south_connections) {
            confidence += 0.25;
        }
        if Self::do_tile_arrs_overlap(&tile.east_valid_tiles, &connections.east_connections) {
            confidence += 0.25;
        }
        if Self::do_tile_arrs_overlap(&tile.west_valid_tiles, &connections.west_connections) {
            confidence += 0.25;
        }

        confidence
    }

    fn get_possible_tiles(&self, pos: (usize, usize)) -> Vec<TileData> {
        let mut tiles = Vec::new();
        if pos.0 > 0 {
            let tile = &self.canvas_representation[pos.0 - 1][pos.1];
            if let Some(tile) = tile {
                for (index, _) in &tile.south_valid_tiles {
                    tiles.push(self.current_tile_set[*index].clone());
                }
            }
        }
        if pos.0 < 9 {
            let tile = &self.canvas_representation[pos.0 + 1][pos.1];
            if let Some(tile) = tile {
                for (index, _) in &tile.north_valid_tiles {
                    tiles.push(self.current_tile_set[*index].clone());
                }
            }
        }
        if pos.1 > 0 {
            let tile = &self.canvas_representation[pos.0][pos.1 - 1];
            if let Some(tile) = tile {
                for (index, _) in &tile.east_valid_tiles {
                    tiles.push(self.current_tile_set[*index].clone());
                }
            }
        }
        if pos.1 < 9 {
            let tile = &self.canvas_representation[pos.0][pos.1 + 1];
            if let Some(tile) = tile {
                for (index, _) in &tile.west_valid_tiles {
                    tiles.push(self.current_tile_set[*index].clone());
                }
            }
        }

        tiles
    }

    fn collapse_tile(
        &mut self,
        tile_connections: &PossibleConnections,
        pos: (usize, usize),
    ) -> TileData {
        let possible_tiles = self.get_possible_tiles(pos);

        let most_likely_tile = if possible_tiles.is_empty() {
            tile_connections.random_tile(&self.current_tile_set).clone()
        } else {
            let mut most_confident_tile = &possible_tiles[0];
            let mut highest_confidence =
                Self::tile_confidence(most_confident_tile, tile_connections);

            for tile in &possible_tiles {
                let confidence = Self::tile_confidence(tile, tile_connections);
                if confidence > highest_confidence {
                    highest_confidence = confidence;
                    most_confident_tile = tile;
                }
            }

            most_confident_tile.clone()

            /* let mut rng = rand::thread_rng();
            let index: usize = rng.gen_range(0..possible_tiles.len());
            possible_tiles[index].clone() */
        };

        if pos.0 > 0 {
            let vec = vec![(most_likely_tile.image_index, Direction::North)];
            self.canvas_connections[pos.0 - 1][pos.1].south_connections = vec;
        }
        if pos.0 < 9 {
            let vec = vec![(most_likely_tile.image_index, Direction::South)];
            self.canvas_connections[pos.0 + 1][pos.1].north_connections = vec;
        }
        if pos.1 > 0 {
            let vec = vec![(most_likely_tile.image_index, Direction::West)];
            self.canvas_connections[pos.0][pos.1 - 1].east_connections = vec;
        }
        if pos.1 < 9 {
            let vec = vec![(most_likely_tile.image_index, Direction::East)];
            self.canvas_connections[pos.0][pos.1 + 1].west_connections = vec;
        }

        most_likely_tile
    }

    fn print_entropy(&self) {
        for (r_i, row) in self.canvas_connections.iter().enumerate() {
            for (c_i, tile) in row.iter().enumerate() {
                if self.canvas_representation[r_i][c_i].is_none() {
                    print!("{} ", tile.total_len());
                } else {
                    print!("__ ");
                }
            }
            println!();
        }
        println!();
    }
}

impl Default for ImageCanvasComponent {
    fn default() -> Self {
        let canvas_connections = (0..10)
            .map(|_| {
                let inner: [PossibleConnections; 10] = (0..10)
                    .map(|_| PossibleConnections::default())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                inner
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let canvas_representation = (0..10)
            .map(|_| {
                let inner: [Option<TileData>; 10] = (0..10)
                    .map(|_| None)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                inner
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            current_tile_set: Vec::new(),
            canvas_connections,
            canvas_representation,
            last_update: Instant::now(),
        }
    }
}

impl ComponentSystem for ImageCanvasComponent {
    fn update(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        _engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        _entities: &mut Vec<Entity>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let tiles = concept_manager
            .get_concept::<Vec<TileData>>(
                (0, TypeId::of::<TileCreationComponent>(), 0),
                "loaded_tiles".to_string(),
            )
            .unwrap()
            .clone();
        if tiles != self.current_tile_set {
            self.fill_representation_array(&tiles);
            self.current_tile_set = tiles;
        }
        if !self.current_tile_set.is_empty()
        // && (Instant::now() - self.last_update).as_millis() > 500
        {
            let lowest_entropy_pos = self.get_lowest_entropy();
            if let Some(lowest_entropy_pos) = lowest_entropy_pos {
                let lowest_entropy_tile =
                    self.canvas_connections[lowest_entropy_pos.0][lowest_entropy_pos.1].clone();
                let result = self.collapse_tile(&lowest_entropy_tile, lowest_entropy_pos);
                // dbg!(&lowest_entropy_pos, &result.image_index);
                self.canvas_representation[lowest_entropy_pos.0][lowest_entropy_pos.1] =
                    Some(result);
                // self.last_update = Instant::now();
                self.print_entropy();
            }
        }
    }

    fn ui_draw(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _ui_manager: &mut gamezap::ui_manager::UiManager,
        ui_frame: &mut imgui::Ui,
        _component_map: &mut AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let images = concept_manager
            .get_concept::<Vec<ImageData>>(
                (0, TypeId::of::<TileCreationComponent>(), 0),
                "loaded_images".to_string(),
            )
            .unwrap()
            .clone();
        /* let tiles = concept_manager
        .get_concept::<Vec<TileData>>(
            (0, TypeId::of::<TileCreationComponent>(), 0),
            "loaded_tiles".to_string(),
        )
        .unwrap()
        .clone(); */

        if !images.is_empty() {
            let style = ui_frame.push_style_var(imgui::StyleVar::CellPadding([0.0, 0.0]));
            ui_frame
                .window("Canvas")
                .resizable(false)
                .title_bar(false)
                .scroll_bar(false)
                .scrollable(false)
                .always_auto_resize(true)
                .position([500.0, 20.0], imgui::Condition::Once)
                .build(|| {
                    let image_table = ui_frame.begin_table("Image table", 10).unwrap();
                    for row in &self.canvas_representation {
                        ui_frame.table_next_row();
                        for tile in row {
                            ui_frame.table_next_column();
                            let image_index = if let Some(tile) = tile {
                                // println!("{tile:?}");
                                // tiles.iter().position(|i| i == tile).unwrap()
                                // 1
                                tile.image_index
                            } else {
                                0
                            };
                            imgui::Image::new(images[image_index].id, [50.0, 50.0]).build(ui_frame);
                        }
                    }
                    /* for i in 0..100 {
                        ui_frame.table_next_column();
                        if i % 10 == 0 {
                            ui_frame.table_next_row();
                        }
                        imgui::Image::new(images[i % images.len()].id, [50.0, 50.0])
                            .build(ui_frame);
                    } */
                    image_table.end();
                });
            style.pop();
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
