mod flowd{
	use std::io::Error;
	use std::io::BufRead;
	#[allow(dead_code)]
	pub fn parse_frame<T>(mut reader: T) -> Result<IP, Error> where T: BufRead {
		// version marker
		let mut version = [0u8; 1];
		reader.read(&mut version).expect("reading version marker");
		if version[0] as char != '2' {
			//TODO return error
		}
		// frame type
		// NOTE: read_line() strangely returns trailing \n, lines() not
		let mut frame_type = String::new();
		let mut bytes_read = reader.read_line(&mut frame_type).expect("reading frame type");
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
        Err(e) => return Err(e)
    	};
			//TODOlet line = line.expect("reading header line");
			if line.len() == 0 {
				// got empty line; done with header
				break;
			}
			// split line
			let splitted = line.splitn(2, ':');
			let line_parts: Vec<&str> = splitted.collect();
			if line_parts.len() != 2 {
				//TODO return error
			}
			// act accordingly
			if line_parts[0] == "port" {
				port = line_parts[1].to_owned();
			} else if line_parts[0] == "type" {
				body_type = line_parts[1].to_owned();
			} else if line_parts[0] == "length" {
				body_length = line_parts[1].parse().expect("parsing body length");
			} else {
				// add to headers
				//println!("{} has value {}", line_parts[0], line_parts[1]);
				header.push(Header(line_parts[0].to_owned(), line_parts[1].to_owned()));
			}
		}
		// body, if length > 0
		let mut body: Vec<u8> = vec![0u8; body_length];	// NOTE: does not work: Vec::with_capacity(body_length);
		reader.read_exact(&mut body).expect("reading body");
		// frame terminator byte
		let mut terminator = [0u8; 1];
		reader.read(&mut terminator).expect("reading frame terminator");
		if terminator[0] != 0 {
			//TODO return error
		}
		return Ok(IP{
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

	impl IP {
		#[allow(dead_code)]
		pub fn marshal(& self) {
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

	fn utf8_bytes_to_string(bytes: &[u8]) -> String {
		let vector: Vec<u8> = Vec::from(bytes);
		String::from_utf8(vector).unwrap()
	}

	fn utf8_vec_to_string(vector: Vec<u8>) -> String {
		String::from_utf8(vector).unwrap()
	}

	#[test]
	fn parse_frame_parses() {
		let frame_str_v2 = format!("2{}\n{}\n{}\n{}\n{}\n\n{}\0", "data", "type:TCPPacket", "port:IN", "conn-id:1", "length:2", "a\n");
		let cursor = io::Cursor::new(frame_str_v2);
		let ip = flowd::parse_frame(cursor);
		let ip = ip.expect("unpacking parse result");
		assert_eq!(ip.frame_type, "data");
		assert_eq!(ip.body_type, "TCPPacket");
		assert_eq!(ip.port, "IN");
		assert_eq!(ip.headers[0].0, "conn-id");
		assert_eq!(ip.headers[0].1, "1");
		let body = utf8_vec_to_string(ip.body);
		assert_eq!(body, "a\n");
	}
}
