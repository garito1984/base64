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

const BASE64_ALPHABET_PAD_INDEX: u8 = 0b0100_0000; // 65 or padding char
const BASE64_ALPHABET: &[u8] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=".as_bytes();
const BASE64_ALPHABET_URL: &[u8] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_=".as_bytes();

pub struct Buffer<'a> {
    buf: Box<[u8]>,
    line_width: usize,
    read_size: usize,
    alphabet: &'a [u8],
}

impl<'a> Buffer<'a> {
    pub fn new(line_width: usize, is_url: bool) -> Buffer<'static> {
        let line_width_non_zero = match line_width {
            0 => DEFAULT_LINE_WIDTH,
            _ => line_width,
        };

        Buffer {
            buf: vec![0; INPUT_BUFFER_SIZE * BASE64_QUANTUM_BYTE_SIZE * line_width_non_zero]
                .into_boxed_slice(),
            line_width,
            read_size: 0,
            alphabet: if is_url {
                BASE64_ALPHABET_URL
            } else {
                BASE64_ALPHABET
            },
        }
    }

    pub fn encode(&mut self, base64: &mut Vec<u8>) {
        let (quantums, remainder) = self.get_buf_read_slice().as_chunks::<3>();
        for quantum in quantums {
            let encoded = [
                // 0b11111100
                quantum[0] >> 2,
                // 0b00000011_11110000
                0b0011_1111 & ((quantum[0] << 4) | (quantum[1] >> 4)),
                // 0b00001111_11000000
                0b0011_1111 & ((quantum[1] << 2) | (quantum[2] >> 6)),
                // 0b00111111
                0b0011_1111 & quantum[2],
            ];

            for el in encoded {
                base64.push(self.alphabet[el as usize]);
            }
        }

        if !remainder.is_empty() {
            let encoded = match remainder.len() {
                2 => [
                    remainder[0] >> 2,
                    0b0011_1111 & ((remainder[0] << 4) | (remainder[1] >> 4)),
                    0b0011_1111 & (remainder[1] << 2),
                    BASE64_ALPHABET_PAD_INDEX,
                ],
                1 => [
                    remainder[0] >> 2,
                    0b0011_1111 & (remainder[0] << 4),
                    BASE64_ALPHABET_PAD_INDEX,
                    BASE64_ALPHABET_PAD_INDEX,
                ],
                val => {
                    panic!("There can only be 1 or 2 octets, but there was: {}", val);
                }
            };

            for el in encoded {
                base64.push(self.alphabet[el as usize]);
            }
        }
    }

    pub fn read(&mut self, input: &mut dyn io::Read) -> Result<usize, io::Error> {
        let size = input.read(&mut self.buf)?;
        self.read_size = size;
        Ok(size)
    }

    pub fn print(&self, base64: &[u8]) {
        use std::io::{BufWriter, Stdout, Write};

        const ERR_WRITE_MSG: &str = "Unexpected error writing the encoding";

        let enable_line_breaks = self.line_width != 0;
        let line_width = match self.line_width {
            0 => base64.len(),
            val => val,
        };

        let mut start: usize = 0;
        let mut end: usize = line_width;
        let mut writer: BufWriter<Stdout> =
            BufWriter::with_capacity(base64.len(), std::io::stdout());
        while start < base64.len() {
            if end > base64.len() {
                end = base64.len();
            }
            let _ = writer.write(&base64[start..end]).expect(ERR_WRITE_MSG);

            if enable_line_breaks && ((end - start) == line_width) {
                let _ = writer.write("\n".as_bytes()).expect(ERR_WRITE_MSG);
            }

            start += line_width;
            end += line_width;
        }

        std::io::stdout()
            .flush()
            .expect("Unexpected error flushing the encoded output");
    }

    pub fn get_buf_len(&self) -> usize {
        self.buf.len()
    }

    fn get_buf_read_slice(&self) -> &[u8] {
        &self.buf[..self.read_size]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;
    use std::str::Utf8Error;

    const WIDTH_TEST_VALUE: usize = 80;

    // BASE64("") = ""
    #[test]
    fn encode_empty() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(0));
        Ok(())
    }

    // BASE64("f") = "Zg=="
    #[test]
    fn encode_f() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("f");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("Zg==", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(1));
        Ok(())
    }

    // BASE64("fo") = "Zm8="
    #[test]
    fn encode_fo() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("fo");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("Zm8=", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(2));
        Ok(())
    }

    // BASE64("foo") = "Zm9v"
    #[test]
    fn encode_foo() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("foo");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("Zm9v", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(3));
        Ok(())
    }

    // BASE64("foob") = "Zm9vYg=="
    #[test]
    fn encode_foob() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("foob");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("Zm9vYg==", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(4));
        Ok(())
    }

    // BASE64("fooba") = "Zm9vYmE="
    #[test]
    fn encode_fooba() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("fooba");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("Zm9vYmE=", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(5));
        Ok(())
    }

    // BASE64("foobar") = "Zm9vYmFy"
    #[test]
    fn encode_foobar() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("foobar");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("Zm9vYmFy", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(6));
        Ok(())
    }

    // BASE64("name)?") = bmFtZSk/
    #[test]
    fn encode_ends_with_special_char_forward_slash() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("name)?");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("bmFtZSk/", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(6));
        Ok(())
    }

    // BASE64("? > 0 ") = PyA+IDAg
    #[test]
    fn encode_ends_with_special_char_plus_sign() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, false);
        let mut input = to_read("? > 0 ");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("PyA+IDAg", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(6));
        Ok(())
    }

    //
    // Test the --url switch that basically only affects
    // special chars, characters and numbers remain the same, this changes:
    // 1. '/' -> '_'
    // 2. '+' -> '-'
    //

    // URL_BASE64("name)?") = bmFtZSk_
    #[test]
    fn url_encode_ends_with_special_underscore_char() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, true);
        let mut input = to_read("name)?");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("bmFtZSk_", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(6));
        Ok(())
    }

    // URL_BASE64("? > 0 ") = PyA-IDAg
    #[test]
    fn url_encode_ends_with_special_dash_char() -> Result<(), Utf8Error> {
        let mut buf = Buffer::new(WIDTH_TEST_VALUE, true);
        let mut input = to_read("? > 0 ");
        let size = buf.read(&mut input);

        let mut base64: Vec<u8> = Vec::new();
        buf.encode(&mut base64);

        assert_eq!("PyA-IDAg", std::str::from_utf8(&base64)?);
        assert_matches!(size, Ok::<usize, io::Error>(6));
        Ok(())
    }

    fn to_read<'a>(s: &'a str) -> Box<dyn io::Read + 'a> {
        Box::new(s.as_bytes())
    }
}
