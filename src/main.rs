use std::{error::Error, fs};

const CPU_TEMP_FILE: &str = "/sys/class/thermal/thermal_zone0/temp";

fn main() {
    match read_cpu_temp(CPU_TEMP_FILE) {
        Ok(temp) => {
            println!("CPU Temp: {:.1} C", temp);
        },
        Err(err) => {
            eprintln!("Failed to read CPU temperature: {:?}", err);
        }
    }
}

fn read_cpu_temp(path: &str) -> Result<f32, Box<dyn Error>> {
    let raw_temp = fs::read_to_string(path)?;
    let parsed_temp_millis: f32 = raw_temp.trim().parse()?;

    Ok(parsed_temp_millis / 1000_f32)
}
