#![allow(warnings, unused)]
mod aes;
mod tables;
mod encrypt;
mod decrypt;

use crate::aes::*;
use crate::encrypt::*;
use crate::decrypt::*;
use std::time::*;
use std::{slice, str};
use std::time::SystemTime;
use std::collections::HashMap;

pub fn makeRandomBytes(mut balls: u128) -> Vec<u8>{
    let mut bytes = Vec::new();
    for i in 0..balls {
        let byte = (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros() % 256) as u8;
        bytes.push(byte);
    }
    return bytes; 
}

pub fn main() {
    let key: [u8; 16] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10];
    let msg = b"This is a message we will encrypt with AES!";
    
    let enc = AES_Encrypt(msg, key);
    println!("{:x?}", enc);
    let dec = AES_Decrypt(&enc, key);
    println!("{}", String::from_utf8_lossy(&dec));
    
    
    /* let mut timings = HashMap::<usize, Vec<usize>>::new();
    for i in 0..100 {
        timings.insert(i, Vec::<usize>::new());
    }
    let mut msg = makeRandomBytes(0);
    let mut now = Instant::now();
    let mut badKey: [u8; 16] = [0; 16];
    
    for i in 0..1000 {
        msg = makeRandomBytes(16 * ((i as u128) % (100)));
        
        now = Instant::now();
        let enc = AES_Encrypt(&msg, key);
        let mut encTime = now.elapsed().as_nanos();
        
        now = Instant::now();
        // let badBytes = makeRandomBytes(16);
        // for o in 0..16 {
            // badKey[o] = badBytes[o];
        // }
        let dec = AES_Encrypt(&enc, key);
        let mut decTime = now.elapsed().as_nanos();
        
        timings.get_mut(&msg.chunks(16).len()).unwrap().push(decTime as usize);
    }
    
    let mut mins = String::from("m_{ins}=[");
    let mut maxs = String::from("m_{axs}=[");
    let mut advs = String::from("a_{dvs}=[");
    
    for (key, value) in timings.into_iter() {
        let mut min: f64 = 999999999999999999999999999999999999.0;
        let mut max: f64 = 0.0;
        let mut adv: f64 = 0.0;
        let valLen = value.len() as f64;
        for k in value {
            let f = k as f64;
            if f < min {
                min = f;
            }
            if f > max {
                max = f;
            }
            adv += f / valLen;
        }
        mins.push_str(&format!("({}, {}), ", key, min));
        maxs.push_str(&format!("({}, {}), ", key, max));
        advs.push_str(&format!("({}, {}), ", key, adv));
    }
    println!("{}", mins);
    println!("{}", maxs);
    println!("{}", advs); */
}