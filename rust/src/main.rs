use std::env;
use std::io::BufRead;
use std::{collections::HashMap, fs::File, io::BufReader, str::FromStr};

use pprof::protos::Message;
use std::io::Write;

fn main() {
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();

    let map = compute_temperatures();
    let s = format(map);
    println!("{s}");

    if let Ok(report) = guard.report().build() {
        let mut file = File::create("profile.pb").unwrap();
        let profile = report.pprof().unwrap();
        let mut content = Vec::new();
        profile.encode(&mut content).unwrap();
        file.write_all(&content).unwrap();
    };
}

fn compute_temperatures() -> HashMap<City, Temperature> {
    let file_path = env::var("FILE").unwrap();
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let mut map: HashMap<City, Temperature> = HashMap::with_capacity(10000);
    for line in reader.lines() {
        let line = line.unwrap();
        let (city, temperature) = parse_temperature(&line);

        let found_temperature = map.get_mut(city);
        if found_temperature.is_some() {
            let found_temperature = found_temperature.unwrap();

            if temperature < found_temperature.minimum {
                found_temperature.minimum = temperature;
            }

            if temperature > found_temperature.maximum {
                found_temperature.maximum = temperature;
            }

            found_temperature.sum += temperature;
            found_temperature.n += 1;
        } else {
            let new_temperature = Temperature {
                minimum: temperature,
                maximum: temperature,
                sum: temperature,
                n: 1,
            };
            map.insert(String::from_str(city).unwrap(), new_temperature);
        }
    }

    map
}

fn parse_temperature(line: &str) -> (&str, f32) {
    let (city, temperature) = line.split_once(';').unwrap();
    (city, temperature.parse().unwrap())
}

fn format(map: HashMap<City, Temperature>) -> String {
    let mut stations: Vec<String> = Vec::with_capacity(map.len());
    for (city, temperature) in map.iter() {
        let mean = temperature.sum / (temperature.n as f32);
        let mean = format!("{:.1}", mean);

        let station = format!(
            "{city}={:.1}/{}/{:.1}",
            temperature.minimum, mean, temperature.maximum
        );
        stations.push(station);
    }

    stations.sort();
    let measures = stations.join(", ");

    format!("{{{}}}", measures)
}

type City = String;

struct Temperature {
    minimum: f32,
    maximum: f32,
    sum: f32,
    n: u32,
}
