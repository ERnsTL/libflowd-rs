mod flowd{
	//use std::io::prelude::*;
	//use std::io;
	use std::io::Error;
	use std::io::BufRead;
	//use std::io::Result;
	pub fn parse_frame<'a, T>(mut reader: T) -> Result<IP<'a>, Error> where T: BufRead {
	//pub fn parse_frame<'a, T>(mut reader: T) where T: BufRead {
		// version marker
		let mut version = [0u8; 1];
		reader.read(&mut version).expect("reading version marker");
		if version[0] as char != '2' {
			//TODO return error
		}
		// frame type
		let mut frame_type = String::new();
		let bytes_read = reader.read_line(&mut frame_type).expect("reading frame type");
		// header
		let mut header: Vec<Header> = vec![];
		/*
		let mut body_type: &'a str = "";
		let mut port: &'a str = "";
		let mut body_length: usize = 0;
		*/
		let mut line2: String;
		for line in reader.by_ref().lines() {
			/*
			let line = match line {
        Ok(line) => line,
        Err(e) => return Err(e)
    	};
			*/
			line2 = line.expect("reading header line");
			/*
			if line.len() == 0 {
				// got empty line; done with header
				break;
			}
			*/
			// split line
			//error here: line2 does not live long enough, shall live as long as 'a
			let splitted = line2.splitn(2, ':');
			let line_parts: Vec<&str> = splitted.collect();
			/*
			if line_parts.len() != 2 {
				//TODO return error
			}
			// act accordingly
			if line_parts[0] == "port" {
				port = line_parts[1];
			} else if line_parts[0] == "type" {
				body_type = line_parts[1];
			} else if line_parts[0] == "length" {
				body_length = line_parts[1].parse().expect("parsing body length");
			} else {
				*/
				// add to headers
				println!("{} has value {}", line_parts[0], line_parts[1]);
				header.push(Header(line_parts[0], line_parts[1]));
			//}
		}
		// body, if length > 0
		/*
		let mut body: Vec<u8> = Vec::with_capacity(body_length);
		reader.read_exact(&mut body).expect("reading body");
		// frame terminator byte
		let mut terminator = [0u8; 1];
		reader.read(&mut terminator).expect("reading frame terminator");
		if terminator[0] != 0 {
			//TODO return error
		}
		return Ok(IP{
			frameType: frame_type,
			bodyType: body_type,
			port: port,
			headers: header,
			body: body,
		});
		*/
		return Ok(IP{
			frameType: frame_type,
			bodyType: "TCPData",
			port: "IN",
			headers: header,
			body: vec![],
		});
}

	// tuple
	pub struct Header<'a>(pub &'a str, pub &'a str);

	pub struct IP<'a> {
		pub frameType: String,
		pub bodyType: &'a str,
		pub port: &'a str,
		pub headers: Vec<Header<'a>>,
		pub body: Vec<u8>,
	}

	impl<'a> IP<'a> {
		pub fn marshal(&'a self) {
			//TODO
			println!("IP.marshal!");
		}
	}
}

//TODO Implement using nom parser

#[cfg(test)]
mod tests {
	use flowd;
	use std::io;
	use std::iter::FromIterator;

	fn utf8_to_string(bytes: &[u8]) -> String {
		let vector: Vec<u8> = Vec::from(bytes);
		String::from_utf8(vector).unwrap()
	}

	#[test]
	fn parse_frame_parses() {
		let frameStrV2 = format!("2{}\n{}\n{}\n{}\n{}\n\n{}\0", "data", "type:TCPPacket", "port:IN", "conn-id:1", "length:2", "a\n");
		let cursor = io::Cursor::new(frameStrV2);
		let ip: flowd::IP = flowd::parse_frame(cursor);
		assert!(ip.frameType == "data");
		assert!(ip.bodyType == "TCPPacket");
		assert!(ip.port == "IN");
		assert!(ip.headers[0].0 == "conn-id");
		assert!(ip.headers[0].1 == "1");
		let bodyStr = utf8_to_string(ip.body);
		assert!(bodyStr == "a\0");
	}
}
