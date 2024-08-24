use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use crate::error::KVStoreError;

pub struct KVStore {
    data: HashMap<String, String>,
    file: File,
}

impl KVStore {
    pub fn new(filename: &str) -> Result<Self, KVStoreError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)?;
        
        let mut data = HashMap::new();
        let reader = BufReader::new(&file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                data.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
        
        Ok(KVStore { data, file })
    }
    
    pub fn write(&mut self, key: &str, value: &str) -> Result<(), KVStoreError> {
        self.data.insert(key.to_string(), value.to_string());
        writeln!(self.file, "{}:{}", key, value)?;
        self.file.flush()?;
        Ok(())
    }
    
    pub fn read(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
}