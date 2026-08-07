
use std::io::{Read, Write};
use std::io;

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

//
// Test with command:
// rustc base_64.rs && echo -n "g" | ./base_64 | base64 -d
//
fn main() -> Result<(), io::Error> {
    
    let mut line_width_flag = false;
    let mut line_width_value: Option<usize> = None;
    for arg in std::env::args() {
	if arg == "-w" {
	    line_width_flag = true;
	} else if line_width_flag {
	    line_width_value = Some(arg.parse::<usize>().unwrap());
	    break;
	}
    }

    let mut buf = Base64Buffer::new(line_width_value.unwrap_or(DEFAULT_LINE_WIDTH));
    let mut base64: Vec<char> = Vec::with_capacity(buf.buf.len() + 500);

    while buf.read(&mut io::stdin())? > 0 {
	// Process
	buf.encode(&mut base64);

	// Print the encoded version
	buf.print(&base64)?;

	// Clear for next iteration
	base64.clear();
    }

    Ok(())
}

struct Base64Buffer {
    buf: Box<[u8]>,
    line_width: usize,
    read_size: usize
}

impl Base64Buffer {
    fn new(line_width: usize) -> Base64Buffer {
	let line_width_non_zero = match line_width {
	    0 => DEFAULT_LINE_WIDTH,
	    _ => line_width
	};
	Base64Buffer {
	    buf: vec![0; INPUT_BUFFER_SIZE * BASE64_QUANTUM_BYTE_SIZE * line_width_non_zero].into_boxed_slice(),
	    line_width: line_width,
	    read_size: 0
	}
    }

    fn encode(&mut self, base64: &mut Vec<char>) {
	let mut fst: u32 = 0;
	let mut snd: u32 = 0;
	let mut trd: u32 = 0;
	let mut frt: u32 = 0;
	
	let mut i: usize = 0;
	while i < self.read_size {
	    let mut quantum: u32 = 0; // a quantum is 24 bits (three bytes)
	    if self.read_size >= i+3 {
		// First octet
		quantum = copy_octect(quantum, self.buf[i+0]);
		// Second octet
		quantum = copy_octect(quantum, self.buf[i+1]);
		// Third octet
		quantum = copy_octect(quantum, self.buf[i+2]);

		// To Base64
		fst = (quantum & 0b00000000_11111100_00000000_00000000) >> 18; // First char
		snd = (quantum & 0b00000000_00000011_11110000_00000000) >> 12; // Second char
		trd = (quantum & 0b00000000_00000000_00001111_11000000) >> 6;  // Third char
		frt =  quantum & 0b00000000_00000000_00000000_00111111;        // Fourth char

	    } else if self.read_size == i+2 {
		// First octet
		quantum = copy_octect(quantum, self.buf[i+0]);
		// Second octet
		quantum = copy_octect(quantum, self.buf[i+1]);
		// Third octet (IS EMPTY)

		// To Base64
		fst = (quantum & 0b00000000_00000000_11111100_00000000) >> 10; // First char
		snd = (quantum & 0b00000000_00000000_00000011_11110000) >> 4;  // Second char
		trd = (quantum & 0b00000000_00000000_00000000_00001111) << 2;  // Third char
		frt = BASE64_ALPHABET_PAD_INDEX;                               // Fourth char
	    } else if self.read_size == i+1 {
		// First octet
		quantum = copy_octect(quantum, self.buf[i+0]);
		// Second octet (IS EMPTY)
		// Third octet (IS EMPTY)

		// To Base64
		fst = (quantum & 0b00000000_00000000_00000000_11111100) >> 2; // First char
		snd = (quantum & 0b00000000_00000000_00000000_00000011) << 4; // Second char
		trd = BASE64_ALPHABET_PAD_INDEX;                              // Third char
		frt = BASE64_ALPHABET_PAD_INDEX;                              // Fourth char
	    }

	    for index in [fst, snd, trd, frt] {
		base64.push(BASE64_ALPHABET[index as usize]);
	    }

	    i += 3; // Jump to next input quantum
	}
    }

    fn read(&mut self, input: &mut dyn Read) -> Result<usize, io::Error> {
	let size = input.read(&mut self.buf)?;
	self.read_size = size;
	Ok(size)
    }

    fn print(&self, base64: &Vec<char>) -> Result<(), io::Error> {
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

	// Make sure the buffer is flushed
	io::stdout().flush()?;
	Ok(())
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
