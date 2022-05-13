use std::{error::Error, fs, thread, time::Duration};

use rppal::pwm::Pwm;

const CPU_TEMP_FILE: &str = "/sys/class/thermal/thermal_zone0/temp";

fn main() -> Result<(), Box<dyn Error>> {
    match read_cpu_temp(CPU_TEMP_FILE) {
        Ok(temp) => {
            println!("CPU Temp: {:.1} C", temp);
        }
        Err(err) => {
            eprintln!("Failed to read CPU temperature: {:?}", err);
        }
    };

    let pin = Pwm::with_frequency(
        rppal::pwm::Channel::Pwm0,
        50.0,
        0.25,
        rppal::pwm::Polarity::Normal,
        true,
    )?;

    thread::sleep(Duration::from_secs(3));

    pin.set_duty_cycle(0.5)?;
    thread::sleep(Duration::from_secs(3));

    pin.set_duty_cycle(0.75)?;
    thread::sleep(Duration::from_secs(3));

    pin.set_duty_cycle(1.0)?;
    thread::sleep(Duration::from_secs(3));

    Ok(())
}

fn read_cpu_temp(path: &str) -> Result<f32, Box<dyn Error>> {
    let raw_temp = fs::read_to_string(path)?;
    let parsed_temp_millis: f32 = raw_temp.trim().parse()?;

    Ok(parsed_temp_millis / 1000_f32)
}
