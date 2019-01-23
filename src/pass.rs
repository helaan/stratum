use argon2rs::verifier::Encoded;
use rand::prelude::*;

pub fn hash(pass: String) -> String {
    let mut salt: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    rand::thread_rng().fill(&mut salt[..]);
    let empty: [u8; 0] = [];
    let hash = Encoded::default2i(&pass.as_bytes(), &salt, &empty, &empty).to_u8();
    String::from_utf8(hash).expect("Could not hash password")
}

pub fn check(hash: &str, submitted: &str) -> Result<bool, String> {
    let enc = Encoded::from_u8(hash.as_bytes());
    match enc {
        Ok(e) => Ok(e.verify(submitted.as_bytes())),
        Err(e) => Err(e.to_string()),
    }
}
