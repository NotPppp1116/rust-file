use crate::utils::ask_number;
use zstd::stream::{decode_all, encode_all};

const MAX: i32 = 22;
const MIN: i32 = -7;

pub fn compress(data: &Vec<u8>) -> Vec<u8> {
    let level: i32 = loop {
        let n = ask_number(
            "what level of compression you want to use -7 to 22 (smaller is faster but less compressed)? ",
        );

        if (MIN..=MAX).contains(&n) {
            break n;
        }
    };

    encode_all(data.as_slice(), level).expect("zstd compression failed")
}

pub fn decompress_bytes(data: &[u8]) -> Vec<u8> {
    decode_all(data).expect("zstd decompression failed")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compress_uncompress() {
        let input = String::from("helo world? ?");

        let compressed = encode_all(input.as_bytes(), 3).expect("zstd compression failed");
        let uncompress = decompress_bytes(&compressed);
        let string = String::from_utf8(uncompress).unwrap();

        assert_eq!(input, string);
    }
}
