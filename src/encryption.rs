use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};

use argon2::Argon2;

use rand::{TryRng, rngs::SysRng};

use std::{fs, io};
use zeroize::Zeroize;

use crate::compression;

//TODO make this better to a out file of argument
const OUTPUT_FILE: &str = "out.mole";
const FALLBACK_OUTPUT_FILE: &str = "out324343.mole";

struct Encryption {
    password: Vec<u8>,
    key: [u8; 32],
    nonce: [u8; 24], //store it in archive
    salt: [u8; 16],  //store it in archive
}

impl Drop for Encryption {
    fn drop(&mut self) {
        self.password.zeroize();
        self.key.zeroize();
        self.nonce.zeroize();
        self.salt.zeroize();
    }
}

impl Encryption {
    fn ask_password(&mut self) {
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .expect("failed to readline");
        self.password = answer.into_bytes();
    }
    fn derive_key(&mut self) {
        SysRng.try_fill_bytes(&mut self.salt).unwrap();
        Argon2::default()
            .hash_password_into(&self.password, &self.salt, &mut self.key)
            .unwrap();
    }

    #[must_use]
    fn encrypt(&mut self, contents: &mut Vec<u8>) -> Vec<u8> {
        let cipher = XChaCha20Poly1305::new((&self.key).into());

        let nonce = XNonce::from_slice(&self.nonce);

        let ciphertext = cipher.encrypt(nonce, contents.as_slice()).unwrap();

        let mut output = Vec::new();

        // prepend salt
        output.extend_from_slice(&self.salt);

        // prepend nonce
        output.extend_from_slice(&self.nonce);

        // encrypted archive
        output.extend_from_slice(&ciphertext);

        output
    }
}

//encrypts compresses and writes to file
pub fn encrypt_and_compress_flow(contents: &mut Vec<u8>) {
    let mut encryption = Encryption {
        password: Vec::new(),
        key: [0u8; 32],
        nonce: [0u8; 24],
        salt: [0u8; 16],
    };
    encryption.ask_password();
    encryption.derive_key();
    let enc_finale = encryption.encrypt(contents);

    let compressed = compression::compress(&enc_finale);
    match fs::write(OUTPUT_FILE, compressed) {
        Ok(v) => v,
        Err(_) => {
            fs::write(FALLBACK_OUTPUT_FILE, contents).unwrap();
        }
    };
}
