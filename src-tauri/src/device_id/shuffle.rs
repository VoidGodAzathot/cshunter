pub fn shuffle(data: String, key: String) -> String {
    let key_r = key.as_bytes();
    let bytes = data.as_bytes();
    let buf_len = bytes.len();
    let mut response = vec![0_u8; buf_len];
    for i in 0..buf_len {
        response[i] = (bytes[i] ^ key_r[i % key_r.len()]) as u8;
    }
    String::from_utf8(response.to_vec()).unwrap()
}
