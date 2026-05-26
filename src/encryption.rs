use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};

use argon2::Argon2;

use rand::{TryRng, rngs::SysRng};

use std::io;
use zeroize::Zeroize;

use crate::compression;

pub struct Encryption {
    pub password: Vec<u8>,
    pub key: [u8; 32],
    pub nonce: [u8; 24], //store it in archive
    pub salt: [u8; 16],  //store it in archive
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
    pub fn ask_password(&mut self) {
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .expect("failed to readline");
        self.password = answer.trim_end_matches(['\r', '\n']).as_bytes().to_vec();
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

        SysRng.try_fill_bytes(&mut self.nonce).unwrap();
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
pub fn encrypt_and_compress_flow(contents: &mut Vec<u8>) -> Vec<u8> {
    let mut encryption = Encryption {
        password: Vec::new(),
        key: [0u8; 32],
        nonce: [0u8; 24],
        salt: [0u8; 16],
    };

    encryption.ask_password();
    encryption.derive_key();
    let mut compressed = compression::compress(contents);

    encryption.encrypt(&mut compressed)
}

#[cfg(test)]
pub fn encrypt_and_compress_with_password(
    contents: &[u8],
    password: &[u8],
    compression_level: i32,
) -> Vec<u8> {
    let mut encryption = Encryption {
        password: password.to_vec(),
        key: [0u8; 32],
        nonce: [0u8; 24],
        salt: [0u8; 16],
    };

    encryption.derive_key();
    let mut compressed = compression::compress_with_level(contents, compression_level);

    encryption.encrypt(&mut compressed)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compression_encryption_flow_round_trips() {
        let input = b"hello encrypted compressed world".to_vec();
        let password = b"test-password";

        let encrypted = encrypt_and_compress_with_password(&input, password, 3);
        let decrypted = crate::decrypt::decrypt_and_decomp_bytes(&encrypted, password);

        assert_eq!(input, decrypted);
    }
}
