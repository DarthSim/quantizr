use std::os::raw::c_uchar;
use std::slice;

pub struct CData {
    ptr: *mut c_uchar,
    len: usize,
}

impl CData {
    pub fn new(ptr: *mut c_uchar, len: usize) -> Self {
        Self{ptr: ptr, len: len}
    }
}

impl std::ops::Deref for CData {
    type Target = [c_uchar];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl std::ops::DerefMut for CData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

#[repr(C)]
pub struct Image {
    pub data: CData,
    pub width: usize,
    pub height: usize,
}

impl Image {
    pub fn new(data_ptr: *mut c_uchar, width: i32, height: i32) -> Self {
        let uwidth: usize = width as usize;
        let uheight: usize = height as usize;
        let size = uwidth * uheight * 4;

        let data = CData::new(data_ptr, size);

        Self{
            data: data,
            width: uwidth,
            height: uheight,
        }
    }
}
