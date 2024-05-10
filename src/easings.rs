use super::*;

/// https://easings.net/#easeOutElastic
pub fn ease_out_elastic(x: f32) -> f32 {
    if x == 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    let c4 = 2.0 * f32::PI / 3.0;
    2.0.powf(-10.0 * x) * ((x * 10.0 - 0.75) * c4).sin() + 1.0
}
