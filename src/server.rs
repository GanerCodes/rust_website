use crate::utils::*;
use crate::*;

use std::lazy::SyncLazy;
use std::{thread, str, fs};
use std::path::{Path, PathBuf};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};
use std::collections::HashMap;

static Body_delim_pattern: [u8; 4] = [13, 10, 13, 10];

pub static MIME_types: SyncLazy<HashMap::<String, String>> = SyncLazy::new(|| {
    let mut MIMEs = HashMap::<String, String>::new();
    let mut MIME_types_file = fs::File::open(&MIME_FILE_PATH).unwrap();
    let mut MIME_types_reader = BufReader::new(MIME_types_file);
    for line in MIME_types_reader.lines() {
        let mut lineRaw = line.unwrap();
        let mut lineSplit = lineRaw.splitn(2, ", ");
        MIMEs.insert(lineSplit.next().unwrap().to_string(), lineSplit.next().unwrap().to_string());
    }
    MIMEs
});

pub fn handle_client(mut stream: TcpStream) {
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
    
    decode_url(&HTTP_Target);
    
    /*TODO:
        escape sequences
        playbackable video/audio
        chunked file delivery
        video/image/file uploader
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
                            let mut MIME = get_MIME_from_filename(&pathString);
                            response_headers.insert("Content-Type", MIME.as_str());
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