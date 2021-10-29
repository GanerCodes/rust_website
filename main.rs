#![allow(warnings, unused)]

#[macro_use]
extern crate once_cell;

use std::{thread, str, fs};
use std::path::{Path, PathBuf};
use std::cmp::{min, max};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};
use std::collections::HashMap;
use once_cell::sync::Lazy;

static MIME_FILE_PATH: &str = "MIME_types.txt";
static BASE_DIR: &str = "/Users/Administrator/Documents/Projects/rust_website/files"; //Keep in linux format
static PORT: u32 = 443;

static Body_delim_pattern: [u8; 4] = [13, 10, 13, 10];
static MIME_types: Lazy<HashMap::<String, String>> = sync_lazy! {
    let mut MIMEs = HashMap::<String, String>::new();
    let mut MIME_types_file = fs::File::open(&MIME_FILE_PATH).unwrap();
    let mut MIME_types_reader = BufReader::new(MIME_types_file);
    for line in MIME_types_reader.lines() {
        println!("{}", line.unwrap());
    }
    MIMEs;
};

fn printMap(mut map: HashMap::<String, String>) {
    for (key, value) in map.into_iter() {
        println!("{:<32} :\t{}", key, value);
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

fn write_file(mut stream: &TcpStream, mut filePath: &PathBuf, mut headers: &HashMap::<&str, &str>) {
    match fs::read(&filePath) {
        Ok(content) => {
            make_response(&stream, 200, "OK", &headers);
            stream.write(&content);
        }, Err(e) => {
            make_response(&stream, 418, "I'm a teapot", &headers);
            stream.write(b"I'm a teapot :>");
        }
    }
}

fn formatPath(mut path: &str) -> String {
    let mut result  = String::new();
    let mut segment = String::new();
    let mut dotCount: i32 = 0;
    let mut ignCount: i32 = 0;
    let mut prevChar = '/';
    
    for mut c in path.chars().rev() {
        if c == '\\' {
            c = '/';
        }
        match c {
            '.' => {
                if dotCount > -1 {
                    dotCount += 1;
                } else {
                    segment.push('.');
                }
            },
            '/' => {
                if prevChar == '/' {
                    continue;
                }
                if dotCount != -1 {
                    ignCount += min(dotCount, 2);
                }else if dotCount == -1 && ignCount == 0 {
                    segment.push('/');
                    result.push_str(segment.as_str());
                }
                segment = String::new();
                if ignCount > 0 { 
                    ignCount -= 1
                }
                dotCount = 0;
            },
            _ => {
                segment.push(c);
                dotCount = -1;
            }
        }
        prevChar = c;
    }
    if result.len() == 0 {
        return String::from("/");
    }
    return result.chars().rev().collect();
}

fn DTAsafe(mut path: &str, mut base: &str) -> bool {
    if path.len() < base.len() {
        return false;
    }
    let mut pathLower = path.to_lowercase();
    let mut baseLower = path.to_lowercase();
    let mut pathChars = pathLower.chars();
    let mut baseChars = baseLower.chars();
    for i in 0..base.len() {
        if pathChars.next() != baseChars.next() {
            return false;
        }
    }
    return true;
}

fn handle_client(mut stream: TcpStream) {
    stream.set_nonblocking(true);
    let mut raw_request = Vec::new();
    stream.read_to_end(&mut raw_request);
        
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
    
    // println!("{}", String::from_utf8_lossy(&mut raw_request));
    // printMap(headers);
    println!("{} {} {}", HTTP_Version, HTTP_Method, HTTP_Target);
    
    /*TODO:
        escape sequences
        mime types
        playbackable video/audio
        chunked file delivery
        url shorterner
        path grepping file, move settings into json, etc
    */
    
    let mut response_headers = HashMap::<&str, &str>::new();
    response_headers.insert("Server", "amogus");
    response_headers.insert("Content-Type", "text/html");
    response_headers.insert("Connection", "Closed");
    match (&HTTP_Method).as_str() {
        "GET" => {
            let mut pathString = format!("{}{}", &BASE_DIR, &HTTP_Target);
            pathString = formatPath(&pathString);
            if DTAsafe(&pathString, &BASE_DIR) {
                let mut filePath = PathBuf::from(&pathString);
                
                match filePath.canonicalize() {
                    Ok(filePath) => {
                        if filePath.is_file() {
                            write_file(&stream, &filePath, &response_headers);
                        }else{
                            let mut indexPath = filePath.clone();
                            indexPath.push("index.html");
                            if indexPath.exists() {
                                write_file(&stream, &indexPath, &response_headers);
                            } else {
                                make_response(&stream, 418, "WIP", &response_headers);
                                stream.write(b"dir");
                            }
                        }
                    },
                    Err(_) => {
                        make_response(&stream, 404, "Not Found", &response_headers);
                        stream.write(b"404");
                    }
                }
            }else{
                make_response(&stream, 200, "OK", &response_headers);
                stream.write(b"<script>location.href = 'https://youtu.be/dQw4w9WgXcQ';</script>");
            }
        },
        _ => ()
    }
}

fn main() {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
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