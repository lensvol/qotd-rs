
pub struct StrfileHeader {
    pub version: u32,
    pub number_of_strings: u32,
    pub longest_length: u32,
    pub shortest_length: u32,
    pub flags: u32,
    pub delim: u8,
    pub offsets: Vec<u32>,
}

pub enum Flags {
    Random = 0x1,
    Ordered = 0x2,
    Rotated = 0x4,
    HasComments = 0x8
}

impl StrfileHeader {
    pub fn flag_is_set(&self, mask: Flags) -> bool {
        self.flags & (mask as u32) == 1
    }

    pub fn is_random(&self) -> bool {
        self.flag_is_set(Flags::Random)
    }

    pub fn is_rotated(&self) -> bool {
        self.flag_is_set(Flags::Rotated)
    }

    pub fn is_ordered(&self) -> bool {
        self.flag_is_set(Flags::Ordered)
    }

    pub fn has_comments(&self) -> bool {
        self.flag_is_set(Flags::HasComments)
    }
}

