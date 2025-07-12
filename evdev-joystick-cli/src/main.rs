use std::path::PathBuf;

use clap::Parser;
use evdev_joystick::Joystick;
use evdev_rs::{InputEvent, enums::EventType};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    device: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let joystick = Joystick::new_from_path(args.device)?;
    for event in joystick.events() {
        let InputEvent {
            time,
            event_code,
            value,
        } = event;
        match event.event_type() {
            Some(EventType::EV_ABS) => {
                let abs_info = joystick
                    .abs_info(&event_code)
                    .expect("Joystick axis must be enabled");
                println!(
                    "{}.{}: code {}, {}",
                    time.tv_sec, time.tv_usec, event_code, abs_info
                );
            }
            Some(EventType::EV_KEY) => {
                println!(
                    "{}.{}: code BTN_{:?}, {}",
                    time.tv_sec,
                    time.tv_usec,
                    joystick
                        .get_button_index(&event_code)
                        .expect("Button pressed must be enabled")
                        + 1,
                    value
                );
            }
            Some(_) => (),
            None => (),
        }
    }
    Ok(())
}
