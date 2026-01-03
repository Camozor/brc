use std::env;
use std::fs::OpenOptions;
use std::{fs::File, str::FromStr};

use ahash::AHashMap;
use memmap2::Mmap;
use pprof::protos::Message;
use std::io::Write;

fn main() {
    let profiling = env::var("PROFILING").unwrap_or(String::from("false"));
    let profiling = profiling == "true";

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();

    let map = compute_temperatures();
    let s = format(map);
    println!("{s}");

    if profiling && let Ok(report) = guard.report().build() {
        let mut file = File::create("profile.pb").unwrap();
        let profile = report.pprof().unwrap();
        let mut content = Vec::new();
        profile.encode(&mut content).unwrap();
        file.write_all(&content).unwrap();
    };
}

fn compute_temperatures() -> AHashMap<City, Temperature> {
    let mut map: AHashMap<City, Temperature> = AHashMap::with_capacity(10000);

    let file_path = env::var("FILE").unwrap();
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .open(file_path)
        .unwrap();

    let mmap = unsafe { Mmap::map(&file).unwrap() };

    for line in mmap.split(|&byte| byte == b'\n') {
        if line.is_empty() {
            continue;
        }

        let line_str = unsafe { std::str::from_utf8_unchecked(line) };
        let line = line_str.to_owned();
        let (city, temperature) = parse_temperature(&line);
        let temperature = (temperature * 10.) as i32;

        let found_temperature = map.get_mut(city);
        if found_temperature.is_some() {
            let found_temperature = found_temperature.unwrap();

            if temperature < found_temperature.minimum {
                found_temperature.minimum = temperature;
            }

            if temperature > found_temperature.maximum {
                found_temperature.maximum = temperature;
            }

            found_temperature.sum += temperature as i64;
            found_temperature.n += 1;
        } else {
            let new_temperature = Temperature {
                minimum: temperature,
                maximum: temperature,
                sum: temperature as i64,
                n: 1,
            };
            map.insert(String::from_str(city).unwrap(), new_temperature);
        }
    }

    map
}

fn parse_temperature(line: &str) -> (&str, f32) {
    let index = line.find(';').unwrap();
    let city = &line[..index];
    let temperature = &line[index + 1..];
    let temperature = parse_number(temperature);
    (city, temperature)
}

fn parse_number(s: &str) -> f32 {
    let mut parsed: f32;
    let mut chars = s.chars();
    let mut negative = false;

    let mut c = chars.next().unwrap();
    if c == '-' {
        negative = true;
        c = chars.next().unwrap();
    }
    parsed = convert_char_to_f32(c);

    c = chars.next().unwrap();
    if c != '.' {
        parsed = parsed * 10. + convert_char_to_f32(c);
        chars.next().unwrap();
    }
    c = chars.next().unwrap();

    parsed += convert_char_to_f32(c) / 10.;

    if negative {
        parsed = -parsed;
    }

    parsed
}

fn convert_char_to_f32(c: char) -> f32 {
    ((c as u32) - (48 as u32)) as f32
}

fn format(map: AHashMap<City, Temperature>) -> String {
    let mut stations: Vec<String> = Vec::with_capacity(map.len());
    for (city, temperature) in map.iter() {
        let minimum = (temperature.minimum as f32) / 10.;
        let maximum = (temperature.maximum as f32) / 10.;

        let mean = temperature.sum / (temperature.n as i64);
        let mean = (mean as f32) / 10.;
        let mean = format!("{:.1}", mean);

        let station = format!("{city}={:.1}/{}/{:.1}", minimum, mean, maximum);
        stations.push(station);
    }

    stations.sort();
    let measures = stations.join(", ");

    format!("{{{}}}", measures)
}

type City = String;

struct Temperature {
    minimum: i32,
    maximum: i32,
    sum: i64,
    n: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("8.3"), 8.3);
        assert_eq!(parse_number("89.2"), 89.2);
        assert_eq!(parse_number("-8.3"), -8.3);
        assert_eq!(parse_number("-87.3"), -87.3);
    }
}
