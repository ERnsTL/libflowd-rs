#![feature(generators, generator_trait)]
#![feature(test)]
extern crate test;

#[macro_use]
extern crate nom;
//extern crate flavors;
extern crate circular;

mod flowd {
	use std::io::{BufRead, Error, Read};
	use std::result::*;
	use std::str;

	use std::fs::File;
	use std::ops::{Generator, GeneratorState};

	//use flavors::parser::header;
	use circular::Buffer;
	use nom::types::{CompleteByteSlice, CompleteStr};
	use nom::{alphanumeric, anychar, char, line_ending, newline};
	use nom::{
		AsBytes, AsChar, Err, FindToken, HexDisplay, InputLength, InputTakeAtPosition, Needed,
		Offset,
	};

	//mod types;

	//use types::*;

	pub struct IP2<'b> {
		pub frame_type: &'b str,
		pub body_type: &'b str,
		pub port: &'b str,
		pub headers: Vec<(String, String)>,
		pub body: &'b [u8],
	}

	pub struct Parser {
		capacity: usize,
		buf: Buffer,
	}

	impl Parser {
		//pub fn new<T: BufReader<u8> + Read + BufRead>(mut reader: T) -> Parser {
		//pub fn new<T: Read>(filename: &str) -> Parser {
		//pub fn new(filename: &str) -> Parser {
		pub fn new() -> Parser {
			// circular::Buffer is a ring buffer abstraction that separates reading and consuming data
			// it can grow its internal buffer and move data around if we reached the end of that buffer
			const capacity: usize = 1000;
			let b = Buffer::with_capacity(capacity);

			Parser {
				capacity: capacity,
				buf: b,
			}
		}

		//impl<'a, T: Read> Iterator for Parser<'a, T> {
		// NOTE: ^ is difficult because of half-baked Dynamically Sized Types (DST) support (2018-12)
		pub fn run(self: &mut Parser, filename: &str) -> std::io::Result<()> {
			let mut file = File::open(filename)?;

			// we write into the `&mut[u8]` returned by `space()`
			let sz = file.read(self.buf.space()).expect("should write");
			self.buf.fill(sz);
			println!("write {:#?}", sz);

			/*
			let length = {
				// `available_data()` returns how many bytes can be read from the buffer
				// `data()` returns a `&[u8]` of the current data
				// `to_hex(_)` is a helper method of `nom::HexDisplay` to print a hexdump of a byte slice
				println!(
					"data({} bytes):\n{}",
					self.buf.available_data(),
					(&self.buf.data()[..min(self.buf.available_data(), 128)]).to_hex(16)
				);

				// we parse the beginning of the file with `flavors::parser::header`
				// a FLV file is made of a header, then a serie of tags, suffixed by a 4 byte integer (size of previous tag)
				// the file header is also followed by a 4 byte integer size
				let res = header(self.buf.data());
				if let IResult::Done(remaining, h) = res {
					println!("parsed header: {:#?}", h);

					// `offset()` is a helper method of `nom::Offset` that can compare two slices and indicate
					// how far they are from each other. The parameter of `offset()` must be a subset of the
					// original slice
					self.buf.data().offset(remaining)
				} else {
					panic!("couldn't parse header");
				}
			};

			// 4 more bytes for the size of previous tag just after the header
			println!("consumed {} bytes", length + 4);
			self.buf.consume(length + 4);
			*/

			let mut generator = move || {
				println!("entered generator");
				// we will count the number of tag and use that and return value for the generator
				let mut tag_count = 0usize;
				//let mut consumed = length;
				let mut consumed = 0;

				println!("entering reading loop");
				// this is the data reading loop. On each iteration we will read more data, then try to parse
				// it in the inner loop
				loop {
					// refill the buffer
					let sz = file.read(self.buf.space()).expect("should write");
					self.buf.fill(sz);
					println!(
						"refill: {} more bytes, available data: {} bytes, consumed: {} bytes",
						sz,
						self.buf.available_data(),
						consumed
					);

					// if there's no more available data in the buffer after a write, that means we reached
					// the end of the file
					if self.buf.available_data() == 0 {
						println!("no more data to read or parse, stopping the reading loop");
						break;
					}

					let needed: Option<Needed>;

					// this is the parsing loop. After we read some data, we will try to parse from it until
					// we get an error or the parser returns `Incomplete`, indicating it needs more data
					println!("entering parsing loop");
					loop {
						let (length, tag) = {
							//println!("[{}] data({} bytes, consumed {}):\n{}", tag_count,
							//  b.available_data(), consumed, (&b.data()[..min(b.available_data(), 128)]).to_hex(16));

							/*
							let bla = frame_header(self.buf.data());
							let remaining = 0;

							(self.buf.data().offset(remaining), bla)
							*/

							// try to parse a tag
							// the `types::flv_tag` parser combines the tag parsing and consuming the 4 byte integer size
							// following it
							match frame_header(self.buf.data()) {
								Ok((remaining, tag)) => {
									tag_count += 1;

									// tags parsed with flavors contain a slice of the original data. We cannot
									// return that from the generator, since it is borrowed from the Buffer's internal
									// data. Instead, we use the `types::Tag` defined in `src/types.rs` to clone
									// the data

									//let t = Tag::new(tag);
									//(self.buf.data().offset(remaining), t)
									//TODO

									//(self.buf.data().offset(remaining), tag)
									(
										self.buf.data().offset(remaining),
										//([97u8, 98u8, 99u8], [65u8, 66u8, 67u8]),
										(tag.0.to_owned(), tag.1.to_owned()),
									)
								}
								Err(nom::Err::Incomplete(n)) => {
									println!("not enough data, needs a refill: {:?}", n);

									needed = Some(n);
									break;
								}
								Err(nom::Err::Error(e)) | Err(Err::Failure(e)) => {
									panic!("parse error: {:#?}", e);
								}
							}
						};

						println!("{}", tag.1[0]);

						println!(
							"consuming {} of {} bytes",
							length,
							self.buf.available_data()
						);
						self.buf.consume(length);
						consumed += length;

						// give the tag to the calling code. On the next call to the generator's `resume()`,
						// we will continue from the parsing loop, and go on the reading loop's next iteration
						// if necessary
						yield tag;
					}

					// if the parser returned `Incomplete`, and it needs more data than the buffer can hold,
					// we grow the buffer. In a more realistic code, you would define a maximal size to which
					// the buffer can grow, instead of letting the input data drive your programm into OOM death
					if let Some(Needed::Size(sz)) = needed {
						if sz > self.buf.capacity() {
							println!(
								"growing buffer capacity from {} bytes to {} bytes",
								self.capacity,
								self.capacity * 2
							);

							self.capacity *= 2;
							self.buf.grow(self.capacity);
						}
					}
				}

				// we finished looping over the data, return how many tag we parsed
				return tag_count;
			};

			loop {
				unsafe {
					match generator.resume() {
						GeneratorState::Yielded(tag) => {
							/*
							println!(
								"next tag: type={:?}, timestamp={}, size={}",
								tag.header.tag_type, tag.header.timestamp, tag.header.data_size
							);
							*/
							println!(
								"gor parser result: {} = {}",
								tag.0.to_hex(16),
								tag.1.to_hex(16)
							);
						}
						GeneratorState::Complete(tag_count) => {
							println!("parsed {} FLV tags", tag_count);
							break;
						}
					}
				}
			}

			Ok(())
		}
	}

	/*
		named!(header<Header>,
	  do_parse!(
				 take_until(line_ending) >>	//newline
		version: be_u8       >>
		flags:   be_u8       >>
		offset:  be_u32      >>
		(Header {
			version: version,
			audio:   flags & 4 == 4,
			video:   flags & 1 == 1,
			offset:  offset
		})
	  )
	);
	*/

	named!(pub frame_header<(&[u8],&[u8])>,
		terminated!(header_line, newline)
		//do_parse!(header_line)
	);

	/*
	named!(
		header_line<(&[u8], &[u8])>,
		do_parse!(
				k: take_until1!(is_colon) >>
				char!(':') >>
				v: take_until1!(newline) >>
				(k, v)
		)
	);
	*/

	named!(
		header_line<(&[u8], &[u8])>,
		do_parse!(
			kv: separated_pair!(alphanumeric1_noncolon, char!(':'), alphanumeric1_nonnewline)
				>> (kv)
		)
	);

	/// Recognizes one or more numerical and alphabetic characters.
	/// For ASCII strings: 0-9a-zA-Z
	/// For UTF8 strings, 0-9 and any alphabetic code point (ie, not only the ASCII ones)
	pub fn alphanumeric1_noncolon<T>(input: T) -> nom::IResult<T, T>
	where
		T: nom::InputTakeAtPosition,
		<T as InputTakeAtPosition>::Item: AsChar,
	{
		input.split_at_position1(
			|item| {
				let c = item.as_char();
				c == ':'
			},
			nom::ErrorKind::AlphaNumeric,
		)
	}

	/// Recognizes one or more numerical and alphabetic characters.
	/// For ASCII strings: 0-9a-zA-Z
	/// For UTF8 strings, 0-9 and any alphabetic code point (ie, not only the ASCII ones)
	pub fn alphanumeric1_nonnewline<T>(input: T) -> nom::IResult<T, T>
	where
		T: nom::InputTakeAtPosition,
		<T as InputTakeAtPosition>::Item: AsChar,
	{
		input.split_at_position1(
			|item| {
				let c = item.as_char();
				c == '\n'
			},
			nom::ErrorKind::Escaped,
		)
	}

	//named!(printable<&[u8],&[u8]>, take_while1!(is_ascii_vchar));

	/// True if `ch` is ascii and "visible"/"printable".
	///
	/// This is the case for any char in the (decimal)
	/// range 33..=126 which is '!'..='~'.
	#[inline(always)]
	pub fn is_ascii_vchar(ch: char) -> bool {
		ch > 32 as char && ch <= 126 as char
	}

	#[inline(always)]
	pub fn is_ascii_vchar2(ch: u8) -> bool {
		ch > 32 && ch <= 126
	}

	#[inline(always)]
	pub fn is_newline(ch: u8) -> bool {
		ch == 10
	}

	#[inline(always)]
	pub fn is_colon(ch: u8) -> bool {
		ch == 58
	}

	#[inline(always)]
	pub fn is_not_colon(ch: u8) -> bool {
		ch != 58
	}

	//TODO convert to &str with the following
	/*
	fn complete_byte_slice_to_str<'a>(s: CompleteByteSlice<'a>) -> Result<&'a str, str::Utf8Error> {
		str::from_utf8(s.0)
	}
	*/

	const VERSION_TWO: u8 = 0x32; // "2"
	#[allow(dead_code)]
	pub fn parse_frame<T>(mut reader: T) -> Result<IP, Error>
	where
		T: BufRead,
	{
		// version marker
		let mut version = [0u8; 1];
		reader.read(&mut version).expect("reading version marker");
		if version[0] != VERSION_TWO {
			Some(Error::new(ErrorKind::Other, "version marker is not '2'"));
		}
		// frame type
		// NOTE: read_line() strangely returns trailing \n, lines() not
		let mut frame_type = String::new();
		let mut bytes_read = reader
			.read_line(&mut frame_type)
			.expect("reading frame type");
		bytes_read -= 1;
		frame_type.truncate(bytes_read);
		// header
		let mut header: Vec<Header> = vec![];
		let mut body_type: String = String::new();
		let mut port: String = String::new();
		let mut body_length: usize = 0;
		for line in reader.by_ref().lines() {
			let line = match line {
				Ok(line) => line,
				Err(e) => return Err(e),
			};
			if line.len() == 0 {
				// got empty line; done with header
				break;
			}
			// split line
			let line_parts: Vec<&str> = line.splitn(2, ':').collect();
			if line_parts.len() != 2 {
				Some(Error::new(
					ErrorKind::Other,
					"header line contains no colon",
				));
			}
			// act accordingly
			if line_parts[0] == "port" {
				port = line_parts[1].to_string();
			} else if line_parts[0] == "type" {
				body_type = line_parts[1].to_string();
			} else if line_parts[0] == "length" {
				body_length = line_parts[1].parse().expect("parsing body length");
			} else {
				// add to headers
				header.push(Header(line_parts[0].to_string(), line_parts[1].to_string()));
			}
		}
		// body, if length > 0
		let mut body: Vec<u8> = Vec::with_capacity(body_length);
		reader.read_exact(&mut body).expect("reading body");
		// frame terminator byte
		let mut terminator = [0u8; 1];
		reader
			.read(&mut terminator)
			.expect("reading frame terminator");
		if terminator[0] != 0 {
			Some(Error::new(
				ErrorKind::Other,
				"frame terminator is no null byte",
			));
		}
		return Ok(IP {
			frame_type: frame_type,
			body_type: body_type,
			port: port,
			headers: header,
			body: body,
		});
	}

	// tuple
	pub struct Header(pub String, pub String);

	pub struct IP {
		pub frame_type: String,
		pub body_type: String,
		pub port: String,
		pub headers: Vec<Header>,
		pub body: Vec<u8>,
	}

	use std::io::ErrorKind;
	use std::io::Write;
	impl IP {
		#[allow(dead_code)]
		//TODO further optimizations using BufWrite @ https://github.com/Kixunil/genio
		// NOTE: use BufWriter to wrap STDOUT, otherwise 1 syscall per byte written
		pub fn marshal<T>(&self, mut writer: T) -> Option<Error>
		where
			T: Write,
		{
			// version marker
			match writer.write(&[b'2']) {
				Err(e) => return Some(e),
				_ => (),
			};
			// frame type
			if self.frame_type == "" {
				return Some(Error::new(ErrorKind::Other, "frame_type emtpy"));
			}
			match write!(&mut writer, "{}\n", self.frame_type) {
				Err(e) => return Some(e),
				_ => (),
			};
			// body type, if present
			if self.body_type != "" {
				match write!(&mut writer, "type:{}\n", self.body_type) {
					Err(e) => return Some(e),
					_ => (),
				};
			}
			// port, if present
			if self.port != "" {
				match write!(&mut writer, "port:{}\n", self.port) {
					Err(e) => return Some(e),
					_ => (),
				};
			}
			// other header fields, if present
			if !self.headers.is_empty() {
				for header in self.headers.iter() {
					match write!(&mut writer, "{}:{}\n", header.0, header.1) {
						Err(e) => return Some(e),
						_ => (),
					};
				}
			}
			// is body present?
			if !self.body.is_empty() {
				// body length and end-of-header marker = empty line
				match write!(&mut writer, "length:{}\n\n", self.body.len()) {
					Err(e) => return Some(e),
					_ => (),
				};
				// body
				match writer.write(&self.body) {
					Err(e) => return Some(e),
					_ => (),
				};
			} else {
				// end-of-header marker
				match writer.write(&[b'\n']) {
					Err(e) => return Some(e),
					_ => (),
				};
			}
			// frame terminator = null byte
			match writer.write(&[0x00]) {
				Err(e) => return Some(e),
				_ => (),
			};
			// success
			None
		}
	}
}

//TODO Implement using nom parser
//TODO problem: Nom is not a streaming parser, but has some support around it:
// https://github.com/Geal/generator_nom
// https://stackoverflow.com/questions/46876879/how-do-i-create-a-streaming-parser-in-nom

//TODO turn into FrameParser, which can consume the BufReader -> feed itself, then hand out references to parsed frames.
// ^ possible without using a parser library.

#[cfg(test)]
mod tests {
	use flowd;
	use std::io;

	#[test]
	fn bla() {
		let mut p: flowd::Parser = flowd::Parser::new();
		p.run("/dev/shm/testframe").expect("MASSIVE ERROR");
	}

	#[test]
	fn parse_frame_parses() {
		let frame_str_v2: String = format!(
			"2{}\n{}\n{}\n{}\n{}\n\n{}\0",
			"data", "type:TCPPacket", "port:IN", "conn-id:1", "length:2", "a\n"
		);
		let cursor = io::Cursor::new(frame_str_v2);
		let ip = flowd::parse_frame(cursor);
		let ip = ip.expect("unpacking parse result");
		assert_eq!(ip.frame_type, "data");
		assert_eq!(ip.body_type, "TCPPacket");
		assert_eq!(ip.port, "IN");
		assert_eq!(ip.headers[0].0, "conn-id");
		assert_eq!(ip.headers[0].1, "1");
		let body = String::from_utf8(ip.body).expect("parsed body to utf8 string");
		assert_eq!(body, "a\n");
	}

	#[test]
	fn marshal_frame_marshals() {
		let frame = flowd::IP {
			frame_type: "data".to_owned(),
			body_type: "TCPPacket".to_owned(),
			port: "IN".to_owned(),
			headers: vec![flowd::Header("conn-id".to_owned(), "1".to_owned())],
			body: b"a\n".to_vec(),
		};
		let mut buffer: Vec<u8> = vec![];
		match frame.marshal(&mut buffer) {
			Some(e) => panic!(e),
			_ => (),
		};
		let marshaled_str: String =
			String::from_utf8(buffer).expect("converting marshaled bytes to utf8 string");
		let frame_str_v2: String = format!(
			"2{}\n{}\n{}\n{}\n{}\n\n{}\0",
			"data", "type:TCPPacket", "port:IN", "conn-id:1", "length:2", "a\n"
		);
		assert_eq!(frame_str_v2, marshaled_str);
	}

	use test::Bencher;

	#[bench]
	// NOTE: to check if compiler did optimize actual benchmark work away
	fn empty(b: &mut Bencher) {
		b.iter(|| 1)
	}

	#[bench]
	#[allow(unused_variables)]
	fn parse_v2(b: &mut Bencher) {
		let frame_str_v2: String = format!(
			"2{}\n{}\n{}\n{}\n{}\n\n{}\0",
			"data", "type:TCPPacket", "port:IN", "conn-id:1", "length:2", "a\n"
		);
		b.iter(|| {
			let cursor = io::Cursor::new(&frame_str_v2);
			let ip = flowd::parse_frame(cursor).unwrap();
		})
	}

	#[bench]
	fn marshal_v2(b: &mut Bencher) {
		let frame = flowd::IP {
			frame_type: "data".to_owned(),
			body_type: "TCPPacket".to_owned(),
			port: "IN".to_owned(),
			headers: vec![flowd::Header("conn-id".to_owned(), "1".to_owned())],
			body: b"a\n".to_vec(),
		};
		b.iter(|| {
			let mut buffer: Vec<u8> = vec![];
			frame.marshal(&mut buffer);
		})
	}
}
