use argon2::Argon2;
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};

use std::fs::File;
use std::io::{self, ErrorKind, Read};

use crate::{compression, debug_safety, encryption::Encryption};

impl Encryption {
    fn get_salt_n_nounce(&mut self, path: &str) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;
        file.read_exact(&mut self.salt)?;
        file.read_exact(&mut self.nonce)?;

        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;

        Ok(encrypted_data)
    }
    fn recreate_key_from_exist_salt(&mut self) {
        Argon2::default()
            .hash_password_into(&self.password, &self.salt, &mut self.key)
            .unwrap();
    }

    fn decrypt_n_compress_to_dst(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        let cipher = XChaCha20Poly1305::new((&self.key).into());
        let nonce = XNonce::from_slice(&self.nonce);

        let compressed = cipher
            .decrypt(nonce, data)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "failed to decrypt archive"))?;

        Ok(compression::decompress_bytes(compressed.as_slice()))
    }
}

pub fn decrypt_and_decomp(path: &str) -> io::Result<Vec<u8>> {
    let mut encryption = Encryption {
        password: Vec::new(),
        key: [0u8; 32],
        nonce: [0u8; 24],
        salt: [0u8; 16],
    };
    //see if we are being debugged if yes panic with unwind so destructor runs
    if debug_safety::is_being_debugged() == true {
        panic!();
    }
    encryption.ask_password();

    let content = encryption.get_salt_n_nounce(path)?;
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
    encryption.decrypt_n_compress_to_dst(&data[40..]).unwrap()
}
