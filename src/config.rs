use std::str::{FromStr};

const ARG_WIDTH: &str = "-w";
const ARG_ALPHABET_URL: &str = "--url";

const ARG_WIDTH_ERROR_MSG: &str = "Width argument has to be a number";

//
// Parse application arguments
//

const DEFAULT_LINE_WIDTH: usize = 76;

pub fn args() -> Result<(usize, bool, Option<String>), String> {
    arguments(std::env::args())
}

fn arguments<I>(env_args: I) -> Result<(usize, bool, Option<String>), String>
where
    I: IntoIterator<Item = String>,
 {
    // Default config
    let mut url = false;
    let mut width: usize = DEFAULT_LINE_WIDTH;
    let mut unknown: Option<String> = None;
    
    // Update the configuration if any relevant arguments are given
    let mut partial: Option<String> = None;
    for arg in env_args.into_iter().skip(1) {
	let reset_partial = partial.is_some();
	match (arg.as_str(), partial.as_deref()) {
	    // Url
	    (ARG_ALPHABET_URL, _) => url = true,
	    // Width
	    (ARG_WIDTH, _) => partial = Some(arg),
	    (_, Some(ARG_WIDTH)) => {
		match usize::from_str(&arg) {
		    Ok(w) => width = w,
		    Err(_) => return Err(ARG_WIDTH_ERROR_MSG.to_string()) 
		}
	    },
	    // No matching option
	    _ => unknown = Some(arg)
	}

	if reset_partial {
	    partial = None
	}
    }

    Ok((width, url, unknown))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_no_args() {
	let args = vec![
            String::from("./pretend-there-is-an-executable")
	];
        assert_eq!(Ok((DEFAULT_LINE_WIDTH, false, None)), arguments(args));
    }

    #[test]
    fn parse_url() {
	let args = vec![
            String::from("./pretend-this-is-an-executable"),
            String::from(ARG_ALPHABET_URL)
	];
        assert_eq!(Ok((DEFAULT_LINE_WIDTH, true, None)), arguments(args));
    }

    #[test]
    fn parse_width() {
	let args = vec![
            String::from("./pretend-this-is-an-executable"),
            String::from(ARG_WIDTH),
	    String::from("99")
	];
        assert_eq!(Ok((99 as usize, false, None)), arguments(args));
    }
    
    #[test]
    fn parse_width_filename() {
	let args = vec![
            String::from("./pretend-this-is-an-executable"),
            String::from(ARG_WIDTH),
	    String::from("99"),
	    String::from("a-file.txt")
	];
        assert_eq!(Ok((99 as usize, false, Some("a-file.txt".to_string()))), arguments(args));
    }

    #[test]
    fn parse_width_url_filename() {
	let args = vec![
            String::from("./pretend-this-is-an-executable"),
            String::from(ARG_WIDTH),
	    String::from("99"),
	    String::from(ARG_ALPHABET_URL),
	    String::from("a-file.txt")
	];
        assert_eq!(Ok((99 as usize, true, Some("a-file.txt".to_string()))), arguments(args));
    }

    #[test]
    fn parse_width_bad_arg() {
	let args = vec![
            String::from("./pretend-this-is-an-executable"),
            String::from(ARG_WIDTH),
	    String::from("abc") // This will provoque a failure
	];
        assert_eq!(Err(ARG_WIDTH_ERROR_MSG.to_string()), arguments(args));
    }
}
