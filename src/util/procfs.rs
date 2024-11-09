use anyhow::Result;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub struct MemoryMap {
    pub start: u64,
    pub end: u64,
    pub permissions: String,
    pub offset: u64,
    pub device: String,
}

impl MemoryMap {
    pub fn parse_maps(pid: u32) -> Result<Vec<MemoryMap>> {
        let path = format!("/proc/{}/maps", pid);
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let mut memory_maps = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 {
                continue; // Skip lines that don't have enough parts
            }

            let address_range: Vec<&str> = parts[0].split('-').collect();
            if address_range.len() != 2 {
                continue; // Skip malformed lines
            }

            let start = u64::from_str_radix(address_range[0], 16).unwrap_or(0);
            let end = u64::from_str_radix(address_range[1], 16).unwrap_or(0);
            let permissions = parts[1].to_string();
            let offset = u64::from_str_radix(parts[2], 16).unwrap_or(0);
            let device = parts[3].to_string();

            memory_maps.push(MemoryMap {
                start,
                end,
                permissions,
                offset,
                device,
            });
        }

        Ok(memory_maps)
    }
}

pub fn get_tasks(pid: u32) -> Result<Vec<u32>> {
    let path = format!("/proc/{}/task", pid);
    let entries = std::fs::read_dir(path)?;
    Ok(entries
        .filter_map(|entry| {
            entry
                .ok()
                .map(|e| e.file_name().to_string_lossy().parse().unwrap())
        })
        .collect())
}

pub fn process_exists(pid: u32) -> bool {
    std::fs::exists(format!("/proc/{}", pid)).unwrap_or(false)
}
