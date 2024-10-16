use memmap2::{MmapMut, MmapOptions};
use std::fs::File;
use std::io::{Error, ErrorKind, Result};

use crate::rime::common::PathExt;

pub struct MappedFile {
    file_path: PathExt,
    file: File,
    mmap: Option<MmapMut>,
    size: usize,
}

impl MappedFile {
    pub(crate) fn new(file_path: PathExt) -> Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;

        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        Ok(Self {
            file_path,
            file,
            mmap: Some(mmap),
            size: 0,
        })
    }

    pub(crate) fn file_path(&self) -> &PathExt {
        &self.file_path
    }

    fn size(&self) -> usize {
        self.size
    }

    fn read(&self, offset: usize, length: usize) -> Option<&[u8]> {
        match &self.mmap {
            Some(mmap) if offset + length <= self.size => Some(&mmap[offset..offset + length]),
            _ => None,
        }
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        let size = data.len();
        match &mut self.mmap {
            Some(mmap) /*if offset + size <= self.size*/ => {
                mmap/*[offset..offset + data.len()]*/.copy_from_slice(data);
                mmap.flush()?;
                Ok(())
            }
            // Some(_) => Err(Error::new(ErrorKind::InvalidInput, "Write out of bounds")),
            None => Err(Error::new(
                ErrorKind::Other,
                "Memory map is not initialized",
            )),
        }
    }
}

#[test]
fn test() -> Result<()> {
    let file_path = PathExt::new("example.txt");
    // 创建一个映射文件
    let mut mapped_file = MappedFile::new(file_path)?;

    // 向文件中写入数据
    let data = b"Hello, Rust!";
    mapped_file.write(0, data)?;

    // 读取文件中的数据
    if let Some(contents) = mapped_file.read(0, data.len()) {
        println!("Read from file: {:?}", std::str::from_utf8(contents));
    }

    Ok(())
}
