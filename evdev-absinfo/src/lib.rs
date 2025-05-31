use std::fmt::Display;

use evdev_rs::{
    AbsInfo as RawAbsInfo, Device, DeviceWrapper,
    enums::{EV_ABS, EventCode},
};

#[derive(Debug)]
pub struct AbsInfo(RawAbsInfo);

impl AbsInfo {
    pub fn set_absinfo(&self, device: &Device, axis: EV_ABS) {
        device.set_abs_info(&EventCode::EV_ABS(axis), &self.0);
    }

    pub fn get_absinfo(device: &Device, axis: EV_ABS) -> Option<Self> {
        device.abs_info(&EventCode::EV_ABS(axis)).map(AbsInfo)
    }

    pub fn get_normalized_value(&self) -> i16 {
        let &AbsInfo(RawAbsInfo {
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

impl Display for AbsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let norm = self.get_normalized_value();
        let &AbsInfo(RawAbsInfo {
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

// these tests only work on my machine until I can read all devices
#[cfg(test)]
mod tests {
    use evdev_rs::ReadFlag;

    use super::*;

    #[test]
    #[ignore]
    fn test_read_and_write() {
        let mut device =
            Device::new_from_path("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
                .expect("This device is available");
        let mut abs_info =
            AbsInfo::get_absinfo(&device, EV_ABS::ABS_Z).expect("Axis 2 on this device is valid");
        println!("{}", abs_info);
        abs_info.0.maximum /= 2;
        let temp = abs_info.0.maximum;
        abs_info.set_absinfo(&device, EV_ABS::ABS_Z);
        let abs_info =
            AbsInfo::get_absinfo(&device, EV_ABS::ABS_Z).expect("Axis 2 on this device is valid");
        assert_eq!(abs_info.0.maximum, temp);
        println!("{}", abs_info);
    }

    #[test]
    fn test_read() {
        let device =
            Device::new_from_path("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
                .expect("This device is available");
        let old_val = device
            .event_value(&EventCode::EV_ABS(EV_ABS::ABS_Z))
            .expect("Axis 2 on this device is valid");
        loop {
            let a = device.next_event(ReadFlag::NORMAL);
            match a {
                Ok(res) => match res.0 {
                    evdev_rs::ReadStatus::Success => {
                        let new_val = device
                            .event_value(&EventCode::EV_ABS(EV_ABS::ABS_Z))
                            .expect("Axis 2 on this device is valid");
                        if new_val != old_val {
                            break;
                        }
                    }
                    evdev_rs::ReadStatus::Sync => {}
                },
                Err(e) => match e.raw_os_error() {
                    Some(libc::EAGAIN) => continue,
                    _ => {
                        println!("{:?}", e);
                        break;
                    }
                },
            }
        }
        let abs_info =
            AbsInfo::get_absinfo(&device, EV_ABS::ABS_Z).expect("Axis 2 on this device is valid");
        println!("{}", abs_info);
    }

    #[test]
    fn test_invalid_axis() {
        let device =
            Device::new_from_path("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
                .expect("This device is available");
        assert!(AbsInfo::get_absinfo(&device, EV_ABS::ABS_WHEEL).is_none());
    }
}
