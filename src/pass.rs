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
