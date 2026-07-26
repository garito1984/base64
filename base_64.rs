
fn main() {
    let l = "g".as_bytes();

    let mut base64: Vec<char> = Vec::with_capacity(4 * 20);

    let mut i: usize = 0;
    while i < l.len() {
	if l.len() == i+1 {
	    // The final quantum is exactly 8 bits.
	    // Final unit of encoded output will be two characters followed
	    // by two "=" padding characters.
	    base64.push(BASE64_ALPHABET[(l[i] >> 2) as usize]);
	    base64.push(BASE64_ALPHABET[((l[i] << 6) >> 2) as usize]);
	    base64.push(BASE64_ALPHABET_PAD);
	    base64.push(BASE64_ALPHABET_PAD);
	}

	i += 1;
    }

    for c in base64 {
	print!("{}", c);
    }
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
