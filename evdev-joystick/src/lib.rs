use std::{
    fmt::Display,
    fs, io,
    ops::{Deref, DerefMut},
    path::Path,
};

use evdev_rs::{
    AbsInfo, Device, DeviceWrapper,
    enums::{self, EV_ABS, EV_KEY, EV_REL, EventCode},
};

mod events;
pub use events::JoystickEvents;

#[derive(Debug)]
pub struct Joystick {
    device: Device,
}

impl Joystick {
    pub fn new_from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        Device::new_from_path(path).map(Joystick::from)
    }

    pub fn abs_info(&self, code: &EventCode) -> Option<JoystickAbsInfo> {
        self.device.abs_info(code).map(JoystickAbsInfo)
    }

    pub fn events<'a>(&'a self) -> JoystickEvents<'a> {
        JoystickEvents(&self.device)
    }

    pub fn joysticks() -> io::Result<impl Iterator<Item = io::Result<Joystick>>> {
        Ok(
            fs::read_dir("/dev/input/by-id/")?.filter_map(|entry| match entry {
                Ok(entry) => {
                    if entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .ends_with("-event-joystick")
                    {
                        Some(Joystick::new_from_path(entry.path()))
                    } else {
                        None
                    }
                }
                Err(e) => Some(Err(e)),
            }),
        )
    }

    pub fn buttons(&self) -> impl Iterator<Item = EV_KEY> {
        (0..EV_KEY::KEY_MAX as u32)
            .filter_map(|i| enums::int_to_ev_key(i).filter(|&key| self.has(EventCode::EV_KEY(key))))
    }

    pub fn abs_axis(&self) -> impl Iterator<Item = EV_ABS> {
        (0..EV_ABS::ABS_MAX as u32)
            .filter_map(|i| enums::int_to_ev_abs(i).filter(|&key| self.has(EventCode::EV_ABS(key))))
    }

    pub fn rel_axis(&self) -> impl Iterator<Item = EV_REL> {
        (0..EV_REL::REL_MAX as u32)
            .filter_map(|i| enums::int_to_ev_rel(i).filter(|&key| self.has(EventCode::EV_REL(key))))
    }
}

pub struct JoystickAbsInfo(AbsInfo);

impl JoystickAbsInfo {
    fn normalized_value(&self) -> i16 {
        let &JoystickAbsInfo(AbsInfo {
            value,
            minimum,
            maximum,
            flat,
            ..
        }) = self;

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

impl Display for JoystickAbsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let norm = self.normalized_value();
        let &JoystickAbsInfo(AbsInfo {
            value,
            minimum,
            maximum,
            fuzz,
            flat,
            ..
        }) = self;
        let flat_percent = f64::from(flat) / f64::from(maximum - minimum) * 100.;
        write!(
            f,
            "(value: {0} (norm: {6}), min: {1}, max: {2}, flatness: {3} (={4:.2}%), fuzz: {5})",
            value, minimum, maximum, flat, flat_percent, fuzz, norm
        )
    }
}

impl From<Device> for Joystick {
    fn from(device: Device) -> Self {
        Joystick { device }
    }
}

impl Deref for Joystick {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl DerefMut for Joystick {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

impl Deref for JoystickAbsInfo {
    type Target = AbsInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JoystickAbsInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// these tests only work on my machine until I can read all devices
#[cfg(test)]
mod tests {
    use super::*;

    fn find_a_joystick() -> Joystick {
        Joystick::joysticks()
            .expect("Devices are readable by id")
            .next()
            .expect("No joystick was found, tests require a joystick be connected.")
            .expect("Joystick could not be opened")
    }

    fn find_an_axis(joystick: &Joystick) -> EV_ABS {
        joystick
            .abs_axis()
            .next()
            .expect("Joystick must have at least one absolute axis")
    }

    #[test]
    #[ignore]
    fn test_read_and_write() {
        let device = find_a_joystick();
        let axis = find_an_axis(&device);
        let mut abs_info = device
            .abs_info(axis)
            .expect("Axis 2 on this device is valid");
        println!("{}", abs_info);
        abs_info.0.maximum /= 2;
        let temp = abs_info.maximum;
        device.set_abs_info(&abs_info, axis);
        let abs_info = device
            .abs_info(axis)
            .expect("Axis 2 on this device is valid");
        assert_eq!(abs_info.maximum, temp);
        println!("{}", abs_info);
    }

    #[test]
    fn test_read() {
        let device = find_a_joystick();
        let axis = device
            .abs_axis()
            .next()
            .expect("Joystick must have at least one absolute axis");
        let abs_info = device
            .abs_info(axis)
            .expect("Axis 2 on this device is valid");
        println!("{}", abs_info);
    }

    #[test]
    fn test_invalid_axis() {
        let device = find_a_joystick();
        assert!(device.abs_info(EV_ABS::ABS_RESERVED).is_none());
    }
}
