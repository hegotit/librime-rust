use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::ops::{Shl, Shr};
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;

use itertools::Itertools;
use log::{info, warn};

use crate::rime::common::PathExt;
use crate::rime::dict::db_utils::{Sink, Source};

// Alias for TSV row representation
pub(crate) type Tsv = Vec<String>;

pub(crate) type TsvParser = Arc<dyn Fn(&[&str]) -> Option<(String, String)> + Send + Sync>;
pub(crate) type TsvFormatter = Arc<dyn Fn(&str, &str) -> Option<Tsv> + Send + Sync>;

pub(crate) struct TsvReader {
    file_path: PathExt,
    parser: TsvParser,
}

impl TsvReader {
    pub(crate) fn new(file_path: PathExt, parser: TsvParser) -> Self {
        Self { file_path, parser }
    }

    // Read TSV file, pass the data to the sink, and return the number of records read
    fn read<S>(&self, sink: Option<&mut S>) -> Result<usize, Box<dyn Error>>
    where
        S: Sink,
    {
        let Some(sink) = sink else {
            return Ok(0);
        };

        info!("Reading tsv file: {}", self.file_path);
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);

        let mut num_entries = 0;
        let mut line_no = 0;
        let mut enable_comment = true;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim_end(); // Trim trailing whitespace
            line_no += 1;

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Skip comments
            if enable_comment && line.starts_with('#') {
                if line.starts_with("#@") {
                    // Metadata
                    let mut split_iter = line[2..].split('\t');
                    match (split_iter.next(), split_iter.next(), split_iter.next()) {
                        (Some(key), Some(value), None) => {
                            if !sink.meta_put(key, value) {
                                warn!(
                                    "Invalid metadata at line {} in file: {}",
                                    line_no, self.file_path
                                );
                            }
                        }
                        _ => warn!(
                            "Invalid metadata at line {} in file: {}",
                            line_no, self.file_path
                        ),
                    }
                } else if line == "# no comment" {
                    // Disable further comments
                    enable_comment = false;
                }
                continue;
            }

            // Read a TSV entry
            let row: Vec<&str> = line.split('\t').collect();
            if let Some((key, value)) = (self.parser)(&row) {
                if !sink.put(&key, &value) {
                    warn!(
                        "Invalid entry at line {} in file: {}",
                        line_no, self.file_path
                    );
                } else {
                    num_entries += 1;
                }
            } else {
                warn!(
                    "Invalid entry at line {} in file: {}",
                    line_no, self.file_path
                );
            }
        }

        Ok(num_entries)
    }
}

pub(crate) struct TsvWriter {
    file_path: PathExt,
    formatter: TsvFormatter,
    pub(crate) file_description: String,
}

impl TsvWriter {
    pub(crate) fn new(file_path: PathExt, formatter: TsvFormatter) -> Self {
        Self {
            file_path,
            formatter,
            file_description: String::new(),
        }
    }

    // Write TSV data from the source, and return the number of records written
    fn write<S>(&self, source: Option<&mut S>) -> Result<usize, Box<dyn Error>>
    where
        S: Source + ?Sized,
    {
        let Some(source) = source else {
            return Ok(0);
        };

        info!("Writing tsv file: {}", self.file_path);
        let mut file = File::create(&self.file_path)?;

        if !self.file_description.is_empty() {
            writeln!(file, "# {}", self.file_description)?;
        }

        if let Some(iter) = source.meta_get() {
            for (key, value) in iter {
                writeln!(file, "#@{}\t{}", key, value)?;
            }
        }

        let mut num_entries = 0;

        if let Some(iter) = source.get() {
            for (key, value) in iter {
                if let Some(row) = (self.formatter)(key, value) {
                    if !row.is_empty() {
                        writeln!(file, "{}", row.iter().format("\t"))?;
                        num_entries += 1;
                    }
                }
            }
        }

        Ok(num_entries)
    }
}

// Overloaded operators for the TSV reader and writer
impl<S> Shr<&mut S> for TsvReader
where
    S: Sink,
{
    type Output = Result<usize, Box<dyn Error>>;

    fn shr(self, sink: &mut S) -> Self::Output {
        self.read(Some(sink))
    }
}

impl<S> Shl<&mut S> for TsvWriter
where
    S: Source + ?Sized,
{
    type Output = Result<usize, Box<dyn Error>>;

    fn shl(self, source: &mut S) -> Self::Output {
        self.write(Some(source))
    }
}
