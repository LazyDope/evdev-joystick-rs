use libc::input_absinfo;
use nix::{request_code_read, request_code_write};

pub const unsafe fn evioc_set_abs(axis_index: u16) -> u64 {
    request_code_write!(
        b'E' as u32,
        0xc0 + axis_index as u32,
        size_of::<input_absinfo>()
    )
}

pub const unsafe fn evioc_get_abs(axis_index: u16) -> u64 {
    request_code_read!(
        b'E' as u32,
        0x40 + axis_index as u32,
        size_of::<input_absinfo>()
    )
}
