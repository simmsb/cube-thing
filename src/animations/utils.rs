use nalgebra::Rotation3;
use palette::{FromColor, Hsva, LinSrgba, RgbHue, Srgba};
use rand::Rng;

pub fn random_rotation() -> Rotation3<f32> {
    let mut rng = rand::thread_rng();
    let theta = (2.0 * rng.gen_range(0.0..1.0f32) - 1.0).acos();
    let phi = std::f32::consts::TAU * rng.gen_range(0.0..1.0);

    Rotation3::from_euler_angles(0.0, phi, theta)
}

pub fn random_colour(saturation: f32, value: f32, alpha: f32) -> LinSrgba {
    let mut rng = rand::thread_rng();
    let rads = rng.gen_range(0.0..std::f32::consts::TAU) - std::f32::consts::PI;

    let hue = RgbHue::from_radians(rads);
    let hsv = Hsva::new(hue, saturation, value, alpha);
    Srgba::from_color(hsv).into_linear()
}
