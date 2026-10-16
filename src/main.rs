mod base64;

use std::io;
use std::fs;
use std::collections::{HashMap};

//
// Based on RFC 4648: The Base16, Base32, and Base64 Data Encodings
//
// Here https://datatracker.ietf.org/doc/html/rfc4648
//
// cargo run -- Cargo.toml
//

const DEFAULT_LINE_WIDTH: usize = 76;

//
// Test with command:
// rustc base_64.rs && echo -n "g" | ./base_64 | base64 -d
//
fn main() -> Result<(), io::Error> {
    let (width, is_url, filename) = args();

    let mut input: Box<dyn io::Read> = match filename {
	Some(filename) => Box::new(fs::File::open(filename)?),
	_ => Box::new(io::stdin())
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
const ARG_WIDTH: &str = "-w";
const ARG_ALPHABET_URL: &str = "--url";
enum AppArg {
    Flag(bool),
    NumArg(String, usize),
    _StrArg(String, Option<String>),
    _FileArg(String, String),
    None
}

fn args<'a>() -> (usize, bool, Option<String>) {
    // Load valid arguments with default values if any apply
    let mut valid_args: HashMap<String, AppArg> = HashMap::from([
	(ARG_WIDTH.to_string(), AppArg::NumArg(ARG_WIDTH.to_string(), DEFAULT_LINE_WIDTH)),
	(ARG_ALPHABET_URL.to_string(), AppArg::Flag(false))
    ]);
    
    // Update the configuration if any relevant arguments are given
    let mut prev_arg: AppArg = AppArg::None;
    let mut unknown_args: Vec<String> = Vec::new();
    for arg in std::env::args() {
	if let AppArg::None = prev_arg {
	    match valid_args.get(&arg) {
		Some(AppArg::Flag(_)) => {
		    valid_args.insert(arg.clone(), AppArg::Flag(true));
		},
		Some(AppArg::NumArg(_, val)) => {
		    prev_arg = AppArg::NumArg(arg, *val);
		},
		Some(AppArg::_StrArg(_, _)) => {
		    prev_arg = AppArg::_StrArg(arg, None);
		},
		_ => {
		    unknown_args.push(arg)
		}
	    }
	} else {
	    // Previously there was a parameterized arg, now we get the value
	    match prev_arg {
		AppArg::NumArg(s, _) => {
		    valid_args.insert(s.clone(), AppArg::NumArg(s.clone(), arg.parse().expect("Width should be a number")));
		},
		AppArg::_StrArg(s, _) => {
		    valid_args.insert(s.clone(), AppArg::_StrArg(s.clone(), Some(arg)));
		},
		_ => {}
	    };
	    prev_arg = AppArg::None;
	}
    }

    // Extract the final app configuration
    let Some(AppArg::NumArg(_, width)) = valid_args.get(ARG_WIDTH) else {
	panic!("The width argument requires a parameter")
    };
    let is_url = match valid_args.get(ARG_ALPHABET_URL) {
	Some(AppArg::Flag(true)) => true,
	_ => false
    };
    let filename = match std::env::args().count() {
	1.. => {
	    let last_unk_arg = unknown_args.last().map(String::to_string);
	    let last_env_arg = std::env::args().last();
	    
	    (last_unk_arg == last_env_arg).then_some(last_unk_arg).flatten()
	},
	_ => None
    };

    // Return the config as a tuple
    (*width, is_url, filename)
}

