#[repr(usize)]
#[derive(Clone)]
pub enum FaceType {
    Moss = 0,
    Dirt = 1,
    Stone = 2,
}

const FACE_TEXTURE: &[(FaceType, &str)] = &[
    (FaceType::Moss, "textures/moss_block.png"),
    (FaceType::Dirt, "textures/dirt.png"),
    (FaceType::Stone, "textures/stone.png"),
];

pub const TILE_SIZE: u32 = 16;
pub const ATLAS_SIZE: u32 = 256;

pub fn create_texture_atlas() {
    let mut atlas = image::RgbaImage::new(ATLAS_SIZE, TILE_SIZE);
    for (i, (face, path)) in FACE_TEXTURE.iter().enumerate() {
        let image = image::open(path).expect("Failed to open image");

        assert!(i == face.clone() as usize);

        use image::GenericImage;
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

#[repr(usize)]
#[derive(Clone, PartialEq, Copy)]
pub enum BlockType {
    Air = 0,
    Moss = 1,
    Dirt = 2,
    Stone = 3,
}

#[derive(Clone)]
pub struct Block {
    pub block_type: BlockType,
    pub face: FaceType,
}

pub const ALL_BLOCKS: &[Block] = &[
    Block {
        block_type: BlockType::Air,
        face: FaceType::Moss,
    },
    Block {
        block_type: BlockType::Air,
        face: FaceType::Moss,
    },
];
