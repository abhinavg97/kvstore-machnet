use std::sync::{Arc, Mutex};
use std::thread;
use log::{error, info, warn};
use machnet::{machnet_attach, machnet_listen, machnet_recv, machnet_send, MachnetChannel, MachnetFlow};
use ctrlc;
use kv_store::{KVStore, Request, Response, KVStoreError};

const MAX_MSG_SIZE: usize = 1024;

fn handle_client(channel: &mut MachnetChannel, store: Arc<Mutex<KVStore>>, flow: MachnetFlow) {
    let mut buffer = vec![0u8; MAX_MSG_SIZE];
    
    loop {
        let size = machnet_recv(channel, &mut buffer, MAX_MSG_SIZE as u64, &mut MachnetFlow::default());
        if size <= 0 {
            warn!("Client disconnected");
            break;
        }
        
        let request: Request = match serde_json::from_slice(&buffer[..size as usize]) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to deserialize request: {}", e);
                continue;
            }
        };
        
        let response = match request {
            Request::Write { key, value } => {
                match store.lock().unwrap().write(&key, &value) {
                    Ok(_) => {
                        info!("Write successful: {} -> {}", key, value);
                        Response::Ok
                    },
                    Err(e) => {
                        error!("Write failed: {}", e);
                        Response::Error(e.to_string())
                    }
                }
            }
            Request::Read { key } => {
                match store.lock().unwrap().read(&key) {
                    Some(value) => {
                        info!("Read successful: {} -> {}", key, value);
                        Response::Value(value)
                    },
                    None => {
                        warn!("Key not found: {}", key);
                        Response::NotFound
                    }
                }
            }
        };
        
        let response_bytes = match serde_json::to_vec(&response) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to serialize response: {}", e);
                continue;
            }
        };
        
        if machnet_send(channel, flow, &response_bytes, response_bytes.len() as u64) < 0 {
            error!("Failed to send response");
            break;
        }
    }
}

fn main() -> Result<(), KVStoreError> {
    env_logger::init();
    
    let store = Arc::new(Mutex::new(KVStore::new("kv_store.txt")?));
    let mut channel = machnet_attach().ok_or_else(|| KVStoreError::MachnetError("Failed to attach machnet".to_string()))?;
    
    // let channel_result = machnet_attach();
    // if channel_result == -1 {
    //     return Err(KVStoreError::MachnetError("Failed to attach machnet".to_string()));
    // }
    // let mut channel = channel_result; // Assuming it's not an Option or Result.
    

    // machnet_listen(&mut channel, "127.0.0.1", 8080).ok_or_else(|e| KVStoreError::MachnetError(e.to_string()))?;

    if machnet_listen(&mut channel, "127.0.0.1", 8080) == -1 {
        return Err(KVStoreError::MachnetError("Failed to listen on machnet".to_string()));
    }

    info!("Server listening on 127.0.0.1:8080");
    
    let running = Arc::new(Mutex::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        info!("Shutting down server...");
        *r.lock().unwrap() = false;
    }).map_err(|e| KVStoreError::MachnetError(e.to_string()))?;
    
    while *running.lock().unwrap() {
        let mut flow = MachnetFlow::default();
        let size = machnet_recv(&mut channel, &mut vec![0u8; 1], 1, &mut flow);
        if size > 0 {
            let store_clone = Arc::clone(&store);
            let channel_ref = &mut channel;
            // let mut new_channel = machnet_attach().expect("Failed to attach new machnet channel"); // Create a new channel
            thread::spawn(move || {
                handle_client(channel_ref, store_clone, flow);
                // handle_client(&mut new_channel, store_clone, flow);
            });
        }
    }
    
    info!("Server shut down successfully");
    Ok(())
}