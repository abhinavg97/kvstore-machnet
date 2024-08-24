use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use log::{error, info};
use machnet::{machnet_attach, machnet_connect, machnet_send, machnet_recv, MachnetChannel, MachnetFlow};
use ctrlc;
use std::sync::{Arc, Mutex};
use kv_store::{Request, Response, KVStoreError};

const MAX_MSG_SIZE: usize = 1024;

fn generate_random_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn send_request(channel: &mut MachnetChannel, flow: MachnetFlow, request: &Request) -> Result<Response, KVStoreError> {
    let request_bytes = serde_json::to_vec(request)?;
    
    if machnet_send(channel, flow, &request_bytes, request_bytes.len() as u64) < 0 {
        return Err(KVStoreError::MachnetError("Failed to send request".into()));
    }
    
    let mut buffer = vec![0u8; MAX_MSG_SIZE];
    let size = machnet_recv(channel, &mut buffer, MAX_MSG_SIZE as u64, &mut MachnetFlow::default());
    
    if size <= 0 {
        return Err(KVStoreError::MachnetError("Failed to receive response".into()));
    }
    
    let response: Response = serde_json::from_slice(&buffer[..size as usize])?;
    Ok(response)
}

fn main() -> Result<(), KVStoreError> {
    env_logger::init();
    
    let mut channel = machnet_attach().map_err(|e| KVStoreError::MachnetError(e.to_string()))?;
    let flow = machnet_connect(&channel, "127.0.0.1", "127.0.0.1", 8080)
        .map_err(|e| KVStoreError::MachnetError(e.to_string()))?;
    
    let mut keys = HashSet::new();
    
    let running = Arc::new(Mutex::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        info!("Shutting down client...");
        *r.lock().unwrap() = false;
    }).map_err(|e| KVStoreError::MachnetError(e.to_string()))?;
    
    while *running.lock().unwrap() {
        if thread_rng().gen_bool(0.7) {
            // Write operation
            let key = generate_random_string(8);
            let value = generate_random_string(16);
            let request = Request::Write { key: key.clone(), value };
            
            match send_request(&mut channel, flow, &request) {
                Ok(Response::Ok) => {
                    info!("Write successful: {}", key);
                    keys.insert(key);
                },
                Ok(Response::Error(e)) => error!("Write failed: {}", e),
                Err(e) => error!("Request failed: {}", e),
                _ => error!("Unexpected response for write operation"),
            }
        } else {
            // Read operation
            if let Some(key) = keys.iter().next().cloned() {
                let request = Request::Read { key: key.clone() };
                
                match send_request(&mut channel, flow, &request) {
                    Ok(Response::Value(value)) => info!("Read successful: {} -> {}", key, value),
                    Ok(Response::NotFound) => info!("Key not found: {}", key),
                    Ok(Response::Error(e)) => error!("Read failed: {}", e),
                    Err(e) => error!("Request failed: {}", e),
                    _ => error!("Unexpected response for read operation"),
                }
            }
        }
        
        thread::sleep(Duration::from_secs(1));
    }
    
    info!("Client shut down successfully");
    Ok(())
}