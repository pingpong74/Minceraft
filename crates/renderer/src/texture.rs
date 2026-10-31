use std::path::Path;

use sgpu::*;

const TEXTURE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../textures");
const TILE_SIZE: u32 = 16;

/// Textures are stored per Layer
pub struct TextureArray {
    texture: Image,
    sampler: Sampler,
}

impl TextureArray {
    pub fn new(names: &[&str]) -> Self {
        let tiles = load_tiles(names, Path::new(TEXTURE_DIR));
        let layers = tiles.tiles.len() as u32;
        let (data, levels) = pack_mips(&tiles);
        let mip_levels = levels.len() as u32;

        let texture = create_image(&ImageDescription {
            usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
            format: Format::Rgba8Unorm,
            extent: Extent3D {
                width: tiles.size,
                height: tiles.size,
                depth: 1,
            },
            memory_type: MemoryType::DeviceLocal,
            mip_levels,
            array_layers: layers,
            default_view: ImageViewDescription {
                view_type: ImageViewType::Type2DArray,
                subresources: ImageSubresources {
                    level_count: mip_levels,
                    layer_count: layers,
                    ..Default::default()
                },
            },
            ..Default::default()
        });

        let staging = create_buffer(&BufferDescription {
            usage: BufferUsage::TRANSFER_SRC,
            size: data.len() as u64,
            memory_type: MemoryType::PreferHost,
        });
        buffer_as_slice_mut::<u8>(&staging)[..data.len()].copy_from_slice(&data);

        // One copy per mip level; each covers all layers.
        let regions: Vec<BufferImageCopyRegion> = levels
            .iter()
            .map(|level| BufferImageCopyRegion {
                buffer_offset: level.offset,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: ImageSubresources {
                    mip_level: level.mip,
                    layer_count: layers,
                    ..Default::default()
                },
                image_offset: Offset3D { x: 0, y: 0, z: 0 },
                image_extent: Extent3D {
                    width: level.size,
                    height: level.size,
                    depth: 1,
                },
            })
            .collect();

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
        cmd.copy_buffer_to_image(&staging, &texture, &regions);
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
            mipmap_mode: FilterMode::Linear,
            address_mode_u: SamplerAddressMode::Repeat,
            address_mode_v: SamplerAddressMode::Repeat,
            address_mode_w: SamplerAddressMode::Repeat,
            lod_min: 0.0,
            lod_max: mip_levels as f32,
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

// Each indec inside tiles is an array layer
struct TileSet {
    size: u32,
    tiles: Vec<Vec<u8>>,
}

fn read_png(path: &Path) -> Result<image::RgbaImage, String> {
    image::open(path).map(|image| image.to_rgba8()).map_err(|e| e.to_string())
}

fn load_tiles(names: &[&str], dir: &Path) -> TileSet {
    let images: Vec<Option<image::RgbaImage>> = names
        .iter()
        .map(|name| {
            if *name == "missing" {
                return None;
            }
            let path = dir.join(format!("{name}.png"));
            read_png(&path).map_err(|e| eprintln!("texture '{name}' ({}): {e}; using a placeholder", path.display())).ok()
        })
        .collect();

    let size = images.iter().flatten().find(|image| image.width() == image.height()).map_or(TILE_SIZE, |image| image.width());

    let tiles = names
        .iter()
        .zip(images)
        .enumerate()
        .map(|(layer, (name, image))| match image {
            Some(image) if image.width() == size && image.height() == size => image.into_raw(),
            Some(image) => {
                eprintln!("texture '{name}' is {}x{}, expected {size}x{size}; using a placeholder", image.width(), image.height());
                placeholder_tile(layer, size)
            }
            None if *name == "missing" => missing_tile(size),
            None => placeholder_tile(layer, size),
        })
        .collect();

    TileSet { size, tiles }
}

/// Classic magenta/black checker.
fn missing_tile(size: u32) -> Vec<u8> {
    let cell = (size / 2).max(1);
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let magenta = (x / cell + y / cell) % 2 == 0;
            pixels.extend_from_slice(if magenta { &[255, 0, 255, 255] } else { &[0, 0, 0, 255] });
        }
    }

    pixels
}

/// Stand-in for a texture whose file is missing: a stable colour per layer with a dark border, so
/// different blocks stay distinguishable.
fn placeholder_tile(layer: usize, size: u32) -> Vec<u8> {
    let h = (layer as u32 + 1).wrapping_mul(0x9E37_79B1);
    let base = [90 + (h >> 24) % 141, 90 + (h >> 16 & 0xFF) % 141, 90 + (h >> 8 & 0xFF) % 141];
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let border = x == 0 || y == 0 || x == size - 1 || y == size - 1;
            let shade = if border { 0.55 } else { 1.0 };
            pixels.extend(base.iter().map(|c| (*c as f32 * shade) as u8));
            pixels.push(255);
        }
    }

    pixels
}

fn srgb_to_linear(value: u8) -> f32 {
    let c = value as f32 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn linear_to_srgb(value: f32) -> u8 {
    let c = value.clamp(0.0, 1.0);
    let encoded = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
    (encoded * 255.0 + 0.5) as u8
}

/// 2x2 box filter in linear light (colour is averaged as light, not as the stored sRGB numbers).
/// Alpha is stored linearly and averaged directly.
fn downsample(src: &[u8], size: u32) -> (Vec<u8>, u32) {
    let out = (size / 2).max(1);
    let lut: [f32; 256] = std::array::from_fn(|i| srgb_to_linear(i as u8));
    let mut dst = Vec::with_capacity((out * out * 4) as usize);

    for y in 0..out {
        for x in 0..out {
            // for odd sizes the last row/column is read twice, which is fine for a placeholder-grade filter
            let xs = [(2 * x).min(size - 1), (2 * x + 1).min(size - 1)];
            let ys = [(2 * y).min(size - 1), (2 * y + 1).min(size - 1)];
            let texels = [(xs[0], ys[0]), (xs[1], ys[0]), (xs[0], ys[1]), (xs[1], ys[1])];

            for channel in 0..3 {
                let sum: f32 = texels.iter().map(|&(tx, ty)| lut[src[((ty * size + tx) * 4) as usize + channel] as usize]).sum();
                dst.push(linear_to_srgb(sum / 4.0));
            }
            let alpha: u32 = texels.iter().map(|&(tx, ty)| src[((ty * size + tx) * 4) as usize + 3] as u32).sum();
            dst.push(((alpha + 2) / 4) as u8);
        }
    }

    (dst, out)
}

/// Mip level 0 is `tile`; every further level halves the size down to 1x1.
fn mip_chain(tile: &[u8], size: u32) -> Vec<Vec<u8>> {
    let mut chain = vec![tile.to_vec()];
    let mut current = (tile.to_vec(), size);

    while current.1 > 1 {
        current = downsample(&current.0, current.1);
        chain.push(current.0.clone());
    }

    chain
}

/// Where one mip level (all layers) sits in the staging buffer.
struct MipLevel {
    mip: u32,
    size: u32,
    offset: u64,
}

/// Staging layout: for each mip level, all layers one after another (what `copy_buffer_to_image`
/// expects for a copy that covers several layers).
fn pack_mips(tiles: &TileSet) -> (Vec<u8>, Vec<MipLevel>) {
    let chains: Vec<Vec<Vec<u8>>> = tiles.tiles.iter().map(|tile| mip_chain(tile, tiles.size)).collect();
    let mip_count = tiles.size.ilog2() as usize + 1;

    let mut data = Vec::new();
    let mut levels = Vec::new();

    for mip in 0..mip_count {
        levels.push(MipLevel {
            mip: mip as u32,
            size: (tiles.size >> mip).max(1),
            offset: data.len() as u64,
        });
        for chain in &chains {
            data.extend_from_slice(&chain[mip]);
        }
    }

    (data, levels)
}
