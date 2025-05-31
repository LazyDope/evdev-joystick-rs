use libc::input_absinfo;
use nix::{request_code_read, request_code_write};

const IOC_NRBITS: u32 = 8;
const IOC_TYPEBITS: u32 = 8;
cfg_if::cfg_if! {
    if #[cfg(any(
        any(target_arch = "powerpc", target_arch = "powerpc64"),
        any(target_arch = "sparc", target_arch = "sparc64"),
        any(target_arch = "mips", target_arch = "mips64"),
    ))] {
        const IOC_SIZEBITS: u32 = 13;
        const IOC_DIRBITS: u32 = 3;

        pub const IOC_NONE: u32 = 0b1;
        pub const IOC_READ: u32 = 0b10;
        pub const IOC_WRITE: u32 = 0b100;
    } else {
        const IOC_SIZEBITS: u32 = 14;
        const IOC_DIRBITS: u32 = 2;

        pub const IOC_NONE: u32 = 0b0;
        pub const IOC_WRITE: u32 = 0b1;
        pub const IOC_READ: u32 = 0b10;
    }
}

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
