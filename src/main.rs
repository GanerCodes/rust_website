#![allow(warnings, unused)]
#![feature(once_cell, lazy_cell)]

mod Rust_AES;
mod sha256;
mod server;
mod config;
mod utils;
use crate::config::*;
use crate::server::*;

use std::mem::drop;
use std::sync::{Arc, Mutex};
use std::{thread, fs};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};

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
                    thread::Builder::new().stack_size(2 * DEFAULT_CHUNK_LENGTH).spawn(
                        move || handle_client(stream, URL_short_clone));
                } Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        drop(listener);
    }
}