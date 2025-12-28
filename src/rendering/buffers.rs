//! GPU buffer management for efficient rendering

use super::*;
use std::sync::Arc;
use std::marker::PhantomData;

/// Vertex buffer wrapper
pub struct VertexBuffer<T> {
    buffer: wgpu::Buffer,
    capacity: usize,
    count: usize,
    device: Arc<wgpu::Device>,
    label: String,
    _phantom: PhantomData<T>,
}

impl<T: bytemuck::Pod> VertexBuffer<T> {
    /// Create a new vertex buffer
    pub fn new(device: Arc<wgpu::Device>, label: &str, capacity: usize) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            count: 0,
            device,
            label: label.to_string(),
            _phantom: PhantomData,
        }
    }

    /// Create a vertex buffer with initial data
    pub fn new_with_data(device: Arc<wgpu::Device>, label: &str, data: &[T]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            capacity: data.len(),
            count: data.len(),
            device,
            label: label.to_string(),
            _phantom: PhantomData,
        }
    }

    /// Update buffer data
    pub fn update(&mut self, queue: &wgpu::Queue, data: &[T]) {
        if data.len() > self.capacity {
            // Reallocate if needed
            self.capacity = (data.len() as f32 * 1.5) as usize; // 50% growth
            self.buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: (self.capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        self.count = data.len();
    }

    /// Append data to buffer
    pub fn append(&mut self, queue: &wgpu::Queue, data: &[T]) {
        let new_count = self.count + data.len();

        if new_count > self.capacity {
            // Need to reallocate
            let new_capacity = (new_count as f32 * 1.5) as usize;
            let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: (new_capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            // Copy existing data
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Buffer Copy Encoder"),
            });
            encoder.copy_buffer_to_buffer(
                &self.buffer,
                0,
                &new_buffer,
                0,
                (self.count * std::mem::size_of::<T>()) as wgpu::BufferAddress,
            );
            queue.submit(std::iter::once(encoder.finish()));

            self.buffer = new_buffer;
            self.capacity = new_capacity;
        }

        // Write new data
        let offset = (self.count * std::mem::size_of::<T>()) as wgpu::BufferAddress;
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data));
        self.count = new_count;
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Get buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get vertex count
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Index buffer wrapper
pub struct IndexBuffer {
    buffer: wgpu::Buffer,
    capacity: usize,
    count: usize,
    device: Arc<wgpu::Device>,
    format: wgpu::IndexFormat,
    label: String,
}

impl IndexBuffer {
    /// Create a new index buffer (u32)
    pub fn new_u32(device: Arc<wgpu::Device>, label: &str, capacity: usize) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (capacity * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            count: 0,
            device,
            format: wgpu::IndexFormat::Uint32,
            label: label.to_string(),
        }
    }

    /// Create a new index buffer (u16)
    pub fn new_u16(device: Arc<wgpu::Device>, label: &str, capacity: usize) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (capacity * std::mem::size_of::<u16>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            count: 0,
            device,
            format: wgpu::IndexFormat::Uint16,
            label: label.to_string(),
        }
    }

    /// Create index buffer with u32 data
    pub fn new_with_data_u32(device: Arc<wgpu::Device>, label: &str, data: &[u32]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            capacity: data.len(),
            count: data.len(),
            device,
            format: wgpu::IndexFormat::Uint32,
            label: label.to_string(),
        }
    }

    /// Create index buffer with u16 data
    pub fn new_with_data_u16(device: Arc<wgpu::Device>, label: &str, data: &[u16]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            capacity: data.len(),
            count: data.len(),
            device,
            format: wgpu::IndexFormat::Uint16,
            label: label.to_string(),
        }
    }

    /// Update with u32 indices
    pub fn update_u32(&mut self, queue: &wgpu::Queue, data: &[u32]) {
        if data.len() > self.capacity {
            self.capacity = (data.len() as f32 * 1.5) as usize;
            self.buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: (self.capacity * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        self.count = data.len();
        self.format = wgpu::IndexFormat::Uint32;
    }

    /// Update with u16 indices
    pub fn update_u16(&mut self, queue: &wgpu::Queue, data: &[u16]) {
        if data.len() > self.capacity {
            self.capacity = (data.len() as f32 * 1.5) as usize;
            self.buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: (self.capacity * std::mem::size_of::<u16>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        self.count = data.len();
        self.format = wgpu::IndexFormat::Uint16;
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Get buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get index count
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get index format
    pub fn format(&self) -> wgpu::IndexFormat {
        self.format
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Uniform buffer wrapper
pub struct UniformBuffer<T> {
    buffer: wgpu::Buffer,
    device: Arc<wgpu::Device>,
    _phantom: PhantomData<T>,
}

impl<T: bytemuck::Pod> UniformBuffer<T> {
    /// Create a new uniform buffer
    pub fn new(device: Arc<wgpu::Device>, label: &str, initial_data: T) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::bytes_of(&initial_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            device,
            _phantom: PhantomData,
        }
    }

    /// Update uniform data
    pub fn update(&self, queue: &wgpu::Queue, data: T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&data));
    }

    /// Get buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

/// Dynamic buffer for frequently updated data
pub struct DynamicBuffer<T> {
    buffers: Vec<wgpu::Buffer>,
    current_index: usize,
    device: Arc<wgpu::Device>,
    capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T: bytemuck::Pod> DynamicBuffer<T> {
    /// Create a new dynamic buffer with triple buffering
    pub fn new(device: Arc<wgpu::Device>, label: &str, capacity: usize) -> Self {
        let buffers = (0..3)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{} {}", label, i)),
                    size: (capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        Self {
            buffers,
            current_index: 0,
            device,
            capacity,
            _phantom: PhantomData,
        }
    }

    /// Get current buffer
    pub fn current_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[self.current_index]
    }

    /// Update current buffer and advance to next
    pub fn update(&mut self, queue: &wgpu::Queue, data: &[T]) -> RenderResult<()> {
        if data.len() > self.capacity {
            return Err(RenderError::BufferCreation(format!(
                "Data size {} exceeds capacity {}",
                data.len(),
                self.capacity
            )));
        }

        queue.write_buffer(
            &self.buffers[self.current_index],
            0,
            bytemuck::cast_slice(data),
        );

        self.current_index = (self.current_index + 1) % self.buffers.len();

        Ok(())
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Buffer pool for managing multiple buffers efficiently
pub struct BufferPool<T> {
    free_buffers: Vec<wgpu::Buffer>,
    used_buffers: Vec<wgpu::Buffer>,
    device: Arc<wgpu::Device>,
    buffer_size: usize,
    label: String,
    _phantom: PhantomData<T>,
}

impl<T: bytemuck::Pod> BufferPool<T> {
    /// Create a new buffer pool
    pub fn new(
        device: Arc<wgpu::Device>,
        label: &str,
        buffer_size: usize,
        initial_count: usize,
    ) -> Self {
        let free_buffers = (0..initial_count)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{} Pool {}", label, i)),
                    size: (buffer_size * std::mem::size_of::<T>()) as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        Self {
            free_buffers,
            used_buffers: Vec::new(),
            device,
            buffer_size,
            label: label.to_string(),
            _phantom: PhantomData,
        }
    }

    /// Acquire a buffer from the pool
    pub fn acquire(&mut self) -> wgpu::Buffer {
        self.free_buffers.pop().unwrap_or_else(|| {
            // Create new buffer if pool is empty
            let index = self.used_buffers.len() + self.free_buffers.len();
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{} Pool {}", self.label, index)),
                size: (self.buffer_size * std::mem::size_of::<T>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
    }

    /// Release a buffer back to the pool
    pub fn release(&mut self, buffer: wgpu::Buffer) {
        self.free_buffers.push(buffer);
    }

    /// Release all used buffers
    pub fn release_all(&mut self) {
        self.free_buffers.append(&mut self.used_buffers);
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.free_buffers.len(), self.used_buffers.len())
    }
}

/// Staging buffer for efficient CPU-to-GPU transfers
pub struct StagingBuffer {
    buffer: wgpu::Buffer,
    size: u64,
}

impl StagingBuffer {
    /// Create a new staging buffer
    pub fn new(device: &wgpu::Device, size: u64) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self { buffer, size }
    }

    /// Copy data to GPU buffer
    pub fn copy_to_buffer<T: bytemuck::Pod>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        dst_buffer: &wgpu::Buffer,
        data: &[T],
    ) -> RenderResult<()> {
        let data_size = (data.len() * std::mem::size_of::<T>()) as u64;

        if data_size > self.size {
            return Err(RenderError::BufferCreation(format!(
                "Data size {} exceeds staging buffer size {}",
                data_size, self.size
            )));
        }

        // For simplicity, just write directly to the destination buffer
        // In production, you might want async mapping for very large transfers
        queue.write_buffer(dst_buffer, 0, bytemuck::cast_slice(data));

        Ok(())
    }
}

// Import wgpu::util for buffer_init
use wgpu::util::DeviceExt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_buffer_creation() {
        // This test would require a wgpu device, so it's mostly a compile check
        // In a real test suite, you'd set up a headless device for testing
    }
}
