
fn main() {
    let bs = "g".as_bytes();
    let mut base64: Vec<u8> = Vec::with_capacity(30);

    let mut i: usize = 0;
    while i < bs.len() {
	let mut quantum: u32 = 0; // a quantum is 24 bits (three bytes)
	if bs.len() >= i+3 {
	    // First octet
	    quantum = (bs[i+0] << 2) as u32; // First 6 bits
	    quantum = quantum | ((bs[i+0] << 6) >> 6) as u32; // Last 2 bits 
	    // Second octet
	    quantum = quantum << 8;
	    quantum = quantum | (bs[i+1] << 2) as u32; // First 6 bits
	    quantum = quantum | ((bs[i+1] << 6) >> 6) as u32; // Last 2 bits
	    // Third octet
	    quantum = quantum << 8;
	    quantum = quantum | (bs[i+2] << 2) as u32; // First 6 bits
	    quantum = quantum | ((bs[i+2] << 6) >> 6) as u32; // Last 2 bits 
	} else if bs.len() == i+2 {
	    // First octet
	    quantum = (bs[i+0] << 1) as u32; // First 6 bits
	    quantum = quantum | ((bs[i+0] << 6) >> 6) as u32; // Last 2 bits 
	    // Second octet
	    quantum = quantum << 8;
	    quantum = quantum | (bs[i+1] << 2) as u32; // First 6 bits
	    quantum = quantum | ((bs[i+1] << 6) >> 6) as u32; // Last 2 bits
	    // Third octet
	    quantum = quantum << 8;
	    quantum = quantum | BASE64_ALPHABET_PAD as u32;
	} else if bs.len() == i+1 {
	    // First octet
	    quantum = (bs[i+0] << 2) as u32; // First 6 bits
	    quantum = quantum | ((bs[i+0] << 6) >> 6) as u32; // Last 2 bits 
	    // Second octet
	    quantum = quantum << 8;
	    quantum = quantum | BASE64_ALPHABET_PAD as u32;
	    // Third octet
	    quantum = quantum << 8;
	    quantum = quantum | BASE64_ALPHABET_PAD as u32;
	}

	// To Base64
	base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00111111_00000000_00000000) >> (8 * 2)) as usize]); // First char
	base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000000_00111111_00000000) >> (8 * 1)) as usize]); // Second char
	base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000000_00000000_00111111)) as usize]);            // Third char

	i += 3; // Jump to next input quantum
    }

    for b in base64 {
	print!("{}", b as char);
    }
}

const BASE64_ALPHABET_PAD: u8 = b'=';

const BASE64_ALPHABET: [u8; 65] = [
    // Upper case
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M',
    b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
    // Lower case
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm',
    b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
    // Numbers
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
    // Special characters
    b'-', b'_', BASE64_ALPHABET_PAD
];
