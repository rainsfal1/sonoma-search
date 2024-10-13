// schema

use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};

pub struct IndexStorage {
    pub index_path: PathBuf,
}

impl IndexStorage {
    // Create a new instance of IndexStorage with the specified path
    pub fn new(index_path: &str) -> Self {
        IndexStorage {
            index_path: PathBuf::from(index_path),
        }
    }

    // Save the index data to a file
    pub fn save_index(&self, index_name: &str, data: &[u8]) -> io::Result<()> {
        let file_path = self.index_path.join(index_name);
        let mut file = File::create(file_path)?;
        file.write_all(data)?;
        Ok(())
    }

    // Load the index data from a file
    pub fn load_index(&self, index_name: &str) -> io::Result<Vec<u8>> {
        let file_path = self.index_path.join(index_name);
        let mut file = File::open(file_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    // Delete an index
    pub fn delete_index(&self, index_name: &str) -> io::Result<()> {
        let file_path = self.index_path.join(index_name);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    // List all stored indexes
    pub fn list_indexes(&self) -> io::Result<Vec<String>> {
        let mut indexes = Vec::new();
        for entry in fs::read_dir(&self.index_path)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap_or_default();
            indexes.push(file_name);
        }
        Ok(indexes)
    }
}
