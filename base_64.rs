
use std::io;

// Test with command:
// rustc base_64.rs && echo -n "g" | ./base_64
//
fn main() -> io::Result<()> {
    let mut input = String::with_capacity(1000);
    io::stdin().read_line(&mut input)?;

    let bs: &[u8] = input.as_bytes();
    let mut base64: Vec<char> = Vec::with_capacity(30);

    let mut i: usize = 0;
    while i < bs.len() {
	let mut quantum: u32 = 0; // a quantum is 24 bits (three bytes)
	if bs.len() >= i+3 {
	    // First octet
	    quantum = move_octect(quantum, bs[i+1]);
	    // Second octet
	    quantum = quantum << 6;
	    quantum = move_octect(quantum, bs[i+1]);
	    // Third octet
	    quantum = quantum << 6;
	    quantum = move_octect(quantum, bs[i+2]);

	    // To Base64
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_11111100_00000000_00000000) >> 18) as usize]); // First char
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000011_11110000_00000000) >> 12) as usize]); // Second char
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000000_00001111_11000000) >> 6) as usize]);  // Third char
	    base64.push(BASE64_ALPHABET[ (quantum & 0b00000000_00000000_00000000_00111111) as usize]);        // Fourth char
	} else if bs.len() == i+2 {
	    // First octet
	    quantum = move_octect(quantum, bs[i+0]);
	    // Second octet
	    quantum = quantum << 6;
	    quantum = move_octect(quantum, bs[i+1]);
	    // Third octet (IS EMPTY)

	    // To Base64
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000000_11111100_00000000) >> 10) as usize]); // First char
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000000_00000011_11110000) >> 4) as usize]); // Second char
	    base64.push(BASE64_ALPHABET[((quantum & 0b00000000_00000000_00000000_00001111) << 2) as usize]);  // Third char
	    base64.push(BASE64_ALPHABET_PAD); // Fourth char
	} else if bs.len() == i+1 {
	    // First octet
	    quantum = move_octect(quantum, bs[i+0]);
	    // Second octet (IS EMPTY)
	    // Third octet (IS EMPTY)

	    // To Base64
	    base64.push(BASE64_ALPHABET[((quantum &  0b00000000_00000000_00000000_11111100) >> 2) as usize]); // First char
	    base64.push(BASE64_ALPHABET[((quantum &  0b00000000_00000000_00000000_00000011) << 4) as usize]); // Second char
	    base64.push(BASE64_ALPHABET_PAD); // Third char
	    base64.push(BASE64_ALPHABET_PAD); // Fourth char
	}

	i += 3; // Jump to next input quantum
    }

    for c in base64 {
	print!("{}", c);
    }

    Ok(())
}

const BASE64_ALPHABET_PAD: char = '=';

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

fn move_octect(quantum: u32, b: u8) -> u32 {
    let q = quantum | (b >> 2) as u32; // Higher 6 bits
    (q << 2) | ((b << 6) >> 6) as u32  // Lower 2 bits 
}

