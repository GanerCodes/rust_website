#![allow(warnings, unused)]

use std::thread;
use std::path::{Path, PathBuf};
use std::str;
use std::fs;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::collections::HashMap;

static BASE_DIR: &str = "/UNSORTED/rust_website/files"; //Keep in linux format
static PORT: u32 = 443;
static Body_delim_pattern: [u8; 4] = [13, 10, 13, 10];

fn printMap(mut map: HashMap::<String, String>) {
    for (key, value) in map.into_iter() {
        println!("{:<25} :\t{}", key, value);
    }
}

fn make_response(mut stream: &TcpStream, mut response_code: i32, mut response_str: &str, mut headers: &HashMap::<&str, &str>) {
    let mut response = String::from("HTTP/1.1 ");
    response.push_str(&response_code.to_string());
    response.push('\n');
    response.push_str(&response_str);
    response.push('\n');
    for (header, value) in headers.into_iter() {
        response.push_str(&header);
        response.push_str(": ");
        response.push_str(&value);
        response.push('\n');
    }
    response.push('\n');
    stream.write(response.as_bytes());
}

fn formatPath(mut path: &str) {
    for c in path.chars() {
        if c == '\\' {
            c = '/';
        }
        
    }
}

fn handle_client(mut stream: TcpStream) {
    stream.set_nonblocking(true);
    let mut raw_request = Vec::new();
    stream.read_to_end(&mut raw_request);
    
    println!("{}", String::from_utf8_lossy(&mut raw_request));
    
    let mut headers = HashMap::<String, String>::new();
    let mut HTTP_Identifier = String::from("");
    let mut HTTP_Method     = String::from("");
    let mut HTTP_Target     = String::from("");
    let mut HTTP_Version    = String::from("");
    let mut HeaderName      = String::from("");
    let mut HeaderValue     = String::from("");
    let mut bodyStartIndex: i8 = -1; //Doesn't do anything yet
    
    let mut bodyDelimIndex = 0;
    let mut editMode = 0;
    for i in 0..(&raw_request).len() {
        let c = char::from((&raw_request)[i]);
        if (&raw_request)[i] == Body_delim_pattern[bodyDelimIndex] {
            if bodyDelimIndex == 2 {
                bodyStartIndex = i as i8;
                break;
            }else{
                bodyDelimIndex += 1;
            }
        }else{
            bodyDelimIndex = 0;
        }
        
        match c {
            '\n' => {
                match editMode {
                    0 if HTTP_Identifier.len() > 0 => {
                        editMode = 1;
                    }, 2 => {
                        headers.insert(HeaderName.clone(), HeaderValue.clone());
                        HeaderName  = String::from("");
                        HeaderValue = String::from("");
                        editMode = 1;
                    }, _ => ()
                }
            },
            '\r' => (),
            _ => {
                match editMode {
                    0 => HTTP_Identifier.push(c),
                    1 => {
                        if c == ':' {
                            editMode = 2;
                        }else{
                            HeaderName.push(c);
                        }
                    },
                    2 => HeaderValue.push(c),
                    _ => ()
                }
            }
        }
    }
    if HTTP_Identifier.len() == 0 {
        println!("Could not read data.");
        return;
    }
    
    editMode = 0;
    for c in HTTP_Identifier.chars() {
        if c == ' ' && editMode < 2 {
            editMode += 1
        }else{
            match editMode {
                0 => HTTP_Method .push(c),
                1 => HTTP_Target .push(c),
                2 => HTTP_Version.push(c),
                _ => ()
            }
        }
    }
    
    // println!("{} {} {}", HTTP_Version, HTTP_Method, HTTP_Target);
    // printMap(headers);
    
    //HERE: perform path grepping and whatnot
    //TODO: mime types and path verification
    
    let mut response_headers = HashMap::<&str, &str>::new();
    response_headers.insert("Server", "amogus");
    response_headers.insert("Content-Type", "text/html");
    response_headers.insert("Connection", "Closed");
    match (&HTTP_Method).as_str() {
        "GET" => {
            let mut path = PathBuf::from(format!("{}{}", &BASE_DIR, &HTTP_Target));
            dbg!(&path);
            // match path.canonicalize() {
            //     Ok(tempPath) => {
            //         path = tempPath.as_path();
                    
            //     },
            //     Err(e) => {
            //         println!("FSDOHUI:JNKL");
            //     }
            // }
            // if path.exists() {
            //     let tempPath = .unwrap();
            //     println!("Path {}", path.clone().to_str().unwrap());
            //     if path.starts_with(&BASE_DIR) {
            //         if(path.is_file()) {
            //             match fs::read(path) {
            //                 Ok(content) => {
            //                     make_response(&stream, 200, "OK", &response_headers);
            //                     stream.write(&content);
            //                 }, Err(e) => {
            //                     make_response(&stream, 418, "I'm a teapot", &response_headers);
            //                     stream.write(b"I'm a teapot :>");
            //                 }
            //             }
            //         }else{
            //             make_response(&stream, 418, "WIP", &response_headers);
            //             stream.write(b"dir");
            //         }
            //     }else{
            //         make_response(&stream, 200, "OK", &response_headers);
            //         stream.write(b"<script>location.href = 'https://youtu.be/dQw4w9WgXcQ';</script>");
            //     }
            // }else{
            //     make_response(&stream, 404, "Not Found", &response_headers);
            //     stream.write(b"404");
            // }
        },
        _ => ()
    }
}


fn main() {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    println!("Server listening on {}", PORT);
    
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