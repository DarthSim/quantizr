pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: &'static [u8],
}

impl Image {
    pub fn new(data: &'static [u8], width: usize, height: usize) -> Self {
        Self{
            data: data,
            width: width,
            height: height,
        }
    }
}
