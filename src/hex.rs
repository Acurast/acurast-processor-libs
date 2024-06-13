
pub fn hex_to_ba(hex_string: &str) -> Option<Vec<u8>> {
    // Check if the string has the correct format (even number of characters)
    if hex_string.len() % 2 != 0 {
        return None; // Invalid format
    }

    let mut bytes = Vec::new();

    // Iterate over pairs of characters and convert them to bytes
    for i in 0..hex_string.len() / 2 {
        let byte_str = &hex_string[i * 2..(i + 1) * 2];
        if let Ok(byte) = u8::from_str_radix(byte_str, 16) {
            bytes.push(byte);
        } else {
            return None; // Invalid hexadecimal character
        }
    }

    Some(bytes)
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for Vec<u8> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}