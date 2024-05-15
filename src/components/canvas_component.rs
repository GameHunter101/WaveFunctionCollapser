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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct VertexData {
    position: [u16; 2],
    color: [u8; 3],
    pixel_size: u8,
}

#[derive(Debug, Clone)]
pub struct CanvasComponent {
    parent: EntityId,
    id: ComponentId,
}

impl Default for CanvasComponent {
    fn default() -> Self {
        CanvasComponent {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
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
        _device: Arc<Device>,
        _queue: Arc<Queue>,
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

            /* ui.window(".").build(|| {
            }) */

            let files = FileDialog::new().pick_file();
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
