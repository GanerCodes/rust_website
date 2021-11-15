#![allow(warnings, unused)]
#![feature(try_blocks)]
#![feature(once_cell)]

mod Rust_AES;
mod sha256;
mod server;
mod utils;
use crate::server::*;

use std::mem::drop;
use std::cmp::{min, max};
use std::lazy::SyncLazy;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::{thread, str, fs};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};

pub const AES_KEY: [u8; 16] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10];
pub const MIME_FILE_PATH: &str = "MIME_types.txt";
pub const BASE_DIR: &str = "/Users/Administrator/Documents/Projects/rust_website/files"; //Keep in linux format
pub const UPLOAD_FILE_PATH: &str = "/e/upload";
pub const PORT: u32 = 234;
pub const ENCRYPTED_PATH_PREFIX: &str = "/e/";
pub const SHORTHAND_PATH_PREFIX: &str = "/s/";

pub fn main() {
    let mut URL_shorts: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    
    drop(URL_shorts.lock().unwrap().insert("test".to_string(), "cheese.org".to_string()));
    
    loop {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
        println!("Started with IP: {}", listener.local_addr().unwrap());
        for stream in listener.incoming() {
            let mut URL_short_clone = URL_shorts.clone();
            match stream {
                Ok(stream) => {
                    thread::spawn(move || handle_client(stream, URL_short_clone));
                } Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        drop(listener);
    }
}