
use std::io::{Read};
use std::io;

// Test with command:
// rustc base_64.rs && echo -n "g" | ./base_64 | base64 -d
//
fn main() -> Result<(), io::Error> {
    let mut rb = ReadBuffer::new(DEFAULT_LINE_WIDTH);
    let mut base64: Vec<char> = Vec::with_capacity(rb.buf.len() + 500);

    loop {
	let size = rb.read(&mut io::stdin())?;
	if size == 0 {
	    break;
	}

	encode(&mut rb, &mut base64);

	let mut count = 0;
	for c in &base64 {
	    print!("{}", c);
	    count += 1;
	    if count == rb.line_width {
		print!("\n");
		count = 0;
	    }
	}

	// Clear for next iteration
	base64.clear();
    }

    Ok(())
}

struct ReadBuffer {
    buf: Box<[u8]>,
    line_width: usize,
    read_size: usize
}

impl ReadBuffer {
    fn new(line_width: usize) -> ReadBuffer {
	ReadBuffer {
	    buf: vec![0; INPUT_BUFFER_SIZE * BASE64_QUANTUM_BYTE_SIZE * line_width].into_boxed_slice(),
	    line_width: line_width,
	    read_size: 0
	}
    }

    fn read(&mut self, input: &mut dyn Read) -> Result<usize, io::Error> {
	let size = input.read(&mut self.buf)?;
	self.read_size = size;
	Ok(size)
    }
}

fn encode(buffer_read: &mut ReadBuffer, base64: &mut Vec<char>) {
    let mut i: usize = 0;
    while i < buffer_read.read_size {
	let mut quantum: u32 = 0; // a quantum is 24 bits (three bytes)
	if buffer_read.read_size >= i+3 {
	    // First octet
	    quantum = copy_octect(quantum, buffer_read.buf[i+0]);
	    // Second octet
	    quantum = copy_octect(quantum, buffer_read.buf[i+1]);
	    // Third octet
	    quantum = copy_octect(quantum, buffer_read.buf[i+2]);

	    // To Base64
	    add_chars(
		base64,
		(quantum & 0b00000000_11111100_00000000_00000000) >> 18, // First char
		(quantum & 0b00000000_00000011_11110000_00000000) >> 12, // Second char
		(quantum & 0b00000000_00000000_00001111_11000000) >> 6,  // Third char
		 quantum & 0b00000000_00000000_00000000_00111111         // Fourth char
	    );
	} else if buffer_read.read_size == i+2 {
	    // First octet
	    quantum = copy_octect(quantum, buffer_read.buf[i+0]);
	    // Second octet
	    quantum = copy_octect(quantum, buffer_read.buf[i+1]);
	    // Third octet (IS EMPTY)

	    // To Base64
	    add_chars(
		base64,
		(quantum & 0b00000000_00000000_11111100_00000000) >> 10, // First char
		(quantum & 0b00000000_00000000_00000011_11110000) >> 4,  // Second char
		(quantum & 0b00000000_00000000_00000000_00001111) << 2,  // Third char
		BASE64_ALPHABET_PAD_INDEX                                // Fourth char
	    );
	} else if buffer_read.read_size == i+1 {
	    // First octet
	    quantum = copy_octect(quantum, buffer_read.buf[i+0]);
	    // Second octet (IS EMPTY)
	    // Third octet (IS EMPTY)

	    // To Base64
	    add_chars(
		base64,
		(quantum & 0b00000000_00000000_00000000_11111100) >> 2, // First char
		(quantum & 0b00000000_00000000_00000000_00000011) << 4, // Second char
		BASE64_ALPHABET_PAD_INDEX,                              // Third char
		BASE64_ALPHABET_PAD_INDEX                               // Fourth char
	    );
	}

	i += 3; // Jump to next input quantum
    }
}

const BASE64_ALPHABET_PAD: char = '=';
const BASE64_ALPHABET_PAD_INDEX: u32 = 0b0100_0000; // 65 or padding char

const BASE64_QUANTUM_BYTE_SIZE: usize = 3;
const INPUT_BUFFER_SIZE: usize = 100;

const DEFAULT_LINE_WIDTH: usize = 76;

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

//
// Shift the quantum 8 bits to the left and copy the byte into the lower bits
//
// 0b00000000_00000000_00000000_11111111 -> 0b00000000_00000000_11111111_10111011
//
fn copy_octect(quantum: u32, byte: u8) -> u32 {
    (quantum << 8) | byte as u32
}

fn add_chars(
    base64out: &mut Vec<char>,
    fst: u32,
    snd: u32,
    trd: u32,
    frt: u32
) {
    for index in [fst, snd, trd, frt] {
	base64out.push(BASE64_ALPHABET[index as usize]);
    }
}
