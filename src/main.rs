#![allow(warnings, unused)]
#![feature(once_cell)]

mod Rust_AES;
mod sha256;
mod server;
mod utils;
use crate::server::*;

use std::mem::drop;
use std::sync::{Arc, Mutex};
use std::{thread, str, fs};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};

pub const DOMAIN_NAME: &str = "localhost:234";
pub const PREFERRED_PROTOCOL: &str = "http";
pub const AES_KEY: [u8; 16] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10];
pub const BASE_DIR: &str = "/Users/Administrator/Documents/Projects/rust_website/files";
pub const SHORTHAND_FILE_PATH: &str = "URL_shortens.kvm";
pub const CODES_FILE_PATH: &str = "response_codes.txt";
pub const MIME_FILE_PATH: &str = "MIME_types.txt";
pub const UPLOAD_FILE_PATH: &str = "/e/upload";
pub const ENCRYPTED_PATH_PREFIX: &str = "/e/";
pub const SHORTHAND_PATH_PREFIX: &str = "/s/";
pub const WEBSITE_PREFIXES: [&str; 2] = ["http://localhost:234/", "https://localhost:234/"];
pub const IP_ADDR: &str = "0.0.0.0";
pub const PORT: u32 = 234;

pub fn main() {
    let mut URL_shorts: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    
    let mut shorts = URL_shorts.lock().unwrap();
    let mut URL_shorts_file = fs::File::open(&SHORTHAND_FILE_PATH).unwrap();
    for line in BufReader::new(URL_shorts_file).lines() {
        let mut lineRaw = line.unwrap();
        if lineRaw.contains(':') {
            let mut lineSplit = lineRaw.splitn(2, ":");
            shorts.insert(lineSplit.next().unwrap().to_string(), lineSplit.next().unwrap().to_string());
        }
    }
    drop(shorts);
    
    loop {
        let listener = TcpListener::bind(format!("{}:{}", IP_ADDR, PORT)).unwrap();
        println!("Started on IP: {}", listener.local_addr().unwrap());
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