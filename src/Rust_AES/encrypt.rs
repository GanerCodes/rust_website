use std::time::{Duration, Instant};
use std::time::SystemTime;
use std::thread::sleep;
use std::{slice, str};
use Rust_AES::tables::*;
use Rust_AES::aes::*;

pub fn AES_SubBytes(state: &mut [u8; 16]) {
    for i in 0..16 {
        state[i] = AES_S_Box[state[i] as usize];
    }
}

pub fn AES_ShiftRows(state: &mut [u8; 16]) {
    let mut buffer: [u8; 16] = [0; 16];
    
    buffer[0] = state[0];
    buffer[1] = state[5];
    buffer[2] = state[10];
    buffer[3] = state[15];
    buffer[4] = state[4];
    buffer[5] = state[9];
    buffer[6] = state[14];
    buffer[7] = state[3];
    buffer[8] = state[8];
    buffer[9] = state[13];
    buffer[10] = state[2];
    buffer[11] = state[7];
    buffer[12] = state[12];
    buffer[13] = state[1];
    buffer[14] = state[6];
    buffer[15] = state[11];
    
    for i in 0..16 {
        state[i] = buffer[i];
    }
}

pub fn AES_MixColumns(state: &mut [u8; 16]) { //Looks even more autistic in rust
    let mut buffer: [u8; 16] = [0; 16];
    
    buffer[0]  = AES_Mul2[state[0] as usize] ^ AES_Mul3[state[1] as usize] ^ state[2] ^ state[3];
    buffer[1]  = state[0] ^ AES_Mul2[state[1] as usize] ^ AES_Mul3[state[2] as usize] ^ state[3];
    buffer[2]  = state[0] ^ state[1] ^ AES_Mul2[state[2] as usize] ^ AES_Mul3[state[3] as usize];
    buffer[3]  = AES_Mul3[state[0] as usize] ^ state[1] ^ state[2] ^ AES_Mul2[state[3] as usize];

    buffer[4]  = AES_Mul2[state[4] as usize] ^ AES_Mul3[state[5] as usize] ^ state[6] ^ state[7];
    buffer[5]  = state[4] ^ AES_Mul2[state[5] as usize] ^ AES_Mul3[state[6] as usize] ^ state[7];
    buffer[6]  = state[4] ^ state[5] ^ AES_Mul2[state[6] as usize] ^ AES_Mul3[state[7] as usize];
    buffer[7]  = AES_Mul3[state[4] as usize] ^ state[5] ^ state[6] ^ AES_Mul2[state[7] as usize];

    buffer[8]  = AES_Mul2[state[8] as usize] ^ AES_Mul3[state[9] as usize] ^ state[10] ^ state[11];
    buffer[9]  = state[8] ^ AES_Mul2[state[9] as usize] ^ AES_Mul3[state[10] as usize] ^ state[11];
    buffer[10] = state[8] ^ state[9] ^ AES_Mul2[state[10] as usize] ^ AES_Mul3[state[11] as usize];
    buffer[11] = AES_Mul3[state[8] as usize] ^ state[9] ^ state[10] ^ AES_Mul2[state[11] as usize];
    
    buffer[12] = AES_Mul2[state[12] as usize] ^ AES_Mul3[state[13] as usize] ^ state[14] ^ state[15];
    buffer[13] = state[12] ^ AES_Mul2[state[13] as usize] ^ AES_Mul3[state[14] as usize] ^ state[15];
    buffer[14] = state[12] ^ state[13] ^ AES_Mul2[state[14] as usize] ^ AES_Mul3[state[15] as usize];
    buffer[15] = AES_Mul3[state[12] as usize] ^ state[13] ^ state[14] ^ AES_Mul2[state[15] as usize];
    
    for i in 0..16 {
        state[i] = buffer[i];
    }
}

pub fn AES_Encrypt_Block(mut message: [u8; 16], mut expandedKey: [u8; 176]) -> [u8; 16] {
    let mut state: [u8; 16] = [0; 16];
    for i in 0..16 {
        state[i] = message[i];
    }
    
    AES_AddRoundKey(&mut state, &expandedKey);
    for i in 0..9 {
        AES_SubBytes(&mut state);
        AES_ShiftRows(&mut state);
        AES_MixColumns(&mut state);
        AES_AddRoundKey(&mut state, &expandedKey[(16 * (i + 1))..]);
    }
    AES_SubBytes(&mut state);
    AES_ShiftRows(&mut state);
    AES_AddRoundKey(&mut state, &expandedKey[160..]);
    return state;
}

pub fn AES_Encrypt(mut message: &[u8], mut key: [u8; 16]) -> Vec::<u8> {
    let mut changedMessage = Vec::<u8>::new();
    let mut buffer: [u8; 16] = [0; 16];
    let mut chunks = message.chunks(16);
    let chunkLength = chunks.len();
    let mut index = chunkLength as u128;
    let mut expandedKey: [u8; 176] = [0; 176];
    let startTime = Instant::now();
    
    for chunk in chunks {
        index -= 1;
        let chunkLen = chunk.len();
        for i in 0..16 {
            buffer[i] = if i < chunkLen {chunk[i]} else {0};
        }
        
        AES_KeyExpansion((u128::from_le_bytes(key).wrapping_add(index)).to_le_bytes(), &mut expandedKey);
        let enc_chunk = AES_Encrypt_Block(buffer, expandedKey).to_vec();
        
        changedMessage.extend(enc_chunk);
    }
    
    let chunkLengthFloat = (chunkLength as f64);
    let expectedMaxtime = Duration::from_nanos(
        (1.5 * 20610.0 * chunkLengthFloat - 55.6488 * chunkLengthFloat * chunkLengthFloat) as u64
    );
    
    while startTime.elapsed() < expectedMaxtime {}
    
    return changedMessage;
}
