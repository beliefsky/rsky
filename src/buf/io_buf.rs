use std::mem::MaybeUninit;

pub trait IoBuf: 'static {
    /// Get the slice of initialized bytes.
    fn as_init(&self) -> &[u8];
}

pub trait IoBufMut: IoBuf {
    fn as_uninit(&mut self) -> &mut [MaybeUninit<u8>];
}
