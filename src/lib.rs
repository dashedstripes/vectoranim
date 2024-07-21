use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Canvas {
    width: u32,
    height: u32,
    frames: Vec<Frame>,
    current_frame: usize,
    drawing_buffer: Vec<u8>,
    dirty_rect: Option<(u32, u32, u32, u32)>, // (x, y, width, height)
}

struct Layer {
    data: Vec<u8>,
    visible: bool,
}

struct Frame {
    layers: Vec<Layer>,
    active_layer: usize,
}

#[wasm_bindgen]
impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        let size = (width * height * 4) as usize;
        let initial_layer = Layer {
            data: vec![0; size],
            visible: true,
        };
        let initial_frame = Frame {
            layers: vec![initial_layer],
            active_layer: 0,
        };
        Canvas {
            width,
            height,
            frames: vec![initial_frame],
            current_frame: 0,
            drawing_buffer: vec![0; size],
            dirty_rect: None,
        }
    }

    pub fn add_layer(&mut self) -> usize {
        let size = (self.width * self.height * 4) as usize;
        let new_layer = Layer {
            data: vec![0; size],
            visible: true,
        };
        let frame = &mut self.frames[self.current_frame];
        frame.layers.push(new_layer);
        frame.layers.len() - 1
    }

    pub fn set_active_layer(&mut self, index: usize) {
        let frame = &mut self.frames[self.current_frame];
        if index < frame.layers.len() {
            frame.active_layer = index;
        }
    }

    pub fn remove_layer(&mut self, index: usize) {
        let frame = &mut self.frames[self.current_frame];
        if frame.layers.len() > 1 && index < frame.layers.len() {
            frame.layers.remove(index);
            if frame.active_layer >= frame.layers.len() {
                frame.active_layer = frame.layers.len() - 1;
            }
        }
    }

    pub fn toggle_layer_visibility(&mut self, index: usize) {
        let frame = &mut self.frames[self.current_frame];
        if index < frame.layers.len() {
            frame.layers[index].visible = !frame.layers[index].visible;
        }
    }

    pub fn is_layer_visible(&self, index: usize) -> bool {
        let frame = &self.frames[self.current_frame];
        if index < frame.layers.len() {
            frame.layers[index].visible
        } else {
            false
        }
    }

    pub fn add_frame(&mut self) -> usize {
        let size = (self.width * self.height * 4) as usize;
        let new_frame = Frame {
            layers: vec![Layer {
                data: vec![0; size],
                visible: true,
            }],
            active_layer: 0,
        };
        self.frames.push(new_frame);
        self.current_frame = self.frames.len() - 1;
        self.current_frame
    }

    pub fn set_current_frame(&mut self, index: usize) {
        if index < self.frames.len() {
            self.current_frame = index;
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

        let radius = size as i32 / 2;
        let radius_squared = radius * radius;

        loop {
            // Use a more efficient circle drawing algorithm
            for cy in (y - radius).max(0)..(y + radius + 1).min(self.height as i32) {
                for cx in (x - radius).max(0)..(x + radius + 1).min(self.width as i32) {
                    let dx = cx - x;
                    let dy = cy - y;
                    if dx * dx + dy * dy <= radius_squared {
                        self.set_pixel(cx as u32, cy as u32, r, g, b, a);
                    }
                }
            }

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

    pub fn commit_drawing(&mut self) {
        if let Some((x, y, width, height)) = self.dirty_rect {
            let frame = &mut self.frames[self.current_frame];
            let layer = &mut frame.layers[frame.active_layer];
            for dy in 0..height {
                for dx in 0..width {
                    let buffer_idx = ((y + dy) * self.width + (x + dx)) as usize * 4;
                    for i in 0..4 {
                        layer.data[buffer_idx + i] = self.drawing_buffer[buffer_idx + i];
                    }
                }
            }
            self.dirty_rect = None;
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            self.drawing_buffer[idx] = r;
            self.drawing_buffer[idx + 1] = g;
            self.drawing_buffer[idx + 2] = b;
            self.drawing_buffer[idx + 3] = a;

            // Update dirty rect
            match self.dirty_rect {
                Some((dx, dy, dw, dh)) => {
                    let new_x = dx.min(x);
                    let new_y = dy.min(y);
                    let new_w = dw.max(x - new_x + 1);
                    let new_h = dh.max(y - new_y + 1);
                    self.dirty_rect = Some((new_x, new_y, new_w, new_h));
                }
                None => self.dirty_rect = Some((x, y, 1, 1)),
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_composite_data(&self, onion_skin_frames: Vec<usize>) -> Vec<u8> {
        let size = (self.width * self.height * 4) as usize;
        let mut composite = vec![0; size];

        // Add onion skin layers
        for &frame_index in &onion_skin_frames {
            if frame_index != self.current_frame {
                let distance = (frame_index as i32 - self.current_frame as i32).abs() as f32;
                let opacity = (0.3 / distance).min(0.3); // Fade out farther frames, max opacity 0.3
                self.composite_frame(&mut composite, frame_index, opacity);
            }
        }

        // Composite the current frame
        self.composite_frame(&mut composite, self.current_frame, 1.0);

        composite
    }

    fn composite_frame(&self, composite: &mut Vec<u8>, frame_index: usize, opacity: f32) {
        if let Some(frame) = self.frames.get(frame_index) {
            for layer in &frame.layers {
                if layer.visible {
                    for (i, chunk) in layer.data.chunks_exact(4).enumerate() {
                        let base_index = i * 4;
                        let layer_alpha = chunk[3] as f32 / 255.0;
                        let alpha = layer_alpha * opacity;

                        for j in 0..3 {
                            // RGB channels
                            let new_color = (1.0 - alpha) * composite[base_index + j] as f32
                                + alpha * chunk[j] as f32;
                            composite[base_index + j] = new_color.min(255.0) as u8;
                        }

                        // Update alpha channel
                        let new_alpha = composite[base_index + 3] as f32
                            + (255.0 - composite[base_index + 3] as f32) * alpha;
                        composite[base_index + 3] = new_alpha.min(255.0) as u8;
                    }
                }
            }
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn get_current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn layer_count(&self) -> usize {
        self.frames[self.current_frame].layers.len()
    }

    pub fn get_active_layer(&self) -> usize {
        self.frames[self.current_frame].active_layer
    }
}
