use sha2::{Digest, Sha256};

pub fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

pub fn compute_sha256_hex(content_without_stamp: &str) -> String {
    let normalized = normalize_newlines(content_without_stamp);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let out = hasher.finalize();
    hex_lower(&out)
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}
