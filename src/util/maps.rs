use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use rhai::{CustomType, TypeBuilder};

#[derive(Debug, Clone, CustomType)]
pub struct MapEntry {
    pub start: usize,
    pub end: usize,
    pub offset: usize,
    pub flags: String,
    pub pathname: String,
}

impl MapEntry {
    pub fn new(start: usize, end: usize, offset: usize, flags: String, pathname: String) -> Self {
        Self {
            start,
            end,
            offset,
            flags,
            pathname,
        }
    }
}

pub fn parse_from_file(pid: i32) -> Result<Vec<MapEntry>> {
    let file = File::open(format!("/proc/{}/maps", pid))?;
    let reader = BufReader::new(file);

    let mut maps = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        let start_end: Vec<&str> = parts[0].split('-').collect();
        let start = usize::from_str_radix(start_end[0], 16)?;
        let end = usize::from_str_radix(start_end[1], 16)?;

        let offset = usize::from_str_radix(parts[2], 16)?;

        let flags = parts[1].to_string();
        let pathname = parts.last().unwrap_or(&"").to_string();

        // println!("{:x} {:x} {:x} {} {}", start, end, offset, flags, pathname);

        maps.push(MapEntry::new(start, end, offset, flags, pathname));
    }

    Ok(maps)
}

// Rhai-specific traits implemented in different files