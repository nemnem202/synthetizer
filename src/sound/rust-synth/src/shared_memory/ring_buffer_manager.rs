use js_sys::{Atomics, Float32Array, Int32Array};

pub struct RingBufferManager<'a> {
    ring_buffer: &'a Float32Array,
    write_idx_atomic: &'a Int32Array,
    buffer_size: i32,
}

impl<'a> RingBufferManager<'a> {
    pub fn new(ring_buffer: &'a Float32Array, write_idx_atomic: &'a Int32Array) -> Self {
        Self {
            ring_buffer,
            write_idx_atomic,
            buffer_size: ring_buffer.length() as i32,
        }
    }

    pub fn write_samples(&self, samples: &[f32]) {
        let space = samples.len() as i32;
        let mut current_write_idx = Atomics::load(self.write_idx_atomic, 0).unwrap();
        let chunk_array = Float32Array::from(samples);

        let contiguous_space = std::cmp::min(space, self.buffer_size - current_write_idx);

        if contiguous_space > 0 {
            self.ring_buffer.set(
                &chunk_array.subarray(0, contiguous_space as u32),
                current_write_idx as u32,
            );
        }

        if space > contiguous_space {
            let rest = space - contiguous_space;
            self.ring_buffer.set(
                &chunk_array.subarray(contiguous_space as u32, (contiguous_space + rest) as u32),
                0,
            );
            current_write_idx = rest;
        } else {
            current_write_idx = (current_write_idx + contiguous_space) % self.buffer_size;
        }

        Atomics::store(self.write_idx_atomic, 0, current_write_idx).unwrap();
    }
}
