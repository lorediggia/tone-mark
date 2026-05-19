pub fn roland_checksum(payload: &[u8]) -> u8 {
    let sum: u32 = payload.iter().map(|&x| x as u32).sum();
    let rem = (sum % 128) as u8;
    if rem == 0 {
        0
    } else {
        128 - rem
    }
}

pub fn build_sysex(address: [u8; 4], payload_data: &[u8]) -> Vec<u8> {
    let mut msg = vec![0xF0, 0x41, 0x00, 0x00, 0x00, 0x00, 0x33, 0x12];
    let mut checksum_payload = address.to_vec();
    checksum_payload.extend_from_slice(payload_data);
    msg.extend_from_slice(&checksum_payload);
    msg.push(roland_checksum(&checksum_payload));
    msg.push(0xF7);
    msg
}
