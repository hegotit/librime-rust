use std::fs::File;
use std::io::Read;

use crc32fast::Hasher;

use crate::rime::common::PathExt;

pub(crate) fn compare_version_string(x: &str, y: &str) -> i32 {
    let mut i = 0;
    let mut j = 0;
    let m = x.len();
    let n = y.len();
    let x_bytes = x.as_bytes();
    let y_bytes = y.as_bytes();

    while i < m || j < n {
        let mut v1 = 0;
        let mut v2 = 0;

        while i < m && x_bytes[i] != b'.' {
            v1 = v1 * 10 + (x_bytes[i] - b'0') as i32;
            i += 1;
        }
        i += 1;
        while j < n && y_bytes[j] != b'.' {
            v2 = v2 * 10 + (y_bytes[j] - b'0') as i32;
            j += 1;
        }
        j += 1;

        if v1 > v2 {
            return 1;
        }
        if v1 < v2 {
            return -1;
        }
    }
    0
}

pub(crate) struct ChecksumComputer {
    hasher: Hasher,
}

impl ChecksumComputer {
    pub(crate) fn new(initial_remainder: u32) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(&initial_remainder.to_le_bytes());
        Self { hasher }
    }

    pub(crate) fn process_file(&mut self, file_path: &PathExt) -> std::io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        self.hasher.update(&buffer);
        Ok(())
    }

    pub(crate) fn checksum(&self) -> u32 {
        self.hasher.clone().finalize()
    }
}

pub(crate) fn checksum(file_path: &PathExt) -> std::io::Result<u32> {
    let mut computer = ChecksumComputer::new(0);
    computer.process_file(file_path)?;
    Ok(computer.checksum())
}
