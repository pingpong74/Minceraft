#[derive(Clone, Copy, PartialEq)]
pub struct Block {
    id: u8,
}

impl Block {
    #[inline]
    pub fn is_air(&self) -> bool {
        return self.id == 0;
    }

    #[inline]
    pub fn get_id(&self) -> u8 {
        return self.id;
    }

    pub const AIR: Block = Block { id: 0 };
    pub const GRASS: Block = Block { id: 1 };
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq)]
pub struct Face {
    data: u32,
}

// x, y and z range from 0 to 31.
// normal is 3 bits.
// block_id is 16 bits
// block | normal | z | y | x.
impl Face {
    pub fn new(x: u32, y: u32, z: u32, normal: u32, block: Block) -> Face {
        return Face {
            data: (x | (y << 5) | (z << 10) | (normal << 15) | ((block.get_id() as u32) << 18)),
        };
    }
}
