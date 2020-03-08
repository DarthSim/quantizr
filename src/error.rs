#[repr(C)]
pub enum Error {
    Ok,
    ValueOutOfRange,
    BufferTooSmall,
}
