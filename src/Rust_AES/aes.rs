use Rust_AES::tables::*;

pub fn AES_KeyExpansionCore(inp: &mut [u8; 4], i: usize) {
    let t = inp[0];
    inp[0] = inp[1];
    inp[1] = inp[2];
    inp[2] = inp[3];
    inp[3] = t;
    
    inp[0] = AES_S_Box[inp[0] as usize];
    inp[1] = AES_S_Box[inp[1] as usize];
    inp[2] = AES_S_Box[inp[2] as usize];
    inp[3] = AES_S_Box[inp[3] as usize];
    
    inp[0] ^= AES_Rcon[i];
}

pub fn AES_KeyExpansion(mut inputKey: [u8; 16], expandedKeys: &mut [u8; 176]) {
    for i in 0..16 {
        expandedKeys[i] = inputKey[i];
    }
    
    let mut bytesGenerated = 16;
    let mut rconIteration = 1;
    let mut buffer: [u8; 4] = [0; 4];
    
    while bytesGenerated < 176 {
        for i in 0..4 {
            buffer[i] = expandedKeys[i + bytesGenerated - 4];
        }
        
        if bytesGenerated % 16 == 0 {
            AES_KeyExpansionCore(&mut buffer, rconIteration);
            rconIteration += 1;
        }
        
        for a in 0..4 {
            expandedKeys[bytesGenerated] = expandedKeys[bytesGenerated - 16] ^ buffer[a];
            bytesGenerated += 1;
        }
    }
}

pub fn AES_AddRoundKey(state: &mut [u8; 16], roundKey: &[u8]) {
    for i in 0..16 {
        state[i] ^= roundKey[i];
    }
}