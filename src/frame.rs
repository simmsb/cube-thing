use palette::LinSrgba;

#[derive(Clone, Debug)]
pub struct Frame([[[LinSrgba; 8]; 8]; 8]);

impl Frame {
    pub const LAYERS: usize = 8;
    pub const ROWS: usize = 8;
    pub const COLUMNS: usize = 8;

    pub fn new() -> Self {
        Self([[[LinSrgba::new(0.0, 0.0, 0.0, 0.0); 8]; 8]; 8])
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> LinSrgba {
        self.0[y][x][z]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, val: LinSrgba) {
        self.0[y][x][z] = val;
    }

    pub fn zero(&mut self) {
        *self = Frame::new();
    }

    pub fn layer_mut(&mut self, n: usize) -> &mut [[LinSrgba; 8]; 8] {
        &mut self.0[n]
    }

    pub fn layers(&self) -> &[[[LinSrgba; 8]; 8]; 8] {
        &self.0
    }

    pub fn pixels<'a>(&'a self) -> impl Iterator<Item = (u8, u8, u8, LinSrgba)> + 'a {
        self.0.iter().zip(0..8u8).flat_map(|(layer, y)| {
            layer
                .iter()
                .zip(0..8u8)
                .flat_map(move |(row, x)| row.iter().zip(0..8u8).map(move |(pix, z)| (x, z, pix)))
                .map(move |(x, z, pix)| (x, y, z, *pix))
        })
    }

    pub fn pixels_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = (u8, u8, u8, &'a mut LinSrgba)> + 'a {
        self.0.iter_mut().zip(0..8u8).flat_map(|(layer, y)| {
            layer
                .iter_mut()
                .zip(0..8u8)
                .flat_map(move |(row, x)| {
                    row.iter_mut().zip(0..8u8).map(move |(pix, z)| (x, z, pix))
                })
                .map(move |(x, z, pix)| (x, y, z, pix))
        })
    }
}
