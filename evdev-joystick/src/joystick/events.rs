use evdev_rs::{Device, InputEvent, ReadFlag, ReadStatus};

pub struct JoystickEvents<'a>(pub(crate) &'a Device);

impl<'a> Iterator for JoystickEvents<'a> {
    type Item = InputEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let mut read_flag = ReadFlag::NORMAL;
        loop {
            match self.0.next_event(read_flag) {
                Ok((status, event)) => match status {
                    ReadStatus::Success => return Some(event),
                    ReadStatus::Sync => read_flag = ReadFlag::SYNC,
                },
                Err(e) => match e.raw_os_error() {
                    Some(libc::EAGAIN) => read_flag = ReadFlag::NORMAL,
                    _ => {
                        eprintln!("{}", e);
                        return None;
                    }
                },
            }
        }
    }
}
