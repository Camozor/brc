use std::collections::hash_map::Entry;
use std::fs::OpenOptions;
use std::sync::mpsc;
use std::{env, thread};
use std::{fs::File, str::FromStr};

use memmap2::Mmap;
use pprof::protos::Message;
use std::io::Write;

use fnv::FnvHashMap;

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

    if profiling {
        if let Ok(report) = guard.report().build() {
            let mut file = File::create("profile.pb").unwrap();
            let profile = report.pprof().unwrap();
            let mut content = Vec::new();
            profile.encode(&mut content).unwrap();
            file.write_all(&content).unwrap();
        };
    }
}

#[no_mangle]
pub extern "C" fn compute_and_format() -> String {
    let map = compute_temperatures();
    format(map)
}

fn compute_temperatures() -> FnvHashMap<City, StationStat> {
    let file_path = env::var("FILE").unwrap();
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .open(file_path)
        .unwrap();

    let whole_memory_map = unsafe { Mmap::map(&file).unwrap() };

    let mut map: FnvHashMap<City, StationStat> =
        FnvHashMap::with_capacity_and_hasher(10_000, Default::default());

    thread::scope(|s| {
        let n_threads = thread::available_parallelism().unwrap().get();
        let (sender, receiver) = mpsc::sync_channel(n_threads);

        let chunk_size = whole_memory_map.len() / n_threads;
        let mut pos = 0;

        for _ in 0..n_threads {
            let start = pos;
            let end = (pos + chunk_size).min(whole_memory_map.len());
            let end = if end == map.len() {
                map.len()
            } else {
                let newline_pos = find_newline(&whole_memory_map[end..]);
                end + newline_pos
            };

            let memory_chunk = &whole_memory_map[start..end];
            pos = end;

            let sender = sender.clone();
            s.spawn(move || sender.send(compute_temperatures_chunk(memory_chunk)));
        }
        drop(sender);

        for chunk_map in receiver {
            for (city, chunk_stat) in chunk_map {
                match map.entry(city) {
                    Entry::Vacant(none) => {
                        none.insert(chunk_stat);
                    }
                    Entry::Occupied(some) => {
                        let stat = some.into_mut();
                        stat.minimum = stat.minimum.min(chunk_stat.minimum);
                        stat.maximum = stat.maximum.max(chunk_stat.maximum);
                        stat.sum += chunk_stat.sum;
                        stat.n += chunk_stat.n;
                    }
                }
            }
        }
    });

    map
}

fn find_newline(data: &[u8]) -> usize {
    let mut pos = 0;
    for c in data {
        if *c == b'\n' {
            break;
        }
        pos += 1;
    }

    pos
}

fn compute_temperatures_chunk(memory_map: &[u8]) -> FnvHashMap<City, StationStat> {
    let mut map: FnvHashMap<City, StationStat> =
        FnvHashMap::with_capacity_and_hasher(10_000, Default::default());

    for line in memory_map.split(|&byte| byte == b'\n') {
        if line.is_empty() {
            continue;
        }

        let line_str = unsafe { std::str::from_utf8_unchecked(line) };
        let line = line_str.to_owned();
        let (city, temperature) = parse_temperature(&line);
        let temperature = (temperature * 10.) as i32;

        let found_station_stat = map.get_mut(city);
        if found_station_stat.is_some() {
            let found_station_stat = found_station_stat.unwrap();

            if temperature < found_station_stat.minimum {
                found_station_stat.minimum = temperature;
            }

            if temperature > found_station_stat.maximum {
                found_station_stat.maximum = temperature;
            }

            found_station_stat.sum += temperature as i64;
            found_station_stat.n += 1;
        } else {
            let new_station_stat = StationStat {
                minimum: temperature,
                maximum: temperature,
                sum: temperature as i64,
                n: 1,
            };
            map.insert(String::from_str(city).unwrap(), new_station_stat);
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

fn format(map: FnvHashMap<City, StationStat>) -> String {
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

#[derive(Debug, Clone, Copy)]
struct StationStat {
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
