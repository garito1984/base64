
use std::io;

//
// Based on RFC 4648: The Base16, Base32, and Base64 Data Encodings
//
// Here https://datatracker.ietf.org/doc/html/rfc4648
//
// cargo run -- Cargo.toml
//

const INPUT_BUFFER_SIZE: usize = 10;
const DEFAULT_LINE_WIDTH: usize = 76;

const BASE64_QUANTUM_BYTE_SIZE: usize = 3;

const BASE64_ALPHABET_PAD: char = '=';
const BASE64_ALPHABET_PAD_INDEX: u32 = 0b0100_0000; // 65 or padding char
const BASE64_ALPHABET: [char; 65] = [
    // Upper case characters
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    // Lower case characters
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    // Numbers
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    // Special characters
    '+', '/', BASE64_ALPHABET_PAD
];
const BASE64_ALPHABET_URL: [char; 65] = [
    // Upper case characters
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    // Lower case characters
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    // Numbers
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    // Special characters
    '-', '_', BASE64_ALPHABET_PAD
];

pub struct Buffer<'a> {
    buf: Box<[u8]>,
    line_width: usize,
    read_size: usize,
    alphabet: &'a[char]
}

impl<'a> Buffer<'a> {
    pub fn new(line_width: usize, is_url: bool) -> Buffer<'static> {
	let line_width_non_zero = match line_width {
	    0 => DEFAULT_LINE_WIDTH,
	    _ => line_width
	};

	Buffer {
	    buf: vec![0; INPUT_BUFFER_SIZE * BASE64_QUANTUM_BYTE_SIZE * line_width_non_zero].into_boxed_slice(),
	    line_width: line_width,
	    read_size: 0,
	    alphabet: if is_url { &BASE64_ALPHABET_URL } else { &BASE64_ALPHABET }
	}
    }

    pub fn encode(&mut self, base64: &mut Vec<char>) {
	let mut i: usize = 0;
	while i < self.read_size {
	    let (fst, snd, trd, frt) = match self.read_size {
		val if val >= i+3 => {
		    
		    let quantum = {
			// First octet
			let mut q = copy_octect(0, self.buf[i+0]);
			// Second octet
			q = copy_octect(q, self.buf[i+1]);
			// Third octet
			copy_octect(q, self.buf[i+2])
		    };

		    // To Base64
		    (
			(quantum & 0b00000000_11111100_00000000_00000000) >> 18,
			(quantum & 0b00000000_00000011_11110000_00000000) >> 12,
			(quantum & 0b00000000_00000000_00001111_11000000) >> 6,
			quantum & 0b00000000_00000000_00000000_00111111
		    )
		},
		val if val == i+2 => {
		    
		    let quantum = {
			// First octet
			let q = copy_octect(0, self.buf[i+0]);
			// Second octet
			copy_octect(q, self.buf[i+1])
			// Third octet (IS EMPTY)
		    };

		    // To Base64
		    (
			(quantum & 0b00000000_00000000_11111100_00000000) >> 10,
			(quantum & 0b00000000_00000000_00000011_11110000) >> 4,
			(quantum & 0b00000000_00000000_00000000_00001111) << 2,
			BASE64_ALPHABET_PAD_INDEX
		    )
		},
		val if val == i+1 => {
		    // First octet
		    let quantum = copy_octect(0, self.buf[i+0]);
		    // Second octet (IS EMPTY)
		    // Third octet (IS EMPTY)

		    // To Base64
		    (
			(quantum & 0b00000000_00000000_00000000_11111100) >> 2,
			(quantum & 0b00000000_00000000_00000000_00000011) << 4,
			BASE64_ALPHABET_PAD_INDEX,
			BASE64_ALPHABET_PAD_INDEX
		    )
		},
		_ => {
		    panic!("This should never happen");
		}
	    };

	    for index in [fst, snd, trd, frt] {
		base64.push(self.alphabet[index as usize]);
	    }

	    i += 3; // Jump to next input quantum
	}
    }

    pub fn read(&mut self, input: &mut dyn io::Read) -> Result<usize, io::Error> {
	let size = input.read(&mut self.buf)?;
	self.read_size = size;
	Ok(size)
    }

    pub fn print(&self, base64: &Vec<char>) {
	let disable_new_lines = 0 == self.line_width;
	let mut count = 0;
	for c in base64 {
	    print!("{}", c);

	    if disable_new_lines {
		continue;
	    }
	    
	    count += 1;
	    if count == self.line_width {
		print!("\n");
		count = 0;
	    }
	}
    }

    pub fn get_buf_len(&self) -> usize {
	self.buf.len()
    }

}

//
// Shift the quantum 8 bits to the left and copy the byte into the lower bits
//
// 0b00000000_00000000_00000000_11111111 -> 0b00000000_00000000_11111111_10111011
//
fn copy_octect(quantum: u32, byte: u8) -> u32 {
    (quantum << 8) | byte as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH_TEST_VALUE: usize = 80;

    // BASE64("") = ""
    #[test]
    fn encode_empty() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

	let s: String = base64.iter().collect();
	assert_eq!("", s);
	std::assert_matches!(r, Ok::<usize, io::Error>(0));
    }

    // BASE64("f") = "Zg=="
    #[test]
    fn encode_f() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("f".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("Zg==", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(1));
    }

    // BASE64("fo") = "Zm8="
    #[test]
    fn encode_fo() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("fo".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("Zm8=", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(2));
    }

    // BASE64("foo") = "Zm9v"
    #[test]
    fn encode_foo() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("foo".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("Zm9v", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(3));
    }

    // BASE64("foob") = "Zm9vYg=="
    #[test]
    fn encode_foob() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("foob".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("Zm9vYg==", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(4));
    }

    // BASE64("fooba") = "Zm9vYmE="
    #[test]
    fn encode_fooba() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("fooba".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("Zm9vYmE=", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(5));
    }

    // BASE64("foobar") = "Zm9vYmFy"
    #[test]
    fn encode_foobar() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("foobar".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("Zm9vYmFy", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(6));
    }

    // BASE64("name)?") = bmFtZSk/
    #[test]
    fn encode_ends_with_special_char_forward_slash() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("name)?".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("bmFtZSk/", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(6));
    }

    // BASE64("? > 0 ") = PyA+IDAg
    #[test]
    fn encode_ends_with_special_char_plus_sign() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
	let mut input: Box<dyn std::io::Read> = Box::new("? > 0 ".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("PyA+IDAg", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(6));
    }

    //
    // Test the --url switch that basically only affects
    // special chars, characters and numbers remain the same, this changes:
    // 1. '/' -> '_'
    // 2. '+' -> '-'
    //

    // URL_BASE64("name)?") = bmFtZSk_
    #[test]
    fn url_encode_ends_with_special_underscore_char() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, true);
	let mut input: Box<dyn std::io::Read> = Box::new("name)?".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("bmFtZSk_", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(6));
    }

    // URL_BASE64("? > 0 ") = PyA-IDAg
    #[test]
    fn url_encode_ends_with_special_dash_char() {
	let mut buf = Buffer::new(WIDTH_TEST_VALUE, true);
	let mut input: Box<dyn std::io::Read> = Box::new("? > 0 ".as_bytes());
	let r = buf.read(&mut input);

	let mut base64: Vec<char> = Vec::new();
	buf.encode(&mut base64);

        assert_eq!("PyA-IDAg", base64.iter().collect::<String>());
	std::assert_matches!(r, Ok::<usize, io::Error>(6));
    }

}
