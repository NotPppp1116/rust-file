use argon2::Argon2;
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};

use std::fs::File;
use std::io::Read;

use crate::{compression, encryption::Encryption};

impl Encryption {
    #[must_use]
    fn get_salt_n_nounce(&mut self, path: &str) -> Vec<u8> {
        let mut file = File::open(path).unwrap();
        file.read_exact(&mut self.salt).unwrap();
        file.read_exact(&mut self.nonce).unwrap();

        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).unwrap();

        encrypted_data
    }
    fn recreate_key_from_exist_salt(&mut self) {
        Argon2::default()
            .hash_password_into(&self.password, &self.salt, &mut self.key)
            .unwrap();
    }

    #[must_use]
    fn decrypt_n_compress_to_dst(&self, data: &Vec<u8>) -> Vec<u8> {
        let cipher = XChaCha20Poly1305::new((&self.key).into());
        let nonce = XNonce::from_slice(&self.nonce);

        let compressed = cipher
            .decrypt(nonce, data.as_slice())
            .expect("failed to decrypt archive");

        compression::decompress_bytes(compressed.as_slice())
    }
}

pub fn decrypt_and_decomp(path: &str) -> Vec<u8> {
    let mut encryption = Encryption {
        password: Vec::new(),
        key: [0u8; 32],
        nonce: [0u8; 24],
        salt: [0u8; 16],
    };

    encryption.ask_password();

    let content = encryption.get_salt_n_nounce(path);
    encryption.recreate_key_from_exist_salt();
    encryption.decrypt_n_compress_to_dst(&content)
}

#[cfg(test)]
pub fn decrypt_and_decomp_bytes(data: &[u8], password: &[u8]) -> Vec<u8> {
    let mut encryption = Encryption {
        password: password.to_vec(),
        key: [0u8; 32],
        nonce: [0u8; 24],
        salt: [0u8; 16],
    };

    encryption.salt.copy_from_slice(&data[..16]);
    encryption.nonce.copy_from_slice(&data[16..40]);
    encryption.recreate_key_from_exist_salt();
    encryption.decrypt_n_compress_to_dst(&data[40..].to_vec())
}
