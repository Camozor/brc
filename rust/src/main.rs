use std::io::{self, BufRead};
use std::{collections::HashMap, str::FromStr};

fn main() {
    let map = compute_temperatures();
    let s = format(map);
    println!("{s}");
}

fn compute_temperatures() -> HashMap<City, Temperature> {
    let lines = read_lines();

    let mut map: HashMap<City, Temperature> = HashMap::new();
    for line in lines {
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

fn read_lines() -> Vec<String> {
    io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap())
        .collect()
}

fn parse_temperature(line: &str) -> (&str, f32) {
    let splits: Vec<&str> = line.split_terminator(";").collect();

    let city = splits[0];
    let temperature: f32 = splits[1].parse().unwrap();

    (city, temperature)
}

fn format(map: HashMap<City, Temperature>) -> String {
    let mut stations: Vec<String> = vec![];
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
    n: u16,
}
