use sgpu::*;

/// Every layer is a TILE_SIZE x TILE_SIZE RGBA8 tile. Layer index = texture id used by the shader.
const TILE_SIZE: u32 = 16;
const LAYERS: u32 = 256;

pub struct TextureArray {
    texture: Image,
    sampler: Sampler,
}

impl TextureArray {
    pub fn new() -> Self {
        let pixels = generate_tiles();

        let texture = create_image(&ImageDescription {
            usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
            format: Format::Rgba8Unorm,
            extent: Extent3D { width: TILE_SIZE, height: TILE_SIZE, depth: 1 },
            memory_type: MemoryType::DeviceLocal,
            mip_levels: 1,
            array_layers: LAYERS,
            default_view: ImageViewDescription {
                view_type: ImageViewType::Type2DArray,
                subresources: ImageSubresources { layer_count: LAYERS, ..Default::default() },
            },
            ..Default::default()
        });

        // Staging buffer with all tiles, laid out layer after layer.
        let staging = create_buffer(&BufferDescription {
            usage: BufferUsage::TRANSFER_SRC,
            size: pixels.len() as u64,
            memory_type: MemoryType::PreferHost,
        });
        buffer_as_slice_mut::<u8>(&staging)[..pixels.len()].copy_from_slice(&pixels);

        let mut cmd = record(QueueType::Graphics);

        cmd.barriers(
            None,
            &[ImageBarrier {
                view: texture.default_view(),
                previous_accesses: &[AccessType::None],
                next_accesses: &[AccessType::TransferWrite],
                previous_layout: ImageLayout::Optimal,
                next_layout: ImageLayout::Optimal,
                discard_contents: true,
            }],
        );

        cmd.copy_buffer_to_image(
            &staging,
            &texture,
            &[BufferImageCopyRegion {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: ImageSubresources { layer_count: LAYERS, ..Default::default() },
                image_offset: Offset3D { x: 0, y: 0, z: 0 },
                image_extent: Extent3D { width: TILE_SIZE, height: TILE_SIZE, depth: 1 },
            }],
        );

        cmd.barriers(
            None,
            &[ImageBarrier {
                view: texture.default_view(),
                previous_accesses: &[AccessType::TransferWrite],
                next_accesses: &[AccessType::FragmentShaderSampledRead],
                previous_layout: ImageLayout::Optimal,
                next_layout: ImageLayout::Optimal,
                discard_contents: false,
            }],
        );

        wait(submit(&[cmd]));
        destroy_buffer(staging);

        let sampler = create_sampler(&SamplerDescription {
            filter: FilterMode::Nearest,
            mipmap_mode: FilterMode::Nearest,
            address_mode_u: SamplerAddressMode::Repeat,
            address_mode_v: SamplerAddressMode::Repeat,
            address_mode_w: SamplerAddressMode::Repeat,
            lod_min: 0.0,
            lod_max: 0.0,
        });

        Self { texture, sampler }
    }

    pub fn texture(&self) -> Image {
        self.texture
    }

    pub fn sampler(&self) -> Sampler {
        self.sampler
    }
}

impl Drop for TextureArray {
    fn drop(&mut self) {
        destroy_sampler(self.sampler);
        destroy_image(self.texture);
    }
}

// some stuff randomly generated colours
fn generate_tiles() -> Vec<u8> {
    let mut pixels = Vec::with_capacity((LAYERS * TILE_SIZE * TILE_SIZE * 4) as usize);

    for layer in 0..LAYERS {
        // cheap integer hash -> base colour in 90..=230 per channel
        let h = (layer + 1).wrapping_mul(0x9E37_79B1);
        let base = [90 + (h >> 24) % 141, 90 + (h >> 16 & 0xFF) % 141, 90 + (h >> 8 & 0xFF) % 141];

        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let border = x == 0 || y == 0 || x == TILE_SIZE - 1 || y == TILE_SIZE - 1;
                let checker = (x / 4 + y / 4) % 2 == 0;

                let shade = if border {
                    0.55
                } else if checker {
                    1.0
                } else {
                    0.88
                };

                pixels.extend(base.iter().map(|c| (*c as f32 * shade) as u8));
                pixels.push(255);
            }
        }
    }

    pixels
}
