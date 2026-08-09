
use std::io::{Read};
use std::io;
use std::collections::{HashMap};

//
// Based on RFC 4648: The Base16, Base32, and Base64 Data Encodings
//
// Here https://datatracker.ietf.org/doc/html/rfc4648
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

//
// Test with command:
// rustc base_64.rs && echo -n "g" | ./base_64 | base64 -d
//
fn main() -> Result<(), io::Error> {
    let (arg_width, url_alphabet) = args();

    let mut buf = Base64Buffer::new(arg_width, url_alphabet);
    let mut base64: Vec<char> = Vec::with_capacity(buf.buf.len() + 500);

    while buf.read(&mut io::stdin())? > 0 {
	// Process
	buf.encode(&mut base64);

	// Print the encoded version
	buf.print(&base64);

	// Clear for next iteration
	base64.clear();
    }

    Ok(())
}

const ARG_WIDTH: &str = "-w";
const ARG_ALPHABET_URL: &str = "--url";
enum AppArg {
    Flag(String, bool),
    NumArg(String, usize),
    StrArg(String, Option<String>),
    _FileArg(String, String),
    None
}

fn args<'a>() -> (usize, &'a[char]) {
    // Load valid arguments with default values if any apply
    let mut valid_args: HashMap<String, AppArg> = HashMap::from([
	(ARG_WIDTH.to_string(), AppArg::NumArg(ARG_WIDTH.to_string(), DEFAULT_LINE_WIDTH)),
	(ARG_ALPHABET_URL.to_string(), AppArg::Flag(ARG_ALPHABET_URL.to_string(), false))
    ]);
    
    // Update the configuration if any relevant arguments are given
    let mut prev_arg: AppArg = AppArg::None;
    for arg in std::env::args() {
	if let AppArg::None = prev_arg {
	    match valid_args.get(&arg) {
		Some(AppArg::Flag(_, _)) => {
		    valid_args.insert(arg.clone(), AppArg::Flag(arg.clone(), true));
		},
		Some(AppArg::NumArg(_, val)) => {
		    prev_arg = AppArg::NumArg(arg, *val);
		},
		Some(AppArg::StrArg(_, _)) => {
		    prev_arg = AppArg::StrArg(arg, None);
		},
		_ => {
		    //eprintln!("Unknown argument: {}", arg);
		}
	    }
	} else {
	    // Previously there was a parameterized arg, now we get the value
	    match prev_arg {
		AppArg::NumArg(s, _) => {
		    valid_args.insert(s.clone(), AppArg::NumArg(s.clone(), arg.parse().expect("Width should be a number")));
		},
		AppArg::StrArg(s, _) => {
		    valid_args.insert(s.clone(), AppArg::StrArg(s.clone(), Some(arg)));
		},
		_ => {}
	    };
	    prev_arg = AppArg::None;
	}
    }

    // Return the results as a tuple
    let Some(AppArg::NumArg(_, width)) = valid_args.get(ARG_WIDTH) else {
	panic!("The width argument requires a parameter")
    };
    let Some(AppArg::Flag(_, is_url)) = valid_args.get(ARG_ALPHABET_URL) else {
	panic!("An error took place with url alphabet argument");
    };
    (*width, if *is_url {&BASE64_ALPHABET_URL} else {&BASE64_ALPHABET})
}

struct Base64Buffer<'a> {
    buf: Box<[u8]>,
    line_width: usize,
    read_size: usize,
    alphabet: &'a[char]
}

impl<'a> Base64Buffer<'a> {
    fn new(line_width: usize, alphabet: &'a[char]) -> Base64Buffer<'a> {
	let line_width_non_zero = match line_width {
	    0 => DEFAULT_LINE_WIDTH,
	    _ => line_width
	};
	Base64Buffer {
	    buf: vec![0; INPUT_BUFFER_SIZE * BASE64_QUANTUM_BYTE_SIZE * line_width_non_zero].into_boxed_slice(),
	    line_width: line_width,
	    read_size: 0,
	    alphabet: alphabet
	}
    }

    fn encode(&mut self, base64: &mut Vec<char>) {
	let mut i: usize = 0;
	while i < self.read_size {
	    let (fst, snd, trd, frt) = match self.read_size {
		val if val >= i+3 => {
		    // First octet
		    let mut quantum = copy_octect(0, self.buf[i+0]);
		    // Second octet
		    quantum = copy_octect(quantum, self.buf[i+1]);
		    // Third octet
		    quantum = copy_octect(quantum, self.buf[i+2]);

		    // To Base64
		    (
			(quantum & 0b00000000_11111100_00000000_00000000) >> 18,
			(quantum & 0b00000000_00000011_11110000_00000000) >> 12,
			(quantum & 0b00000000_00000000_00001111_11000000) >> 6,
			quantum & 0b00000000_00000000_00000000_00111111
		    )
		},
		val if val == i+2 => {
		    // First octet
		    let mut quantum = copy_octect(0, self.buf[i+0]);
		    // Second octet
		    quantum = copy_octect(quantum, self.buf[i+1]);
		    // Third octet (IS EMPTY)

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

    fn read(&mut self, input: &mut dyn Read) -> Result<usize, io::Error> {
	let size = input.read(&mut self.buf)?;
	self.read_size = size;
	Ok(size)
    }

    fn print(&self, base64: &Vec<char>) {
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
}

//
// Shift the quantum 8 bits to the left and copy the byte into the lower bits
//
// 0b00000000_00000000_00000000_11111111 -> 0b00000000_00000000_11111111_10111011
//
fn copy_octect(quantum: u32, byte: u8) -> u32 {
    (quantum << 8) | byte as u32
}
