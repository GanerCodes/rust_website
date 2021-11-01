use crate::server::{MIME_types};

use std::lazy::SyncLazy;
use std::{thread, str, fs};
use std::path::{Path, PathBuf};
use std::cmp::{min, max};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};
use std::collections::HashMap;

pub fn printMap(mut map: HashMap::<String, String>) {
    for (key, value) in map.into_iter() {
        println!("{:<32} :\t{}", key, value);
    }
}

pub fn make_response(mut stream: &TcpStream, mut response_code: i32, mut response_str: &str, mut headers: &HashMap::<&str, &str>) {
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

pub fn write_file(mut stream: &TcpStream, mut filePath: &PathBuf, mut headers: &HashMap::<&str, &str>) {
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

pub fn formatPath(mut path: &str) -> String {
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

pub fn byte_hex_to_u8(mut c: u8) -> u8 {
    if c < 58 {
        return c - 48;
    }
    return c - 55;
}

pub fn decode_url(mut url: &str) -> String {
    let mut result = String::new();
    let mut char_buf: [u8; 4] =[0, 0, 0, 0];
    let mut expectedDigitCount = 0 as u8;
    let mut check = 0;
    for mut c in url.chars() {
        // dbg!(&c, &expectedDigitCount, &check);
        let mut append = false;
        if c == '%' {
            if expectedDigitCount == 0 {
                check = 1;
            }
            continue;
        }
        if expectedDigitCount == 0 {
            if check == 1 || check == 2 {
                let hex_digit = byte_hex_to_u8(c as u8);
                if check == 1 {
                    char_buf[0] = hex_digit * 16;
                    dbg!(&c, &hex_digit, &char_buf[0]);
                }else{
                    char_buf[0] += hex_digit;
                    dbg!(&char_buf[0]);
                    if char_buf[0] < 128 {
                        append = true;
                    }else if char_buf[0] >> 5 == 6 {
                        expectedDigitCount = 2;
                    }else if char_buf[0] >> 4 == 14 {
                        expectedDigitCount = 4;
                    }else if char_buf[0] >> 3 == 30 { 
                        expectedDigitCount = 6;
                    }
                }
                check += 1;
            }
        }else{
            let hex_digit = byte_hex_to_u8(c as u8);
            dbg!(&c);
            char_buf[(expectedDigitCount / 2) as usize] += hex_digit * (15 * (1 - expectedDigitCount % 2) + 1);
            expectedDigitCount -= 1;
            if expectedDigitCount == 0 {
                append = true;
            }
        }
        if append {
            dbg!(&char_buf);
            result.push_str(str::from_utf8(&char_buf).unwrap());
            char_buf[0] = 0;
            char_buf[1] = 0;
            char_buf[2] = 0;
            char_buf[3] = 0;
            append = false;
        }
    }
    dbg!(&result);
    return result;
}

pub fn get_extension(mut path: &str) -> String {
    let mut result  = String::new();
    for mut c in path.chars().rev() {
        match c {
            '/' => {
                break;
            },
            '.' => {
                result.push('.');
                break;
            },
            h => {
                result.push(h);
            }
        }
    }
    return (result.chars().rev().collect());
}

pub fn get_MIME_from_filename(mut path: &str) -> String {
    match MIME_types.get(&get_extension(path)) {
        Some(val) => {
            return (&val).to_string();
        },
        None => {
            return "text".to_string();
        }
    }
}