use crate::{
    animation::Animation,
    frame::Frame,
    sdf::{render_sdf, MultiUnion},
};
use itertools::Itertools;
use nalgebra::{vector, SMatrix, Vector3};
use palette::{LinSrgba, Mix};
use rand::Rng;
use sdfu::SDF;

#[cfg_attr(feature = "visual", derive(bevy_inspector_egui::Inspectable))]
pub struct Waves {
    c2: f32,
    h2: f32,
    drop_speed: f32,
    delta_t: f32,
    drop_volume: f32,
    drain_rate: f32,
    #[cfg_attr(feature = "visual", inspectable(min = 0.0, max = 100.0))]
    max_gradient: f32,
    #[cfg_attr(feature = "visual", inspectable(read_only))]
    current_volume: f32,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    u: SMatrix<f32, 8, 8>,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    u_new: SMatrix<f32, 8, 8>,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    v: SMatrix<f32, 8, 8>,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    drops: Vec<(u8, u8, f32)>,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    sdf_cache: Vec<sdfu::mods::Translate<Vector3<f32>, sdfu::Sphere<f32>>>,
}

impl Default for Waves {
    fn default() -> Self {
        Self {
            c2: 0.3,
            h2: 3.0,
            drop_speed: 0.1,
            delta_t: 0.2,
            drop_volume: 10.0,
            drain_rate: 0.1,
            max_gradient: 20.0,
            current_volume: 0.0,
            u: Default::default(),
            u_new: Default::default(),
            v: Default::default(),
            drops: Default::default(),
            sdf_cache: Default::default(),
        }
    }
}

impl std::fmt::Debug for Waves {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Waves").finish()
    }
}

impl Waves {
    fn u(&self, x: usize, dx: isize, y: usize, dy: isize) -> f32 {
        fn u_inner(this: &Waves, x: usize, dx: isize, y: usize, dy: isize) -> Option<f32> {
            let x = x.checked_add_signed(dx)?;
            let y = y.checked_add_signed(dy)?;

            this.u.get((x, y)).copied()
        }

        u_inner(self, x, dx, y, dy).unwrap_or_else(|| self.u[(x, y)])
    }
}

impl Animation for Waves {
    fn next_frame(&mut self, frame: &mut Frame) {
        let mut rng = rand::thread_rng();

        for (x, z) in (0..8).cartesian_product(0..8) {
            let f = self.c2
                * (self.u(x, 1, z, 0)
                    + self.u(x, -1, z, 0)
                    + self.u(x, 0, z, 1)
                    + self.u(x, 0, z, -1)
                    - 4.0 * self.u(x, 0, z, 0))
                / self.h2;

            let f = f.clamp(-self.max_gradient, self.max_gradient);

            self.v[(x, z)] += f * self.delta_t;

            self.u_new[(x, z)] = (self.u[(x, z)] + self.v[(x, z)] * self.delta_t).max(0.0);
            self.v[(x, z)] *= 0.97;
        }

        self.u.copy_from(&self.u_new);
        self.u.add_scalar_mut(-self.drain_rate * self.delta_t);
        self.current_volume = self.u.sum();

        if self.drops.len() < 3 && rng.gen_bool(0.3) {
            self.drops
                .push((rng.gen_range(0..8), rng.gen_range(0..8), 8.0));
        }

        let pred = |drop: &mut (u8, u8, f32)| {
            drop.2 -= self.drop_speed;

            if drop.2 < 0.0 {
                self.u[(drop.0 as usize, drop.1 as usize)] += self.drop_volume;

                false
            } else {
                true
            }
        };
        self.drops.retain_mut(pred);

        self.sdf_cache.clear();

        for &(x, z, y) in &self.drops {
            let sdf = sdfu::Sphere::new(0.1, LinSrgba::new(0.0, 0.2, 0.9, 1.0))
                .translate(vector![x as f32, y, z as f32]);

            self.sdf_cache.push(sdf);
        }

        if self.sdf_cache.is_empty() {
            frame.zero();
        } else {
            let union = MultiUnion::hard(&self.sdf_cache);

            render_sdf(union, frame);
        }

        'outer: for (x, z) in (0..8).cartesian_product(0..8) {
            let mut height = self.u[(x, z)];
            let mut y = 0;

            let led_volume = 6.0;

            while height > led_volume {
                frame.set(x, y, z, LinSrgba::new(0.0, 0.2, 0.9, 0.8));
                y += 1;
                height -= led_volume;

                if y >= 8 {
                    continue 'outer;
                }
            }

            let colour = LinSrgba::new(0.0, 0.2, 0.9, 0.8).mix(
                &LinSrgba::new(1.0, 1.0, 1.0, 0.8),
                height.max(0.0) / led_volume,
            );

            frame.set(x, y, z, colour);
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}
