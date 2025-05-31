use std::{
    fmt::Display,
    fs::File,
    mem::MaybeUninit,
    os::fd::{AsRawFd, RawFd},
};

use libc::{ABS_MAX, input_absinfo as CAbsInfo};
use nix::errno::Errno;

mod raw;
use raw::AbsBitMask;

#[derive(Debug)]
pub struct AbsInfo {
    axis: u16,
    pub inner: CAbsInfo,
}

impl AbsInfo {
    pub fn set_absinfo(self, file: &mut File) -> Result<(), Error> {
        let fd = file.as_raw_fd();
        check_valid_axis(fd, self.axis)?;
        let int_result =
            unsafe { libc::ioctl(fd, raw::evioc_set_abs(self.axis), &raw const self.inner) };
        Errno::result(int_result)?;
        Ok(())
    }

    pub fn get_absinfo(file: &File, axis_index: u16) -> Result<Self, Error> {
        if axis_index > ABS_MAX {
            return Err(Error::UnboundAxis(axis_index));
        }
        let fd = file.as_raw_fd();
        check_valid_axis(fd, axis_index)?;
        let mut abs_info: MaybeUninit<CAbsInfo> = MaybeUninit::zeroed();
        let int_result =
            unsafe { libc::ioctl(fd, raw::evioc_get_abs(axis_index), abs_info.as_mut_ptr()) };
        Errno::result(int_result)?;
        Ok(AbsInfo {
            axis: axis_index,
            inner: unsafe { abs_info.assume_init() },
        })
    }

    pub fn get_normalized_value(&self) -> i16 {
        let &AbsInfo {
            inner:
                CAbsInfo {
                    value,
                    minimum,
                    maximum,
                    flat,
                    ..
                },
            ..
        } = self;
        const I16_RANGE: i64 = u16::MAX as i64;
        let value = i64::from(value.max(minimum).min(maximum));
        let range_size = i64::from(maximum) - i64::from(minimum);
        let translation = i64::from(i16::MIN) - i64::from(minimum);
        let norm_value = i16::try_from(value * I16_RANGE / range_size + translation)
            .expect("This value should always be within i16 range");
        apply_flatness(norm_value, flat)
    }
}

fn apply_flatness(value: i16, flat: i32) -> i16 {
    if (value as i32) >= (-flat).div_euclid(2) && (value as i32) <= flat.div_euclid(2) {
        0
    } else {
        value
    }
}

fn check_valid_axis(fd: RawFd, axis_index: u16) -> Result<(), Error> {
    // Check axis is an abs axis
    let mut abs_bitmask: MaybeUninit<AbsBitMask> = MaybeUninit::zeroed();
    unsafe { raw::evioc_get_abs_bit(fd, abs_bitmask.as_mut_ptr()) }?;
    let abs_bitmask = unsafe { abs_bitmask.assume_init() };
    if !test_axis(axis_index, &abs_bitmask) {
        return Err(Error::NonAbsAxis(axis_index));
    }
    Ok(())
}

#[inline(always)]
const fn test_axis(axis_index: u16, bit_array: &AbsBitMask) -> bool {
    bit_array[(axis_index / 8) as usize] & (1 << (axis_index % 8)) != 0
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("ioctl error: {0}")]
    CErrno(#[from] Errno),
    #[error("invalid axis: {0}, must be between 0 and {ABS_MAX}")]
    UnboundAxis(u16),
    #[error("invalid axis: {0}, must be a valid ABS axis for this device")]
    NonAbsAxis(u16),
}

impl Display for AbsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let norm = self.get_normalized_value();
        let &AbsInfo {
            axis,
            inner:
                CAbsInfo {
                    value,
                    minimum,
                    maximum,
                    fuzz,
                    flat,
                    ..
                },
        } = self;
        let flat_percent = f64::from(flat) / f64::from(maximum - minimum) * 100.;
        write!(
            f,
            "Absolute axis {0:#x} ({0}) (value: {1} (norm: {7}), min: {2}, max: {3}, flatness: {4} (={5:.2}%), fuzz: {6})",
            axis, value, minimum, maximum, flat, flat_percent, fuzz, norm
        )
    }
}

// these tests only work on my machine until I can read all devices
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_read_and_write() {
        let mut file = File::open("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
            .expect("This device is available");
        let mut abs_info = AbsInfo::get_absinfo(&file, 2).expect("Axis 2 on this device is valid");
        println!("{}", abs_info);
        abs_info.inner.maximum /= 2;
        let temp = abs_info.inner.maximum;
        abs_info
            .set_absinfo(&mut file)
            .expect("Setting the maximum value to half should always succeed");
        let abs_info = AbsInfo::get_absinfo(&file, 2).expect("Axis 2 on this device is valid");
        assert_eq!(abs_info.inner.maximum, temp);
        println!("{}", abs_info);
    }

    #[test]
    fn test_read() {
        let file = File::open("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
            .expect("This device is available");
        let abs_info = AbsInfo::get_absinfo(&file, 2).expect("Axis 2 on this device is valid");
        println!("{}", abs_info);
    }

    #[test]
    fn test_out_of_bounds_axis() {
        let file = File::open("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
            .expect("This device is available");
        assert!(matches!(
            AbsInfo::get_absinfo(&file, ABS_MAX + 1),
            Err(Error::UnboundAxis(_))
        ));
    }

    #[test]
    fn test_invalid_axis() {
        let file = File::open("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
            .expect("This device is available");
        assert!(matches!(
            AbsInfo::get_absinfo(&file, 8),
            Err(Error::NonAbsAxis(_))
        ));
    }
}
