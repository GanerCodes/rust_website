use crate::Rust_AES::encrypt::AES_Encrypt;
use crate::server::{MIME_types, response_codes};

use std::lazy::SyncLazy;
use std::{fmt, thread, str, fs};
use std::path::{Path, PathBuf};
use std::cmp::{min, max};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};
use std::collections::HashMap;
use std::fmt::Write as fmtWrite;

pub fn printMap(mut map: HashMap::<String, String>) {
    for (key, value) in map.into_iter() {
        println!("{:<32} :\t{}", key, value);
    }
}

pub struct Response<> {
    pub code: u16,
    pub headers: HashMap<String, String>
}

pub fn make_response(mut stream: &TcpStream, response: &Response) {
    let mut request = String::from("HTTP/1.1 ");
    write!(request, "{} {}\r\n", &response.code, response_codes.get(&response.code).unwrap());
    for (header, value) in response.headers.iter() {
        request.push_str(&header);
        request.push_str(": ");
        request.push_str(&value);
        request.push_str("\r\n");
    }
    request.push_str("\r\n");
    stream.write(request.as_bytes());
}

pub fn write_file(mut stream: &TcpStream, response: &Response, mut filePath: &PathBuf) {
    match fs::read(&filePath) {
        Ok(content) => {
            make_response(&stream, &response);
            stream.write(&content);
        }, Err(e) => {
            make_response(&stream, &Response {
                code: 418,
                headers: response.headers.clone()
            });
            stream.write(b"I'm a teapot :>");
        }
    }
}

pub fn send_redirect(mut stream: &TcpStream, mut newLoc: &String) {
    stream.write(format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {}\r\n\r\n", newLoc).as_bytes());
}

pub fn formatPath(mut path: &str) -> String {
    let mut result  = String::new();
    let mut segment = String::new();
    let mut dotCount: i32 = 0;
    let mut ignCount: i32 = 0;
    let mut prevChar = '/';
    let mut c = ' ';
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

pub fn DTAsafe(mut path: &str, mut base: &str) -> bool {
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

pub fn byte_hex_to_u8(mut c: u8) -> u8 { //assumed safe input
    return if c < 58 {c - 48} else {c - 55};
}

pub fn decode_url(mut url: &str) -> String { //Redo this at some point more efficently
    let mut result = String::new();
    let mut char_buf = [0u8; 4];
    
    let mut maxNibbles = -1i8;
    let mut nibbleCount = 0;
    for mut c in url.chars() {
        if c == '%' {
            if maxNibbles == -1 {
                nibbleCount = -1;
            }
        }else if nibbleCount == -1 {
            char_buf[0] = byte_hex_to_u8(c as u8) << 4;
            if char_buf[0] >> 7 == 0 {
                maxNibbles = 1;
            }else if char_buf[0] >> 5 == 6 {
                maxNibbles = 3;
            }else if char_buf[0] == 224 {
                maxNibbles = 5;
            }else{
                maxNibbles = 7;
            }
            nibbleCount = 0;
        }else if maxNibbles > 0 {
            nibbleCount += 1;
            let nibble = byte_hex_to_u8(c as u8);
            if nibbleCount % 2 == 0 {
                char_buf[(nibbleCount >> 1) as usize] = nibble << 4;
            }else{
                char_buf[(nibbleCount >> 1) as usize] |= nibble;
            }
            if nibbleCount == maxNibbles {
                result.push_str(&String::from_utf8_lossy(&char_buf[..(1 + maxNibbles >> 1) as usize]));
                maxNibbles = -1;
            }
        }else{
            result.push(c);
        }
    }
    return result;
}

pub fn get_extension(mut path: &str) -> String {
    let mut result  = String::new();
    for mut c in path.chars().rev() {
        match c {
            '/' => {
                break;
            }, '.' => {
                result.push('.');
                break;
            }, h => {
                result.push(h);
            }
        }
    }
    return result.chars().rev().collect();
}

pub fn get_MIME_from_filename(mut path: &str) -> String {
    dbg!(&path, &get_extension(path));
    match MIME_types.get(&get_extension(path)) {
        Some(val) => {
            return (&val).to_string();
        },
        None => {
            return "text".to_string();
        }
    }
}

//I'm really considering using regex and the such at this point, this is so many lines for such a stupid thing
pub fn parseRangeHeader(header: &String, fileSize: u64) -> (u64, u64) {
    let mut sStr = "".to_string();
    let mut eStr = "".to_string();
    
    let mut state = 0;
    for mut c in header.chars() {
        match state {
            0 => { if c == '=' { state = 1; } },
            1 => { if c == '-' { state = 2; } else { sStr.push(c); } },
            _ => { eStr.push(c); }
        }
    }
    
    let sNum = sStr.parse::<u64>().unwrap_or(0);
    let eNum = eStr.parse::<u64>().unwrap_or(fileSize - 1);
    return (sNum, eNum);
}

pub fn splitMIME(mime: &str) -> (&str, &str) {
    let mut spl = mime.splitn(2, '/');
    (spl.next().unwrap(), spl.next().unwrap())
}

pub fn hexToBytes(mut digits: Vec<u8>) -> Vec<u8> {
    digits.chunks(2)
        .map(|c| String::from_utf8_lossy(c))
        .map(|s| u8::from_str_radix(&s, 16))
        .map_while(|r| r.ok())
        .collect()
}

pub fn bytesToHex(mut digits: Vec<u8>) -> String {
    let mut result = String::new();
    for digit in digits {
        result.push_str(format!("{:02x}", digit).as_str());
    }
    result
}

pub fn respond(mut stream: &TcpStream, response_headers: HashMap<String, String>, code: u16) {
    make_response(&stream, &Response {
        code: code,
        headers: response_headers
    });
}

pub fn respondCodeText(mut stream: &TcpStream, response_headers: HashMap<String, String>, code: u16) {
    make_response(&stream, &Response {
        code: code,
        headers: response_headers
    });
    stream.write(code.to_string().as_bytes());
}

pub fn encrypt_fileName(mut fileName: &String, key: [u8; 16]) -> String {
    bytesToHex(AES_Encrypt(fileName.as_bytes(), key).to_vec())
}

pub fn hashmapFromDelims(input: &String, del: char, sep: char) -> HashMap::<String, String> {
    let mut map = HashMap::<String, String>::new();
    let mut flagName  = String::from("");
    let mut flagValue = String::from("");
    let mut valueEdit = false;
    for c in input.chars() {
        match c {
            c if c == del && !valueEdit => { valueEdit = true; },
            c if c == sep => {
                map.insert(flagName.clone().trim().to_string(), flagValue.clone().trim().to_string());
                flagName  = "".to_string();
                flagValue = "".to_string();
                valueEdit = false;
            }, _ => {
                if valueEdit == false {
                    flagName.push(c);
                }else{
                    flagValue.push(c);
                }
            }
        }
    }
    if flagName.len() > 0 {
        map.insert(flagName.clone().trim().to_string(), flagValue.clone().trim().to_string());
    }
    map
}