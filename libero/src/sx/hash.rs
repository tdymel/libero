pub(crate) const fn hash_u8(mut hash: u64, value: u8) -> u64 {
    hash ^= value as u64;
    hash.wrapping_mul(0x00000100000001B3)
}

pub(crate) const fn hash_str(mut hash: u64, value: &str) -> u64 {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        hash = hash_u8(hash, bytes[index]);
        index += 1;
    }
    hash
}
