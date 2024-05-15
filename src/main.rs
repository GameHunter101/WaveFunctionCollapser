use components::canvas_component::CanvasComponent;
use gamezap::{ecs::scene::Scene, GameZap};

pub mod components {
    pub mod canvas_component;
}

#[tokio::main]
async fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let application_title = "Wave Function Collapser";
    let window_size = (800, 600);
    let window = video_subsystem
        .window(application_title, window_size.0, window_size.1)
        .resizable()
        .build()
        .unwrap();

    let mut engine = GameZap::builder()
        .window_and_renderer(
            sdl_context,
            video_subsystem,
            event_pump,
            window,
            wgpu::Color {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            },
        )
        .antialiasing()
        .build()
        .await;

    let mut scene = Scene::default();

    let canvas_component = CanvasComponent::default();

    let _canvas_entity = scene.create_entity(0, true, vec![Box::new(canvas_component)], None);

    engine.create_scene(scene);

    engine.main_loop();
}
