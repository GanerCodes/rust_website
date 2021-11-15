use crate::Rust_AES::decrypt::AES_Decrypt;
use crate::utils::*;
use crate::sha256::sha256;
use crate::*;

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::lazy::SyncLazy;
use std::time::SystemTime;
use std::{thread, str, fs};
use std::fs::{File, DirEntry};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{SeekFrom, BufReader, Read, Write, prelude::*};

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

pub fn handle_client(mut stream: TcpStream, mut URL_Shorts_shared: Arc<Mutex<HashMap::<String, String>>>) {
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
    let mut bodyStartIndex: usize = 0;
    
    let mut bodyDelimIndex = 0;
    let mut editMode = 0;
    for i in 0..(&raw_request).len() {
        let c = char::from((&raw_request)[i]);
        if (&raw_request)[i] == Body_delim_pattern[bodyDelimIndex] {
            if bodyDelimIndex == 2 {
                bodyStartIndex = i + 2;
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
                        headers.insert(HeaderName.clone().trim().to_string(), HeaderValue.clone().trim().to_string());
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
        
    /*TODO:
        directory listing
        video/image/file uploader
        url shorterner
        gallery view, with comics [as well as supporting video and audio] with multiple views [scroll, page-by-page [with mouse click side turning direction]]
        chunked file delivery?
        https?
        path grepping file, move settings into json, etc
    */
    
    let mut response_headers = HashMap::<String, String>::new();
    response_headers.insert("Server"      .to_string(), "amogus"   .to_string());
    response_headers.insert("Content-Type".to_string(), "text/html".to_string());
    response_headers.insert("Connection"  .to_string(), "Closed"   .to_string());
    
    // dbg!(&encrypt_fileName(&String::from("/e/"), AES_KEY));
    let mut pathString = formatPath(&HTTP_Target);
    if pathString.chars().last().unwrap() != '/' { //Enforce prefix detection
        pathString.push('/');
    }
    
    println!("{} {} {}", HTTP_Version, HTTP_Method, HTTP_Target);
    
    loop {
    let mut shorthandDir = false;
    let mut encryptedDir = false;
    
    // let name = format!("cheese{}", URL_Shorts.len());
    // URL_Shorts.insert(name, "cheese".to_string());
    // dbg!(&URL_Shorts);
    
    if pathString.starts_with(SHORTHAND_PATH_PREFIX) {
        let mut URL_Shorts = URL_Shorts_shared.lock().unwrap();
        let cutPath = (&pathString[3..]).trim_matches('/');
        if URL_Shorts.contains_key(cutPath) {
            pathString = URL_Shorts.get(cutPath).unwrap().to_string();
            // pathString.trim_start_matches(pat: P)("http")
        }else{
            respond_404(&stream, response_headers);
            break;
        }
    }
    
    if pathString.starts_with(ENCRYPTED_PATH_PREFIX) { //Secret filepaths
        let cutPath = &pathString[3..];
        let mut cutPathHexDecoded = hexToBytes(String::from(cutPath).as_bytes().to_vec());
        let mut decryptedPathRaw = AES_Decrypt(&cutPathHexDecoded, AES_KEY);
        
        if decryptedPathRaw.len() == 0 {
            respond_404(&stream, response_headers);
            break;
        }
        
        //Stupid way to trim ending 0 bytes
        let mut k = 0;
        for i in (&decryptedPathRaw).into_iter().rev() {
            if *i == 0 {
                k += 1;
                continue;
            }
            break;
        }
        let mut trimmedDecryptedPath = &decryptedPathRaw[..(decryptedPathRaw.len() - k)];
        
        match String::from_utf8(trimmedDecryptedPath.to_vec()) {
            Ok(path) => {
                HTTP_Target = path;
                encryptedDir = true;
            }, _ => {
                respond_404(&stream, response_headers);
                break;
            }
        }
    }
    
    pathString = formatPath(&format!("{}{}", &BASE_DIR, &HTTP_Target));
    
    match (&HTTP_Method).as_str() {
    "GET" => {
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
                    if(HTTP_Target.chars().last().unwrap() != '/') {
                        send_redirect(&stream, &(format!("{}/", &HTTP_Target)).to_string());
                        break;
                    }
                    let mut indexPath = filePath.clone();
                    indexPath.push("index.html");
                    if indexPath.exists() {
                        write_file(&stream, &Response {
                            code: 200,
                            code_name: "OK",
                            headers: response_headers
                        }, &indexPath);
                    } else {
                        let mut dirListingHtml = String::from("<html>\n<head>\n<style>\nhtml {\ncolor:white;\nbackground-color:black;\n}\n</style>\n</head><body>");
                        for fileEntry in fs::read_dir(filePath).unwrap() {
                            let fileName = fileEntry.unwrap().file_name().into_string().unwrap();
                            let mut file_URL_Path = if encryptedDir {
                                encrypt_fileName(&(String::from("/e/") + &fileName), AES_KEY)
                            } else {
                                fileName.clone()
                            };
                            dirListingHtml.push_str(&format!("<p><a href=\"{}\">{}</a></p>\n", &file_URL_Path, &fileName));
                        }
                        dirListingHtml.push_str("</body>\n</html>");
                        make_response(&stream, &Response{
                            code: 200,
                            code_name: "OK",
                            headers: response_headers
                        });
                        stream.write(dirListingHtml.as_bytes());
                    }
                }
            },
            Err(_) => {
                respond_404(&stream, response_headers);
                break;
            }
        }
    },
    "POST" => {
        if bodyStartIndex >= raw_request.len() {
            respond_400(&stream, response_headers);
            break;
        }
        let data = &raw_request[(bodyStartIndex as usize)..];
        
        let newFileDir  = format!("{}{}", BASE_DIR, UPLOAD_FILE_PATH);
        
        let fileNameFromHeader = if headers.contains_key("filename") {headers.get("filename").unwrap()} else {"youJustLostTheGame.txt"};
        let fileName = format!("{}{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(), fileNameFromHeader);
        let fileExt  = get_extension(&fileName);
        
        let newFileName = format!("{}{}", &sha256(fileName.as_bytes())[..24], &fileExt);
        let newFilePath = format!("{}/{}", newFileDir, newFileName);
        
        dbg!(&fileName, &fileExt, &newFileName, &newFilePath);
        
        fs::create_dir_all(&newFileDir).unwrap();
        let mut newFile = File::create(newFilePath).unwrap();
        newFile.write_all(&data);
        
        let encryptedSegment = encrypt_fileName(
            &format!("{}/{}", UPLOAD_FILE_PATH, newFileName),
            AES_KEY
        );
        
        let encryptedURL = format!("localhost:{}/e/{}", PORT, encryptedSegment);
        let finalURL = encryptedURL.as_bytes();
        
        response_headers.insert("Content-Length".to_string(), finalURL.len().to_string());
        make_response(&stream, &Response{
            code: 200,
            code_name: "OK",
            headers: response_headers
        });
        stream.write(&finalURL);
        break;
    },
    _ => ()
    
    }
    
    break;
    }
}