#![allow(warnings, unused)]
#![feature(try_blocks)]
#![feature(once_cell)]

mod server;
mod utils;
use crate::server::*;

use std::lazy::SyncLazy;
use std::{thread, str, fs};
use std::path::{Path, PathBuf};
use std::cmp::{min, max};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};
use std::collections::HashMap;

pub const MIME_FILE_PATH: &str = "MIME_types.txt";
pub const BASE_DIR: &str = "/Users/Administrator/Documents/Projects/rust_website/files"; //Keep in linux format
pub const PORT: u32 = 443;

pub fn main() {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    println!("Started with IP: {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream));
            } Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}