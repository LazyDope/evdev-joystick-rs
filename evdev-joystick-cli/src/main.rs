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
    for event in joystick
        .events()
        .filter(|event| event.is_type(&EventType::EV_ABS))
    {
        let InputEvent {
            time, event_code, ..
        } = event;
        let abs_info = joystick.abs_info(&event_code).unwrap();
        println!(
            "{}.{}: code {}, {}",
            time.tv_sec, time.tv_usec, event_code, abs_info
        );
    }
    Ok(())
}
