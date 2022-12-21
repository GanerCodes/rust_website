use crate::Rust_AES::encrypt::AES_Encrypt;
use crate::server::{MIME_types, response_codes};
use crate::config::*;

use std::{fmt, thread, str, fs};
use std::path::{Path, PathBuf};
use std::cmp::{min, max};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, Read, Write, prelude::*};
use std::collections::HashMap;
use std::fmt::Write as fmtWrite;

pub struct Response<> {
    pub code: u16,
    pub headers: HashMap<String, String>
}

pub struct HTTP_Parsed<> {
    pub Headers    : HashMap::<String, String>,
    pub Parameters : HashMap::<String, String>,
    pub Identifier : String,
    pub Method     : String,
    pub Heads      : String,
    pub Params     : String,
    pub Version    : String,
    pub Target     : String,
    pub Path       : String,
    pub Body       : Vec::<u8>
}

static Body_delim_pattern: [u8; 4] = [13, 10, 13, 10];
pub fn parse_request(raw_request : Vec<u8>) -> HTTP_Parsed {
    let request_string = String::from_utf8_lossy(&raw_request);
    let request_length = (&raw_request).len();
    
    let mut HTTP_Headers    = HashMap::<String, String>::new(); // {"User-Agent": "Various Personal Info"}
    let mut HTTP_Parameters = HashMap::<String, String>::new(); // {"foo": "bar"}
    let mut HTTP_Identifier = String::from(""); // First line of request
    let mut HTTP_Method     = String::from(""); // GET, POST, etc
    let mut HTTP_Heads      = String::from(""); // accept: *\*\r\ncache-control: no-cache
    let mut HTTP_Params     = String::from(""); // ?foo=bar&amogus=sus
    let mut HTTP_Version    = String::from(""); // HTTP/1.1
    let mut HTTP_Target     = String::from(""); // 
    let mut HTTP_Path       = String::from(""); //
    let mut HTTP_Body       = Vec::<u8>::new(); // Binary data
    
    let mut flagName        = String::from("");
    let mut flagValue       = String::from("");
    let mut bodyStartIndex: usize = 0;
    
    let mut bodyDelimIndex = 0;
    let mut editMode = 0;
    
    let mut spl_1 = request_string.splitn(2, "\r\n");
    HTTP_Identifier = spl_1.next().unwrap().to_string();
    HTTP_Heads = spl_1.next().unwrap().to_string();
    HTTP_Headers = hashmapFromDelims(&HTTP_Heads, ':', '\n');
    /* for (_, v) in HTTP_Headers.iter_mut() { // Why did I have this?
        *v = v.to_lowercase();
    } */
    
    let mut j = 0;
    for i in 0..(request_length - 1) {
        if raw_request[i] == Body_delim_pattern[j] {
            if j == 3 {
                HTTP_Body = (&raw_request[i + 1..]).to_vec();
                break;
            }
            j += 1
        }else{
            j = 0;
        }
    }
    
    let mut Identifer_itter = HTTP_Identifier.split(' ');
    HTTP_Method  = Identifer_itter.next().unwrap().trim().to_string();
    HTTP_Target  = Identifer_itter.next().unwrap().trim().to_string();
    HTTP_Version = Identifer_itter.next().unwrap().trim().to_string();
    
    editMode = 0;
    if HTTP_Target.contains('?') {
        editMode = 0;
        let loc = HTTP_Target.find('?').unwrap();
        HTTP_Path   = HTTP_Target[..loc].to_string();
        HTTP_Params = HTTP_Target[(loc+1)..].to_string();
        HTTP_Parameters = hashmapFromDelims(&HTTP_Params, '=', '&');
    }else{
        HTTP_Path = HTTP_Target.clone();
    }
    
    HTTP_Parsed{Headers    : HTTP_Headers,
                Parameters : HTTP_Parameters,
                Identifier : HTTP_Identifier,
                Method     : HTTP_Method,
                Heads      : HTTP_Heads,
                Params     : HTTP_Params,
                Version    : HTTP_Version,
                Target     : HTTP_Target,
                Path       : HTTP_Path,
                Body       : HTTP_Body}
}

pub fn printMap(mut map: HashMap::<String, String>) {
    for (key, value) in map.into_iter() {
        println!("{:<32} :\t{}", key, value);
    }
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
    match fs::File::open(&filePath) {
        Ok(file) => {
            make_response(&stream, &response);
            
            let mut buffer = [0u8; DEFAULT_CHUNK_LENGTH];
            loop {
                let read_bytes = (&file).read(&mut buffer).unwrap() as usize;
                stream.write(&buffer[..read_bytes]);
                if read_bytes < DEFAULT_CHUNK_LENGTH {
                    break;
                }
            }
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
    let mut baseLower = base.to_lowercase();
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
    match MIME_types.get(&get_extension(path)) {
        Some(val) => {
            return (&val).to_string();
        },
        None => {
            return "text/plain".to_string();
        }
    }
}

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
    let eNum = min(
        eStr.parse::<u64>().unwrap_or(fileSize - 1),
        sNum + (DEFAULT_CHUNK_LENGTH as u64));
    
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

pub fn makeFileURL_bruh(fileName: &String, pathString: &String, Site_Path: &String, encryptedDir: bool) -> String {
    format!("{}://{}{}", PREFERRED_PROTOCOL, DOMAIN_NAME,
        if encryptedDir {
            let relative_name = format!("{}/{}", 
                pathString.trim_start_matches(&format!("{}/", BASE_DIR)),
            fileName);
            
            format!("{}{}", ENCRYPTED_PATH_PREFIX,
                encrypt_fileName(&format!("/{}", relative_name), AES_KEY)
            )
        } else {
            format!("{}{}", Site_Path, fileName)
        }
    )
}