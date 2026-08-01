
fn main() {
    let bs: &[u8] = "g".as_bytes();
    let mut base64: Vec<char> = Vec::with_capacity(30);

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

	    // To Base64
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_11111100_00000000_00000000) >> 18) as usize]); // First char
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000011_11110000_00000000) >> 12) as usize]); // Second char
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000000_00001111_11000000) >> 6) as usize]);  // Third char
	    base64.push(BASE64_ALPHABET[ (quantum & 0b00000000_00000000_00000000_00111111) as usize]);        // Fourth char
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

	    // To Base64
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_11111100_00000000_00000000) >> 18) as usize]); // First char
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000011_11110000_00000000) >> 12) as usize]); // Second char
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000000_00001111_11000000) >> 6) as usize]);  // Third char
	    base64.push(BASE64_ALPHABET[ (quantum & 0b00000000_00000000_00000000_00111111) as usize]);        // Fourth char
	} else if bs.len() == i+1 {
	    // First octet
	    quantum = (bs[i+0] >> 2) as u32; // First 6 bits
	    quantum = (quantum << 2) | ((bs[i+0] << 6) >> 6) as u32; // Last 2 bits
	    // Second octet
	    quantum = quantum << 7;
	    quantum = quantum | BASE64_ALPHABET_PAD_INDEX as u32;
	    // Third octet
	    quantum = quantum << 7; 
	    quantum = quantum | BASE64_ALPHABET_PAD_INDEX as u32;

	    // To Base64
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00111111_00000000_00000000) >> 16) as usize]); // First char
	    base64.push(BASE64_ALPHABET[(((quantum & 0b00000000_00000000_11000000_00000000) >> 14) << 4) as usize]); // Second char
	    base64.push(BASE64_ALPHABET_PAD); // Third char
	    base64.push(BASE64_ALPHABET_PAD); // Fourth char
	}

	i += 3; // Jump to next input quantum
    }

    for c in base64 {
	print!("{}", c);
    }
}

const BASE64_ALPHABET_PAD: char = '=';
const BASE64_ALPHABET_PAD_INDEX: u8 = (BASE64_ALPHABET.len() - 1) as u8; // Last itemp is the pad char

const BASE64_ALPHABET: [char; 65] = [
    // Upper case
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    // Lower case
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    // Numbers
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    // Special characters
    '-', '_', BASE64_ALPHABET_PAD
];
