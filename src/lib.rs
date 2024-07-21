use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Canvas {
    width: u32,
    height: u32,
    layers: Vec<Vec<u8>>,
    active_layer: usize,
}

#[wasm_bindgen]
impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        let size = (width * height * 4) as usize;
        let initial_layer = vec![255; size];
        Canvas {
            width,
            height,
            layers: vec![initial_layer],
            active_layer: 0,
        }
    }

    pub fn add_layer(&mut self) -> usize {
        let size = (self.width * self.height * 4) as usize;
        let new_layer = vec![0; size];
        self.layers.push(new_layer);
        self.layers.len() - 1
    }

    pub fn set_active_layer(&mut self, index: usize) {
        if index < self.layers.len() {
            self.active_layer = index;
        }
    }

    pub fn remove_layer(&mut self, index: usize) {
        if self.layers.len() > 1 && index < self.layers.len() {
            self.layers.remove(index);
            if self.active_layer >= self.layers.len() {
                self.active_layer = self.layers.len() - 1;
            }
        }
    }

    pub fn draw_line(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: u32,
        is_eraser: bool,
        size: u32,
    ) {
        let (r, g, b, a) = if is_eraser {
            (0, 0, 0, 0) // Transparent for eraser
        } else {
            (
                (color >> 16) as u8,
                ((color >> 8) & 255) as u8,
                (color & 255) as u8,
                255,
            )
        };

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            self.draw_circle(x as u32, y as u32, size, r, g, b, a);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_circle(&mut self, cx: u32, cy: u32, radius: u32, r: u8, g: u8, b: u8, a: u8) {
        let radius = radius as i32;
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= radius * radius {
                    self.set_pixel((cx as i32 + x) as u32, (cy as i32 + y) as u32, r, g, b, a);
                }
            }
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            let layer = &mut self.layers[self.active_layer];
            layer[idx] = r;
            layer[idx + 1] = g;
            layer[idx + 2] = b;
            layer[idx + 3] = a;
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_composite_data(&self) -> Vec<u8> {
        let size = (self.width * self.height * 4) as usize;
        let mut composite = vec![0; size];

        for layer in &self.layers {
            for i in (0..layer.len()).step_by(4) {
                let alpha = layer[i + 3] as f32 / 255.0;
                for j in 0..3 {
                    // Iterate over R, G, B channels
                    composite[i + j] = (composite[i + j] as f32 * (1.0 - alpha)
                        + layer[i + j] as f32 * alpha) as u8;
                }
                // Blend alpha channel separately
                composite[i + 3] =
                    (composite[i + 3] as f32 * (1.0 - alpha) + layer[i + 3] as f32 * alpha) as u8;
            }
        }

        composite
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn get_active_layer(&self) -> usize {
        self.active_layer
    }
}
