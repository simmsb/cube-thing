use crate::{
    animation::Animation,
    frame::Frame,
    sdf::{render_sdf, MultiUnion},
};
use itertools::Itertools;
use nalgebra::{vector, SMatrix, Vector3};
use rand::Rng;
use sdfu::SDF;

pub struct Waves {
    c2: f32,
    h2: f32,
    drop_speed: f32,
    delta_t: f32,
    drop_volume: f32,
    max_gradient: f32,
    u: SMatrix<f32, 8, 8>,
    u_new: SMatrix<f32, 8, 8>,
    v: SMatrix<f32, 8, 8>,
    drops: Vec<(u8, u8, f32)>,
    sdf_cache: Vec<sdfu::mods::Translate<Vector3<f32>, sdfu::Sphere<f32>>>,
}

impl Default for Waves {
    fn default() -> Self {
        Self {
            c2: 0.3,
            h2: 3.0,
            drop_speed: 0.1,
            delta_t: 0.5,
            drop_volume: 0.3,
            max_gradient: 0.8,
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

            self.u_new[(x, z)] = self.u[(x, z)] + self.v[(x, z)] * self.delta_t;
            self.v[(x, z)] *= 0.97;
        }

        self.u.copy_from(&self.u_new);

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
            let sdf = sdfu::Sphere::new(0.1).translate(vector![x as f32, y, z as f32]);

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

            while height > 1.0 {
                frame.set(x, y, z, 255);
                y += 1;
                height -= 1.0;

                if y >= 8 {
                    continue 'outer;
                }
            }

            frame.set(x, y, z, (height * 255.0) as u8);
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}
