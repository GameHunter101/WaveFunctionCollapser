use gamezap::{ecs::scene::Scene, GameZap};

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

    engine.create_scene(scene);

    engine.main_loop();
}
