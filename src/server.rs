use crate::utils::*;
use crate::*;

use std::lazy::SyncLazy;
use std::{thread, str, fs};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{SeekFrom, BufReader, Read, Write, prelude::*};
use std::collections::HashMap;
use std::time::Duration;

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
    let mut raw_request = Vec::new();
    stream.set_read_timeout(Some(Duration::from_millis(150)));
    stream.read_to_end(&mut raw_request);
    // println!("{}", String::from_utf8_lossy(&mut raw_request));
        
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
                            HeaderName.push_str(&c.to_lowercase().to_string());
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
        stream.write(b"HTTP/1.1 200\nOK\nConnection: Closed\n\nHELP???");
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
    HTTP_Target = decode_url(&HTTP_Target);
    
    // printMap(headers);
    println!("{} {} {}", HTTP_Version, HTTP_Method, HTTP_Target);
    
    /*TODO:
        directory listing
        video/image/file uploader
        chunked file delivery
        https?
        url shorterner
        path grepping file, move settings into json, etc
    */
    
    let mut response_headers = HashMap::<String, String>::new();
    response_headers.insert("Server"      .to_string(), "amogus"   .to_string());
    response_headers.insert("Content-Type".to_string(), "text/html".to_string());
    response_headers.insert("Connection"  .to_string(), "Closed"   .to_string());
    
    loop { //Janky but whatever
        match (&HTTP_Method).as_str() {
            "GET" => {
                let mut pathString = format!("{}{}", &BASE_DIR, &HTTP_Target);
                pathString = formatPath(&pathString);
                if !DTAsafe(&pathString, &BASE_DIR) {
                    make_response(&stream, &Response {
                        code: 200,
                        code_name: "OK",
                        headers: response_headers
                    });
                    stream.write(b"<script>location.href = 'https://youtu.be/dQw4w9WgXcQ';</script>");
                    break;
                }
                
                let mut filePath = PathBuf::from(&pathString);
                match filePath.canonicalize() {
                    Ok(filePath) => {
                        if filePath.is_file() {
                            let mut MIME = get_MIME_from_filename(&pathString);
                            response_headers.insert("Content-Type".to_string(), MIME.clone());
                            response_headers.insert("Accept-Ranges".to_string(), "bytes".to_string());
                            
                            let fileMetadata = fs::metadata(&filePath).unwrap();
                            let fileSize = fileMetadata.len();
                            let fileSizeString = fileSize.to_string();
                            let rangeKey = "range".to_string();
                            if headers.contains_key(&rangeKey) {
                                let contentRange = parseRangeHeader(&headers.get(&rangeKey).unwrap(), fileSize);
                                let rangeHeader = format!("bytes {}-{}/{}", contentRange.0, contentRange.1, fileSize);
                                let byteReadCount = (contentRange.1 - contentRange.0) + 1;
                                response_headers.insert("Content-Range".to_string(), rangeHeader);
                                response_headers.insert("Content-Length".to_string(), byteReadCount.to_string());
                                
                                let mut videoFile = File::open(&filePath).unwrap();
                                let mut videoFileBuffer = vec![0; byteReadCount as usize];
                                videoFile.seek(SeekFrom::Start(contentRange.0)).unwrap();
                                videoFile.read(&mut videoFileBuffer).unwrap();
                                make_response(&stream, &Response {
                                    code: 206,
                                    code_name: "Partial Content",
                                    headers: response_headers
                                });
                                stream.write(&videoFileBuffer);
                                break;
                            }else{
                                response_headers.insert("Content-Range".to_string(), format!("bytes {}-{}/{}", 0, fileSize - 1, fileSize));
                                response_headers.insert("Content-Length".to_string(), fileSizeString);
                                write_file(&stream, &Response {
                                    code: 200,
                                    code_name: "OK",
                                    headers: response_headers },
                                    &filePath
                                );
                            }
                            break;
                        }else{
                            let mut indexPath = filePath.clone();
                            indexPath.push("index.html");
                            if indexPath.exists() {
                                write_file(&stream, &Response {
                                    code: 200,
                                    code_name: "OK",
                                    headers: response_headers
                                }, &indexPath);
                            } else {
                                make_response(&stream, &Response{
                                    code: 418,
                                    code_name: "WIP",
                                    headers: response_headers
                                });
                                stream.write(b"dir");
                            }
                        }
                    },
                    Err(_) => {
                        make_response(&stream, &Response {
                            code: 404,
                            code_name: "Not Found",
                            headers: response_headers
                        });
                        stream.write(b"404");
                    }
                }
            },
            _ => ()
        }
        break;
    }
}