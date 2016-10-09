use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use chrono::*;

const CHARS: &'static str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn load(data: String, id: String) -> Result<String, io::Error> {
	let mut path = PathBuf::from(data.clone());
	path.push(id.clone());
	
	let file = match File::open(path.clone()) {
		Ok(r) => r,
		Err(e) => return Err(io::Error::new(
			e.kind(),
			format!("couldn't read file: {}: {}", path.display(), e.description())
		)),
	};
	
	let mut reader = BufReader::new(file);
	let mut buf = String::new();
	match reader.read_to_string(&mut buf) {
		Ok(_) => Ok(buf),
		Err(e) => Err(e),
	}
}	

pub fn store(data: String, text: String) -> Result<String, io::Error> {
	let dt: DateTime<UTC> = UTC::now();
	let mut value = String::new();
	let mut s = String::new();
	
	let times = vec![
		dt.nanosecond() / 10000000,
		dt.second(),
		dt.minute(),
		dt.hour(),
		dt.day(),
		dt.month(),
		(dt.year() % 100) as u32,
	];
	
	for t in times {
		s = String::new();
		value = format!("{}{:02}", value, t);
		let mut v = value.parse::<u64>().unwrap();
		
		if v == 0 {
			s = "0".to_string();
		} else {
			while v > 0 {
				s.insert(0, CHARS.chars().nth((v % CHARS.len() as u64) as usize).unwrap());
				v /= CHARS.len() as u64;
			}
		}
		
		let mut path = PathBuf::from(data.clone());
		path.push(s.clone());
		
		if !path.exists() {
			let file = try!(File::create(path));
			let mut buf = BufWriter::new(file);
			try!(buf.write_all(text.as_bytes()));
			return Ok(s);
		}
	}
	
	Ok(s)
}
