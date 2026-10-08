pub struct RecvFlags(u32);

impl RecvFlags {
    pub const EMPTY: Self = Self(0);

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn flags(&self) -> u32 {
        self.0
    }
}

pub struct SendFlags(u32);

impl SendFlags {
    pub const EMPTY: Self = Self(0);

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn flags(&self) -> u32 {
        self.0
    }
}
