mod base64;
mod config;

use std::io;
use std::fs;

//
// Based on RFC 4648: The Base16, Base32, and Base64 Data Encodings
//
// Here https://datatracker.ietf.org/doc/html/rfc4648
//
//
// Test with command:
// echo -n "g" | ./main | base64 -d
//
fn main() -> Result<(), io::Error> {
     let (width, is_url, filename) = config::args();

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
