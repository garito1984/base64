mod base64;

use std::io;
use std::fs;

//
// Based on RFC 4648: The Base16, Base32, and Base64 Data Encodings
//
// Here https://datatracker.ietf.org/doc/html/rfc4648
//

const ARG_WIDTH: &str = "-w";
const ARG_ALPHABET_URL: &str = "--url";

//
// Test with command:
// echo -n "g" | ./main | base64 -d
//
fn main() -> Result<(), io::Error> {
    let (width, is_url, filename) = args();

    let mut input: Box<dyn io::Read> = match filename {
	Some(filename) => Box::new(fs::File::open(filename)?),
	None => Box::new(io::stdin())
    };

    let mut buf = base64::Base64Buffer::new(width, is_url);
    let mut base64: Vec<char> = Vec::with_capacity(buf.get_buf_len() + 500);

    while buf.read(&mut input)? > 0 {
	// Process
	buf.encode(&mut base64);

	// Print the encoded version
	buf.print(&base64);

	// Clear for next iteration
	base64.clear();
    }

    Ok(())
}

//
// Parse application arguments
//

const DEFAULT_LINE_WIDTH: usize = 76;

fn args() -> (usize, bool, Option<String>) {
    // Default config
    let mut url = false;
    let mut width: usize = DEFAULT_LINE_WIDTH;
    let mut unknown: Option<String> = None;
    
    // Update the configuration if any relevant arguments are given
    let mut partial: Option<String> = None;
    for arg in std::env::args() {
	let reset_partial = partial.is_some();
	match (arg.as_str(), partial.as_deref()) {
	    // Url
	    (ARG_ALPHABET_URL, _) => url = true,
	    // Width
	    (ARG_WIDTH, _) => partial = Some(arg),
	    (_, Some(ARG_WIDTH)) => width = arg.parse().expect("Width should be a number"),
	    // No matching option
	    _ => unknown = Some(arg)
	}

	if reset_partial {
	    partial = None
	}
    }

    let filename = match std::env::args().count() {
	1.. => unknown,
	_ => None
    };

    (width, url, filename)
}

