use crate::{
    animation::Animation,
    frame::Frame,
    sdf::{render_sdf, MultiUnion},
};
use itertools::Itertools;
use nalgebra::{vector, SMatrix, Vector3};
use rand::Rng;
use sdfu::SDF;

#[derive(Default)]
pub struct Waves {
    c2: f32,
    h2: f32,
    u: SMatrix<f32, 8, 8>,
    u_new: SMatrix<f32, 8, 8>,
    v: SMatrix<f32, 8, 8>,
    drops: Vec<(u8, u8, f32)>,
    sdf_cache: Vec<sdfu::mods::Translate<Vector3<f32>, sdfu::Sphere<f32>>>,
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

        u_inner(self, x, dx, y, dy).unwrap_or(0.0)
    }
}

impl Animation for Waves {
    fn next_frame(&mut self, frame: &mut Frame) {
        let mut rng = rand::thread_rng();

        if self.drops.len() < 3 && rng.gen_bool(0.3) {
            self.drops
                .push((rng.gen_range(0..8), rng.gen_range(0..8), 8.0));
        }

        let pred = |drop: &mut (u8, u8, f32)| {
            drop.2 -= 0.1;

            if drop.2 < 0.0 {
                self.u[(drop.0 as usize, drop.1 as usize)] += 0.1;

                false
            } else {
                true
            }
        };
        self.drops.retain_mut(pred);

        for (x, y) in (0..8).cartesian_product(0..8) {
            let f = self.c2
                * (self.u(x, 1, y, 0)
                    + self.u(x, -1, y, 0)
                    + self.u(x, 0, y, 1)
                    + self.u(x, 0, y, -1)
                    - 4.0 * self.u(x, 0, y, 0))
                / self.h2;

            let delta_t = 0.1;

            self.v[(x, y)] += f * delta_t;

            self.u_new[(x, y)] = self.u[(x, y)] + self.v[(x, y)] * delta_t;
        }

        self.u.copy_from(&self.u_new);

        self.sdf_cache.clear();

        for &(x, y, z) in &self.drops {
            let sdf = sdfu::Sphere::new(0.1).translate(vector![x as f32, y as f32, z]);

            self.sdf_cache.push(sdf);
        }

        if self.sdf_cache.is_empty() {
            frame.zero();
        } else {
            let union = MultiUnion::hard(&self.sdf_cache);

            render_sdf(union, frame);
        }

        'outer: for (x, y) in (0..8).cartesian_product(0..8) {
            let mut height = self.u[(x, y)];
            let mut z = 0;

            while height > 1.0 {
                frame.set(x, y, z, 255);
                z += 1;
                height -= 1.0;

                if z >= 8 {
                    continue 'outer;
                }
            }

            frame.set(x, y, z, (height * 255.0) as u8);
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
        self.c2 = 1.0;
        self.h2 = 1.0;
    }
}
