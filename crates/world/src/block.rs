macro_rules! textures {
    ($($name:ident = $file:literal),+ $(,)?) => {
        pub const TEXTURE_NAMES: &[&str] = &[$($file),+];

        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[repr(u16)]
        enum Layer { $($name),+ }

        $(pub const $name: u16 = Layer::$name as u16;)+
    };
}

textures! {
    TEX_MISSING = "missing",
    TEX_MOSS = "moss_block",
    TEX_FLOWER_MOSS = "flower_block",
    TEX_DIRT = "dirt",
    TEX_STONE = "stone",
}

pub type BlockId = u16;

pub const AIR: BlockId = 0;
pub const MOSS: BlockId = 1;
pub const FLOWER: BlockId = 2;
pub const STONE: BlockId = 3;
pub const DIRT: BlockId = 4;

pub struct BlockInfo {
    pub opaque: bool,
    pub textures: [u16; 6],
}

// Indexed by BlockId.
const BLOCKS: [BlockInfo; 5] = [
    BlockInfo { opaque: false, textures: [TEX_MISSING; 6] },
    BlockInfo { opaque: true, textures: [TEX_MOSS; 6] },
    BlockInfo { opaque: true, textures: [TEX_FLOWER_MOSS; 6] },
    BlockInfo { opaque: true, textures: [TEX_STONE; 6] },
    BlockInfo { opaque: true, textures: [TEX_DIRT; 6] },
];

pub fn info(block: BlockId) -> &'static BlockInfo {
    BLOCKS.get(block as usize).unwrap_or(&BLOCKS[AIR as usize])
}

pub fn is_opaque(block: BlockId) -> bool {
    info(block).opaque
}

/// Texture layer of `block` on face `face` (0..6, same order as `BlockInfo::textures`).
pub fn texture_layer(block: BlockId, face: usize) -> u16 {
    info(block).textures[face]
}
