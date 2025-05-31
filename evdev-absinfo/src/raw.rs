use libc::{ABS_MAX, input_absinfo};
use nix::{ioctl_read, request_code_read, request_code_write};

const EV_IOC_MAGIC: u8 = b'E';
// EVIOCSABS from linux/input.h
pub const fn evioc_set_abs(axis_index: u16) -> u64 {
    request_code_write!(EV_IOC_MAGIC, 0xc0 + axis_index, size_of::<input_absinfo>())
}

// EVIOCGABS from linux/input.h
pub const fn evioc_get_abs(axis_index: u16) -> u64 {
    request_code_read!(EV_IOC_MAGIC, 0x40 + axis_index, size_of::<input_absinfo>())
}

// EVIOCGBIT from linux/input.h using EV_ABS from linux/input-event-codes.h
const EV_ABS: u8 = 0x03;
pub type AbsBitMask = [u8; ABS_MAX as usize / 8 + 1];
ioctl_read!(evioc_get_abs_bit, EV_IOC_MAGIC, 0x20 + EV_ABS, AbsBitMask);
