#![feature(bufreader_buffer)]
#![feature(test)]
extern crate test;

mod flowd {
	//use std::io::prelude::*;
	use std::io::{BufRead, BufReader, Error, Read};
	use std::marker::PhantomData;
	use std::str;

	pub struct Parser<'a, T: 'a> {
		reader: BufReader<T>,
		buf: &'a [u8],
		err: Option<Error>,
		phantom: PhantomData<&'a T>,
	}

	impl<'a, T: Read> Parser<'a, T> {
		//pub fn new<T: BufReader<u8> + Read + BufRead>(mut reader: T) -> Parser {
		pub fn new(stream: T) -> Parser<'a, T> {
			Parser {
				reader: BufReader::<T>::new(stream),
				buf: &[1, 1], //TODO
				err: None,
				phantom: PhantomData,
			}
		}
	}

	const NEWLINE: u8 = 0x0a;

	impl<'a, T: Read> Iterator for Parser<'a, T> {
		type Item = IP2<'a>;

		// NOTE: Iterator wants Option; thus returning error information via Parser.err
		fn next(&mut self) -> Option<IP2<'a>> {
			/*
			FIXME this would provide the maximum speed-up, but unfortunately there is no transitive
			ownership concept in Rust. The BufReader is owned by Parser, but reader.buffer() only returns
			a borrow into its internal buffer. Access to that would allow references into it
			= without allocations. But that is not possible -> Nom parser using Rust generator.
			*/
			//self.reader.fill_buf();
			self.buf = self.reader.buffer();
			//let mut version_marker: [u8; 1] = [0u8; 1];
			//self.reader.read_exact(&mut version_marker).unwrap();

			// version marker
			if self.buf[0] != VERSION_TWO {
				self.err = Some(Error::new(ErrorKind::Other, "version marker is not '2'"));
				return None;
			}

			// frame type
			let frame_type_end: usize;
			let mut i: usize = 0;
			loop {
				if self.buf[i] == NEWLINE {
					frame_type_end = i - 1;
					break;
				} else {
					i += 1;
				}
			}
			//let sl = std::slice::from_raw_parts(&buf[1], frame_type_end - 1);
			//let frame_type: &str = std::str::from_utf8(sl).unwrap();
			let frame_type: &str = str::from_utf8(&self.buf[1..frame_type_end - 1]).unwrap();

			/*
			// header
			let mut header: Vec<Header> = vec![];
			let mut body_type: String = String::new();
			let mut port: String = String::new();
			let mut body_length: usize = 0;
			for line in self.reader.lines() {
				let line = match line {
					Ok(line) => line,
					Err(e) => {
						self.err = Some(e);
						return None;
					}
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
			self.reader.read_exact(&mut body).expect("reading body");
			// frame terminator byte
			let mut terminator = [0u8; 1];
			self.reader
				.read(&mut terminator)
				.expect("reading frame terminator");
			if terminator[0] != 0 {
				self.err = Some(Error::new(
					ErrorKind::Other,
					"frame terminator is no null byte",
				));
				return None;
			}
			*/

			//None
			Some(IP2 {
				frame_type: frame_type,
				body_type: frame_type,
				port: frame_type,
				headers: vec![("test".to_string(), "test".to_string())],
				body: &[8],
			})
		}
	}

	pub struct IP2<'b> {
		pub frame_type: &'b str,
		pub body_type: &'b str,
		pub port: &'b str,
		pub headers: Vec<(String, String)>,
		pub body: &'b [u8],
	}

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
