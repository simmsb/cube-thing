use nalgebra::Rotation3;
use rand::Rng;

pub fn random_rotation() -> Rotation3<f32> {
    let mut rng = rand::thread_rng();
    let theta = (2.0 * rng.gen_range(0.0..1.0f32) - 1.0).acos();
    let phi = std::f32::consts::TAU * rng.gen_range(0.0..1.0);

    Rotation3::from_euler_angles(0.0, phi, theta)
}
