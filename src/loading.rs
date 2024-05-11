use super::*;

pub async fn run(geng: &Geng) {
    let mut camera = Camera2d {
        center: vec2::ZERO,
        rotation: Angle::ZERO,
        fov: 20.0,
    };
    let timer = Timer::new();
    while let Some(event) = geng.window().events().next().await {
        if let geng::Event::Draw = event {
            geng::async_state::with_current_framebuffer(geng.window(), |framebuffer| {
                ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
                const SPINNERS: usize = 5;
                camera.rotation = Angle::from_degrees(180.0 * timer.elapsed().as_secs_f64() as f32);
                for i in 0..SPINNERS {
                    geng.draw2d().quad(
                        framebuffer,
                        &camera,
                        Aabb2::point(
                            vec2(1.0, 0.0)
                                .rotate(Angle::from_degrees(360.0 * i as f32 / SPINNERS as f32)),
                        )
                        .extend_uniform(0.1),
                        Rgba::WHITE,
                    );
                }
            });
        }
    }
}
