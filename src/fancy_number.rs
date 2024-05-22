use super::*;

struct DigitPlace {
    current_value: f32,
}

impl DigitPlace {
    fn update(&mut self, target_value: i32, delta_time: f32) {
        self.current_value += (target_value as f32 - self.current_value) * delta_time.min(1.0);
    }
}

pub struct FancyNumber {
    ctx: Ctx,
    target_value: i32,
    /// from least to most significant
    digits: Vec<DigitPlace>,
}

impl FancyNumber {
    pub fn new(ctx: &Ctx, value: i32) -> Self {
        Self {
            ctx: ctx.clone(),
            target_value: value,
            digits: vec![DigitPlace {
                current_value: -0.5,
            }],
        }
    }

    pub fn set_value(&mut self, new_value: i32) {
        self.target_value = new_value;
    }

    pub fn update(&mut self, delta_time: f32) {
        let delta_time = delta_time * self.ctx.config.score.digit_update_speed;
        let mut target_value = self.target_value;
        let mut i = 0;
        while target_value != 0 || i == 0 {
            if i >= self.digits.len() {
                self.digits.push(DigitPlace {
                    current_value: target_value as f32 - 0.5,
                });
            }
            self.digits[i].update(target_value, delta_time);
            target_value /= 10;
            i += 1;
        }
        self.digits.truncate(i);
    }

    fn width(&self) -> f32 {
        self.digits.len() as f32 * self.ctx.config.digit_size
    }

    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn AbstractCamera3d,
        align: f32,
        transform: mat4<f32>,
    ) {
        let transform = transform * mat4::translate(vec3(-align * self.width(), 0.0, 0.0));
        for (i, digit) in self.digits.iter().rev().enumerate() {
            let x = (i as f32 + 0.5) * self.ctx.config.digit_size;
            self.ctx.render.digit(
                framebuffer,
                camera,
                digit.current_value,
                Rgba::WHITE,
                transform * mat4::translate(vec3(x, 0.0, 0.0)),
            );
        }
    }
}
