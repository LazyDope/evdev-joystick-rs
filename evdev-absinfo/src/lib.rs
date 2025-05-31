use std::{fmt::Display, fs::File, mem::MaybeUninit, os::fd::AsRawFd};

use libc::{ABS_MAX, input_absinfo as CAbsInfo};
use nix::errno::Errno;

mod raw;

#[derive(Debug)]
struct AbsInfo {
    axis: u16,
    inner: CAbsInfo,
}

impl AbsInfo {
    fn set_absinfo(self, file: &mut File) -> Result<(), Error> {
        let fd = file.as_raw_fd();
        let int_result = unsafe { libc::ioctl(fd, raw::evioc_set_abs(self.axis), &self.inner) };
        Errno::result(int_result).map(|_| ()).map_err(|e| e.into())
    }

    fn get_absinfo(file: &File, axis_index: u16) -> Result<Self, Error> {
        if axis_index > ABS_MAX {
            return Err(Error::AxisErr(axis_index));
        }
        let mut abs_info: MaybeUninit<CAbsInfo> = MaybeUninit::uninit();
        let fd = file.as_raw_fd();
        let int_result = unsafe { libc::ioctl(fd, raw::evioc_get_abs(axis_index), &mut abs_info) };
        Errno::result(int_result)?;
        Ok(AbsInfo {
            axis: axis_index,
            inner: unsafe { abs_info.assume_init() },
        })
    }
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("ioctl error: {0}")]
    CErr(#[from] Errno),
    #[error("invalid axis: {0}, must be between 0 and {ABS_MAX}")]
    AxisErr(u16),
}

impl Display for AbsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        let flat_percent = flat as f64 / (maximum - minimum) as f64 * 100.;
        write!(
            f,
            "Absolute axis {0:#x} ({0}) (value: {1}, min: {2}, max: {3}, flatness: {4} (={5:.2}%), fuzz: {6})",
            axis, value, minimum, maximum, flat, flat_percent, fuzz
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut file = File::open("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
            .expect("This device is available");
        let mut abs_info = AbsInfo::get_absinfo(&file, 2).expect("Axis 2 on this device is valid");
        println!("{}", abs_info);
        abs_info.inner.maximum /= 2;
        abs_info
            .set_absinfo(&mut file)
            .expect("Setting the maximum value to half should always succeed");
        let mut abs_info = AbsInfo::get_absinfo(&file, 2).expect("Axis 2 on this device is valid");
        println!("{}", abs_info);
    }
}
