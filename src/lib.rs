#![feature(generators, generator_trait, const_str_as_bytes)]
#![allow(non_upper_case_globals)]	//TODO until version and terminator are wrappen in struct
#![feature(test)]
extern crate test;


#[macro_use]
extern crate nom;
//extern crate flavors;
extern crate circular;

pub mod flowd {
	use std::io::{BufRead, Error, ErrorKind}; //, Read};
	use std::result::*;
	use std::str;
	use std::str::FromStr; //SplitN};

	//use std::fs::File;
	use std::ops::{Generator, GeneratorState};

	//use flavors::parser::header;
	use circular::Buffer;
	//use nom::types::{CompleteByteSlice, CompleteStr};
	use nom::char;
	//use nom::{alphanumeric, anychar, char, line_ending, newline};
	use nom::{AsChar, Err, InputTakeAtPosition, Needed, Offset};
	//use nom::{AsBytes, FindSubstring, FindToken, HexDisplay, InputLength, InputTakeAtPosition, Needed, Offset, };

	pub struct IP2 {
		pub frame_type: Vec<u8>,
		pub body_type: Vec<u8>,
		pub port: Vec<u8>,
		pub headers: Vec<(Vec<u8>, Vec<u8>)>,
		pub body: Vec<u8>,
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
			let b = Buffer::with_capacity(1000); // starting capacity

			Parser {
				capacity: 1000, // capacity
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
			//println!("DEBUG: read {:#?} bytes into buffer", sz);

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
				//println!("DEBUG: entered generator");
				// we will count the number of frames and use that and return value for the generator
				let mut frame_count: usize = 0;

				// overall counter of consumed bytes
				//let mut consumed = length;
				//let mut consumed = 0;

				//println!("DEBUG: entering reading loop");
				// this is the data reading loop. On each iteration we will read more data, then try to parse
				// it in the inner loop
				loop {
					// refill the buffer
					let sz = file.read(self.buf.space()).expect("should write");
					self.buf.fill(sz);
					/*
					println!(
						"DEBUG: refill: {} more bytes, available data: {} bytes, consumed: {} bytes",
						sz,
						self.buf.available_data(),
						consumed
					);
					*/

					// if there's no more available data in the buffer after a write, that means we reached
					// the end of the file
					if self.buf.available_data() == 0 {
						//println!("DEBUG: no more data to read or parse, stopping the reading loop");
						break;
					}

					let needed: Option<Needed>;

					// this is the parsing loop. After we read some data, we will try to parse from it until
					// we get an error or the parser returns `Incomplete`, indicating it needs more data.
					//println!("DEBUG: entering parsing loop");
					loop {
						let (length, frame) = {
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
							match frame_full(self.buf.data()) {
								Ok((remaining, frame)) => {
									frame_count += 1;

									// tags parsed with flavors contain a slice of the original data. We cannot
									// return that from the generator, since it is borrowed from the Buffer's internal
									// data. Instead, we use the `types::Tag` defined in `src/types.rs` to clone
									// the data

									//let t = Tag::new(tag);
									//(self.buf.data().offset(remaining), t)
									//TODO

									//(self.buf.data().offset(remaining), frame)

									// NOTE: for reason of cloning see https://stackoverflow.com/questions/35664419/how-do-i-duplicate-a-u8-slice
									(
										self.buf.data().offset(remaining),
										//([97u8, 98u8, 99u8], [65u8, 66u8, 67u8]),
										frame, //(tag.0.to_owned(), tag.1.to_owned()),
									)
								}
								Err(nom::Err::Incomplete(n)) => {
									//println!("DEBUG: not enough data, needs a refill: {:?}", n);

									needed = Some(n);
									break;
								}
								Err(nom::Err::Error(e)) | Err(Err::Failure(e)) => {
									panic!("parse error: {:#?}", e);
								}
							}
						};

						//println!("{}", tag);

						/*
						println!(
							"DEBUG: consuming {} of {} bytes",
							length,
							self.buf.available_data()
						);
						*/
						self.buf.consume(length);
						//consumed += length;

						// give the frame to the calling code. On the next call to the generator's `resume()`,
						// we will continue from the parsing loop, and go on the reading loop's next iteration
						// if necessary
						yield frame;
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

				// we finished looping over the data, return how many frames we parsed
				return frame_count;
			};

			loop {
				unsafe {
					match generator.resume() {
						GeneratorState::Yielded(frame) => {
							/*
							println!(
								"got parser result: {} = {}",
								tag.0.to_hex(16),
								tag.1.to_hex(16)
							);
							*/
							/*
							println!(
								"DEBUG: next frame:\n\tframe_type={}\n\tbody_type={}\n\tport={}\n\theaders:",
								String::from_utf8(frame.frame_type)
									.expect("unvalid utf8 in frame type"),
								String::from_utf8(frame.body_type)
									.expect("unvalid utf8 in body type"),
								String::from_utf8(frame.port).expect("unvalid utf8 in port name"),
							);
							for header in frame.headers {
								println!(
									"\t\t{} = {}",
									String::from_utf8(header.0)
										.expect("unvalid utf8 in a header field name"),
									String::from_utf8(header.1)
										.expect("unvalid utf8 in a header value")
								);
							}
							println!(
								"\tbody={}",
								String::from_utf8(frame.body).expect("unvalid utf8 in frame body")
							);
							*/
						}
						GeneratorState::Complete(tag_count) => {
							//println!("DEBUG: parsed {} framed IPs", tag_count);
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

	named!(
		frame_full<IP2>,
		do_parse!(
			char!('2')
				>> frame_type: terminated!(alphanumeric1_nonnewline, char!('\n'))
				>> headers: many_till!(header_line, char!('\n'))
				//>> headers: header_line
				>> body: take!(body_get_length(&headers.0))
				>> opt!(char!('\0'))	// finish byte for synchronization in frame stream	//TODO keep that optional?
				>> (IP2 {
					frame_type: frame_type.to_vec(),
					//frame_type: "TODO".to_owned().into_bytes(),
					body_type: "TODO".to_owned().into_bytes(),
					port: "TODO".to_owned().into_bytes(),
					headers: headers.0.to_owned(),
					//headers: vec![headers.to_owned()],
					//body: "TODO".to_owned().into_bytes(),
					body: body.to_owned(),
				}) //TODO final \0 byte
		)
	);

	const LENGTH_BYTES: &[u8] = "length".as_bytes();

	//TODO make this function obsolete - extract that length already during a do_parse! block and return it up into the frame_full do_parse! block
	fn body_get_length(headers: &Vec<(Vec<u8>, Vec<u8>)>) -> usize {
		for header in headers {
			if header.0 == LENGTH_BYTES {
				unsafe {
					return usize::from_str(str::from_utf8_unchecked(&header.1)).unwrap();
				}
			}
		}
		0
	}

	/*
	named!(
		frame_header<(&[u8], &[u8])>,
		terminated!(header_line, newline) //do_parse!(header_line)
	);
	*/

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
		header_line<(Vec<u8>, Vec<u8>)>,
		do_parse!(
			kv: terminated!(
				separated_pair!(alphanumeric1_noncolon, char!(':'), alphanumeric1_nonnewline),
				char!('\n')
			) >> ((kv.0.to_vec(), kv.1.to_vec()))
		)
	);

	// TODO optimization? 	#[inline(always)]

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
				//println!("alphanumeric1_noncolon: got char {}", c);
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
				//println!("alphanumeric1_nonewline: got char {}", c);
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
	pub fn parse_frame<T>(reader: &mut T) -> Result<IP, Error>
	where
		T: BufRead,
	{
		// version marker
		let mut version = [0u8; 1]; //TODO optimize could re-use
		reader.read(&mut version).expect("reading version marker");
		if version[0] != VERSION_TWO {
			Some(Error::new(ErrorKind::Other, "version marker is not '2'"));
		}
		// frame type
		// NOTE: read_line() strangely returns trailing \n, lines() not
		let mut frame_type = String::with_capacity(4);
		let bytes_read = reader
			.read_line(&mut frame_type)
			.expect("reading frame type");
		frame_type.truncate(bytes_read - 1);
		//frame_type.truncate(bytes_read);
		// header
		let mut header: Vec<Header> = vec![];
		let mut body_type: String = String::new();
		let mut port: String = String::new();
		let mut body_length: usize = 0;
		for line in reader.lines() {
			let line = line?;
			if line.is_empty() {
				// got empty line; done with header
				break;
			}
			// split line
			let line_parts: Vec<&str> = line.splitn(2, ':').collect();
			if line_parts.len() != 2 {
				return Err(Error::new(
					ErrorKind::InvalidInput,
					"header line contains no colon",
				));
			}
			// act accordingly
			match line_parts[0] {
				"port" => {
					port = line_parts[1].to_string();
				}
				"type" => {
					body_type = line_parts[1].to_string();
				}
				"length" => {
					// TODO optimize
					body_length = usize::from_str(&line_parts[1]).expect("parsing body length");
				}
				_ => {
					// add to headers
					header.push(Header(line_parts[0].to_string(), line_parts[1].to_string()));
				}
			}
		}
		// body, if length > 0
		let mut body: Vec<u8> = vec![0u8; body_length]; // NOTE: does not work: Vec::with_capacity(body_length);
		reader.read_exact(&mut body).expect("reading body");
		// frame terminator byte
		let mut terminator = [0u8; 1]; //TODO optimize this could be reused
		reader
			.read(&mut terminator)
			.expect("reading frame terminator");
		if terminator[0] != 0u8 {
			Some(Error::new(
				ErrorKind::InvalidData,
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

	const NULL_BYTE: &[u8] = &[0x00];
	const NEWLINE: &[u8] = &[b'\n'];
	const BODY_TYPE: &[u8] = "type:".as_bytes();
	const PORT: &[u8] = "port:".as_bytes();
	const LENGTH: &[u8] = "length:".as_bytes();
	const COLON: &[u8] = &[b':'];
	const VERSION_TWO_BUF: &[u8] = &[b'2'];

	use std::io::Write;
	impl IP {
		#[allow(dead_code)]
		//TODO further optimizations using BufWrite @ https://github.com/Kixunil/genio
		// NOTE: use BufWriter to wrap STDOUT, otherwise 1 syscall per byte written
		pub fn marshal<T>(&self, mut writer: T) -> std::io::Result<()>
		where
			T: Write,
		{
			// version marker
			writer.write(VERSION_TWO_BUF)?;
			// frame type
			if self.frame_type.is_empty() {
				return Err(Error::new(ErrorKind::Other, "frame_type emtpy"));
			}
			writer.write(self.frame_type.as_bytes())?;
			writer
				.write(NEWLINE)
				.expect("writing newline after frame type");
			// body type, if present
			if self.body_type != "" {
				/*
				match write!(&mut writer, "type:{}\n", self.body_type) {
					Err(e) => return Some(e),
					_ => (),
				};
				*/
				writer
					.write(BODY_TYPE)
					.expect("writing body type field name");
				writer
					.write(self.body_type.as_bytes())
					.expect("writing body type value");
				writer
					.write(NEWLINE)
					.expect("writing newline after body type");
			}
			// port, if present
			if self.port != "" {
				/*
				match write!(&mut writer, "port:{}\n", self.port) {
					Err(e) => return Some(e),
					_ => (),
				};
				*/
				writer.write(PORT).expect("writing port field name");
				writer
					.write(self.port.as_bytes())
					.expect("writing port value");
				writer.write(NEWLINE).expect("writing newline after port");
			}
			// other header fields, if present
			if !self.headers.is_empty() {
				for i in 0..self.headers.len() {
					//for header in self.headers.iter() {
					/*
					match write!(&mut writer, "{}:{}\n", header.0, header.1) {
						Err(e) => return Some(e),
						_ => (),
					};
					*/
					writer
						.write(self.headers[i].0.as_bytes())
						.expect("writing a header field name");
					writer.write(COLON).expect("writing a header colon");
					writer
						.write(self.headers[i].1.as_bytes())
						.expect("writing a header field value");
					writer
						.write(NEWLINE)
						.expect("writing a header field newline");
				}
			}
			// is body present?
			if !self.body.is_empty() {
				// body length
				writer.write(LENGTH).expect("writing length field name");
				writer
					.write(self.body.len().to_string().as_bytes())
					.expect("writing body length value");
				writer.write(NEWLINE)?;
				/*
				match write!(&mut writer, "length:{}\n\n", self.body.len()) {
					Err(e) => return Some(e),
					_ => (),
				};
				*/
				// end-of-header marker
				writer.write(NEWLINE)?;
				// body
				writer.write(&self.body)?;
			} else {
				// end-of-header marker
				writer.write(NEWLINE)?;
			}
			// frame terminator = null byte
			writer.write(NULL_BYTE)?;
			// success
			Ok(())
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
	fn nom_parser_parses() {
		let mut p: flowd::Parser = flowd::Parser::new();
		p.run("./testframe").expect("MASSIVE FAILURE");
	}

	#[test]
	fn parse_frame_parses() {
		let frame_str_v2: String = format!(
			"2{}\n{}\n{}\n{}\n{}\n\n{}\0",
			"data", "type:TCPPacket", "port:IN", "conn-id:1", "length:2", "a\n"
		);
		let mut cursor = io::Cursor::new(frame_str_v2);
		let ip = flowd::parse_frame(&mut cursor);
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
			Err(e) => panic!(e),
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
		let mut cursor = io::Cursor::new(&frame_str_v2);
		b.iter(|| {
			cursor.set_position(0);
			let ip = flowd::parse_frame(&mut cursor).unwrap();
		})
	}

	#[bench]
	//#[allow(unused_variables)]
	fn parse_v2_nom(b: &mut Bencher) {
		let mut p: flowd::Parser = flowd::Parser::new();
		b.iter(|| {
			p.run("./testframe").expect("MASSIVE FAILURE");
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
		let mut buffer: Vec<u8> = vec![];
		b.iter(|| {
			buffer.clear();
			frame.marshal(&mut buffer).unwrap();
		});
	}
}
