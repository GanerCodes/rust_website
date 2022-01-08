use crate::Rust_AES::decrypt::AES_Decrypt;
use crate::utils::*;
use crate::sha256::sha256;
use crate::*;

use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};
use std::lazy::SyncLazy;
use std::{thread, str, fs};
use std::fs::{File, DirEntry, OpenOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{SeekFrom, BufReader, Read, Write, prelude::*};

static Body_delim_pattern: [u8; 4] = [13, 10, 13, 10];

pub static MIME_types: SyncLazy<HashMap::<String, String>> = SyncLazy::new(|| {
    let mut MIMEs = HashMap::<String, String>::new();
    let MIME_types_file = fs::File::open(&MIME_FILE_PATH).unwrap();
    let MIME_types_reader = BufReader::new(MIME_types_file);
    for line in MIME_types_reader.lines() {
        let lineRaw = line.unwrap();
        let mut lineSplit = lineRaw.splitn(2, ", ");
        MIMEs.insert(lineSplit.next().unwrap().to_string(), lineSplit.next().unwrap().to_string());
    }
    MIMEs
});
pub static response_codes: SyncLazy<HashMap::<u16, String>> = SyncLazy::new(|| {
    let mut codeMapping = HashMap::<u16, String>::new();
    let response_codes_file = fs::File::open(&CODES_FILE_PATH).unwrap();
    let response_codes_reader = BufReader::new(response_codes_file);
    for line in response_codes_reader.lines() {
        let lineRaw = line.unwrap();
        let mut lineSplit = lineRaw.splitn(2, ", ");
        codeMapping.insert(
            lineSplit.next().unwrap().to_string().parse::<u16>().unwrap(),
            lineSplit.next().unwrap().to_string()
        );
    }
    codeMapping
});

pub fn handle_client(mut stream: TcpStream, mut URL_Shorts_shared: Arc<Mutex<HashMap::<String, String>>>) {
    let mut raw_request = Vec::new();
    stream.set_read_timeout(Some(Duration::from_millis(150)));
    stream.read_to_end(&mut raw_request);
    
    let request_length = (&raw_request).len();
    if request_length == 0 { return; } //why does this happen so much?
    
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
    
    let request_string = String::from_utf8_lossy(&raw_request);
    let mut spl_1 = request_string.splitn(2, "\r\n");
    HTTP_Identifier = spl_1.next().unwrap().to_string();
    HTTP_Heads = spl_1.next().unwrap().to_string();
    HTTP_Headers = hashmapFromDelims(&HTTP_Heads, ':', '\n');
    
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
    
    let mut Site_Path = decode_url(&HTTP_Path);
        
    /*TODO:
        unretard the code
        path grepping file, move settings into json, etc
    */
    
    let mut response_headers = HashMap::<String, String>::new();
    response_headers.insert("Server"      .to_string(), "Gone, reduced to atoms".to_string());
    response_headers.insert("Content-Type".to_string(), "text/html".to_string());
    response_headers.insert("Connection"  .to_string(), "Closed".to_string());
    
    let mut pathString = formatPath(&Site_Path);
    
    if pathString.chars().last().unwrap() != '/' { //Enforce prefix detection
        pathString.push('/');
    }
    
    println!("{} {} {}", &HTTP_Version, &HTTP_Method, &HTTP_Target);
    
    'big: loop {
    let mut shorthandDir = false;
    let mut encryptedDir = false;
    
    if pathString.starts_with(SHORTHAND_PATH_PREFIX) {
        let cutPath = (&pathString[3..]).trim_matches('/');
        let URL_Shorts = URL_Shorts_shared.lock().unwrap();
        if URL_Shorts.contains_key(cutPath) { //I need to figure out a way to do better string pattern stuff
            'shorthand: loop {
                let redirPath = URL_Shorts.get(cutPath).unwrap().to_string();
                for i in WEBSITE_PREFIXES {
                    if redirPath.starts_with(i) {
                        pathString = format!("/{}", redirPath.strip_prefix(i).unwrap().to_string());
                        Site_Path = pathString.clone();
                        shorthandDir = true;
                        break 'shorthand;
                    }
                }
                send_redirect(&stream, &redirPath);
                break 'big;
            }
        }else{
            respondCodeText(&stream, response_headers, 404);
            break;
        }
    }
    
    if pathString.starts_with(ENCRYPTED_PATH_PREFIX) { //Secret filepaths
        let cutPath = &pathString[ENCRYPTED_PATH_PREFIX.len()..];
        let mut cutPathHexDecoded = hexToBytes(String::from(cutPath).as_bytes().to_vec());
        let mut decryptedPathRaw = AES_Decrypt(&cutPathHexDecoded, AES_KEY);
        if decryptedPathRaw.len() == 0 {
            respondCodeText(&stream, response_headers, 404);
            break;
        }
        
        //Stupid way to trim ending 0 bytes
        let mut k = 0;
        for i in (&decryptedPathRaw).into_iter().rev() {
            if *i == 0 {
                k += 1;
            }else{
                break;
            }
        }
        let mut trimmedDecryptedPath = &decryptedPathRaw[..(decryptedPathRaw.len() - k)];
        
        match String::from_utf8(trimmedDecryptedPath.to_vec()) {
            Ok(path) => {
                Site_Path = path;
                encryptedDir = true;
            }, _ => {
                respondCodeText(&stream, response_headers, 404);
                break;
            }
        }
    }
    
    pathString = formatPath(&format!("{}{}", &BASE_DIR, &Site_Path));
    
    match (&HTTP_Method).as_str() {
    "GET" => {
        if !DTAsafe(&pathString, &BASE_DIR) {
            make_response(&stream, &Response {
                code: 200,
                headers: response_headers
            });
            stream.write(b"<script>location.href='https://youtu.be/dQw4w9WgXcQ';</script>");
            break;
        }
        
        let mut filePath = PathBuf::from(&pathString);
        match filePath.canonicalize() {
            Ok(filePath) => {
                if filePath.is_file() {
                    let mut MIME = get_MIME_from_filename(&pathString);
                    let mut sMIME = splitMIME(&MIME).0;
                    
                    if HTTP_Parameters.contains_key("e") && (sMIME == "image" || sMIME == "video") {
                        make_response(&stream, &Response{
                            code: 200,
                            headers: response_headers
                        });
                        let newURL = format!("{}://{}{}?e", PREFERRED_PROTOCOL, DOMAIN_NAME, &HTTP_Path);
                        stream.write(format!( //this is a good way to do this
                            "<!DOCTYPE html> <html> <head> <style> html {{ background: #010101; overflow: auto; width: 100vw; height: 100vh; }} body {{ display: flex; justify-content: center; align-items: center; margin: auto; width: 100%; height: 100%; }}</style><meta content=\"{}\" property=\"og:image\"/><meta name=\"twitter:card\" content=\"summary_large_image\"></head><body><{} src=\"{}\"></{}></body><html>",
                            newURL, sMIME, newURL, sMIME
                        ).as_bytes());
                        break;
                    }
                    
                    response_headers.insert("Content-Type".to_string(), MIME.clone());
                    response_headers.insert("Accept-Ranges".to_string(), "bytes".to_string());
                    
                    let fileMetadata = fs::metadata(&filePath).unwrap();
                    let fileSize = fileMetadata.len();
                    let fileSizeString = fileSize.to_string();
                    
                    let rangeKey = "range".to_string();
                    if HTTP_Headers.contains_key(&rangeKey) {
                        let contentRange = parseRangeHeader(&HTTP_Headers.get(&rangeKey).unwrap(), fileSize);
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
                            headers: response_headers
                        });
                        stream.write(&videoFileBuffer);
                        break;
                    }
                    
                    response_headers.insert("Content-Range".to_string(), format!("bytes {}-{}/{}", 0, fileSize - 1, fileSize));
                    response_headers.insert("Content-Length".to_string(), fileSizeString);
                    write_file(&stream, &Response {
                        code: 200,
                        headers: response_headers },
                        &filePath
                    );
                    break;
                }else{
                    if Site_Path.chars().last().unwrap() != '/' && !(shorthandDir || encryptedDir) {
                        send_redirect(&stream, &(format!("{}://{}{}/", PREFERRED_PROTOCOL, DOMAIN_NAME, &Site_Path)).to_string());
                        break;
                    }
                    let mut indexPath = filePath.clone();
                    indexPath.push("index.html");
                    if indexPath.exists() {
                        write_file(&stream, &Response {
                            code: 200,
                            headers: response_headers
                        }, &indexPath);
                    } else {
                        let is_json = HTTP_Parameters.contains_key("json");
                        let file_listing = fs::read_dir(filePath.clone()).unwrap();
                        let mut dir_listing_data = String::from("");
                        
                        if is_json {
                            response_headers.insert("Content-Type".to_string(), "application/JSON".to_string());
                            dir_listing_data = String::from("[");
                            let mut file_listing_peekable = file_listing.peekable();
                            while let Some(fileEntry) = file_listing_peekable.next() {
                                let file = fileEntry.unwrap();
                                let metadata = file.metadata().unwrap();
                                let time_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH).duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                                
                                let fileName = file.file_name().into_string().unwrap();
                                dir_listing_data.push_str(
                                    format!("{{\"url\":\"{}\",\"file_name\":\"{}\",\"time\":\"{}\",\"is_file\":\"{}\"}}",
                                        makeFileURL_bruh(&fileName, &pathString, &Site_Path, encryptedDir),
                                        fileName,
                                        time_modified,
                                        if metadata.is_file() {"true"} else {"false"}
                                    ).as_str()
                                );
                                if !file_listing_peekable.peek().is_none() {
                                    dir_listing_data.push_str(",");
                                }
                            }
                            dir_listing_data.push_str("]");
                        } else {
                            dir_listing_data = String::from("<html>\n<head>\n<style>\nhtml {\ncolor:white;\nbackground-color:black;\n}\n</style>\n</head><body>");
                            for fileEntry in file_listing {
                                let fileName = fileEntry.unwrap().file_name().into_string().unwrap();
                                let mut file_URL_Path = makeFileURL_bruh(&fileName, &pathString, &Site_Path, encryptedDir);
                                dir_listing_data.push_str(&format!("<p><a href=\"{}\">{}</a></p>\n", &file_URL_Path, &fileName));
                            }
                            dir_listing_data.push_str("</body>\n</html>");
                        }
                        make_response(&stream, &Response{
                            code: 200,
                            headers: response_headers
                        });
                        stream.write(dir_listing_data.as_bytes());
                    }
                }
            },
            Err(_) => {
                respondCodeText(&stream, response_headers, 404);
                break;
            }
        }
    },
    "POST" => {
        if !HTTP_Headers.contains_key("access") || HTTP_Headers.get("access").unwrap() != ACCESS_PASSWORD {
            respondCodeText(&stream, response_headers, 404);
            break;
        }
        
        match Site_Path.as_str() {
            "/upload" => {
                if HTTP_Body.len() == 0 {
                    make_response(&stream, &Response{
                        code: 400,
                        headers: response_headers
                    });
                    break;
                }
                
                let newFileDir  = format!("{}{}", BASE_DIR, UPLOAD_FILE_PATH);
                
                let fileNameFromHeader = if HTTP_Headers.contains_key("filename") {HTTP_Headers.get("filename").unwrap()} else {"youJustLostTheGame.txt"};
                let fileName = format!("{}{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(), fileNameFromHeader);
                let fileExt  = get_extension(&fileName);
                
                let newFileName = format!("{}{}", &sha256(fileName.as_bytes())[..24], &fileExt);
                let newFilePath = format!("{}/{}", newFileDir, newFileName);
                
                fs::create_dir_all(&newFileDir).unwrap();
                let mut newFile = File::create(newFilePath).unwrap();
                newFile.write_all(&HTTP_Body);
                
                let encryptedSegment = encrypt_fileName(
                    &format!("{}/{}", UPLOAD_FILE_PATH, newFileName),
                    AES_KEY
                );
                
                let encryptedURL = format!("{}://{}{}{}", PREFERRED_PROTOCOL, DOMAIN_NAME, ENCRYPTED_PATH_PREFIX, encryptedSegment);
                let finalURL = encryptedURL.as_bytes();
                
                response_headers.insert("Content-Length".to_string(), finalURL.len().to_string());
                make_response(&stream, &Response{
                    code: 200,
                    headers: response_headers
                });
                stream.write(&finalURL);
                break;
            },
            "/shortenURL" => {
                if HTTP_Headers.contains_key("url") {
                    let mut URL_Shorts = URL_Shorts_shared.lock().unwrap();
                    let passed_url = HTTP_Headers.get("url").unwrap();
                    let isFilepath = HTTP_Headers.contains_key("localpath");
                    let base_url = if isFilepath {
                        format!("{}://{}{}{}", PREFERRED_PROTOCOL, DOMAIN_NAME, ENCRYPTED_PATH_PREFIX, encrypt_fileName(passed_url, AES_KEY))
                    } else {
                        passed_url.to_string()
                    };
                    
                    let url_hash = &sha256(base_url.as_bytes())[..16];
                    make_response(&stream, &Response{
                        code: 200,
                        headers: response_headers
                    });
                    if !URL_Shorts.contains_key(url_hash) {
                        URL_Shorts.insert(url_hash.to_string(), (&base_url).to_string());
                        let mut shorthand_file = OpenOptions::new().write(true).append(true).open(SHORTHAND_FILE_PATH).unwrap();
                        writeln!(shorthand_file, "{}:{}\n", url_hash, base_url).unwrap();
                    }
                    stream.write(format!("{}://{}{}{}", PREFERRED_PROTOCOL, DOMAIN_NAME, SHORTHAND_PATH_PREFIX, url_hash).as_bytes());
                }else{
                    respondCodeText(&stream, response_headers, 404);
                    break;
                }
            },
            _ => {
                respondCodeText(&stream, response_headers, 404);
                break;
            }
        }
        
    }, _ => ()
    
    } //Match protocols
    
    break;
    } //Janky breakable loop
}