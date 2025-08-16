use image::GenericImage;

#[repr(usize)]
#[derive(Clone)]
pub enum Blocks {
    Moss = 0,
    Dirt = 1,
    Stone = 2,
}

const BLOCK_TEXTURE: &[(Blocks, &str)] = &[
    (Blocks::Moss, "textures/moss_block.png"),
    (Blocks::Dirt, "textures/dirt.png"),
    (Blocks::Stone, "textures/stone.png"),
];

pub const TILE_SIZE: u32 = 16;
pub const ATLAS_SIZE: u32 = 256;

pub fn create_texture_atlas() {
    let mut atlas = image::RgbaImage::new(ATLAS_SIZE, TILE_SIZE);
    for (i, (block, path)) in BLOCK_TEXTURE.iter().enumerate() {
        let image = image::open(path).expect("Failed to open image");

        assert!(
            image.width() == TILE_SIZE
                && image.height() == TILE_SIZE
                && i == block.clone() as usize
        );

        atlas.copy_from(&image, i as u32 * TILE_SIZE, 0).unwrap();
    }

    image::save_buffer(
        "textures/atlas.png",
        atlas.into_raw().as_slice(),
        ATLAS_SIZE,
        TILE_SIZE,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
