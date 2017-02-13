use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

use iron::typemap::Key;
use toml;

#[derive(Clone, Deserialize)]
pub struct Config {
	pub site: Site,
	pub paths: Paths,
}

#[derive(Clone, Deserialize)]
pub struct Site {
	pub url: String,
	pub serve: String,
}

#[derive(Clone, Deserialize)]
pub struct Paths {
	pub templates: String,
	pub assets: String,
	pub data: String,
}

impl Key for Config {
	type Value = Config;
}

pub fn load<T: AsRef<Path>>(path: T) -> Result<Config, io::Error> {
	let mut file = fs::File::open(path.as_ref())?;
	let mut buf = String::new();
	file.read_to_string(&mut buf)?;
	
	match toml::from_str(&buf) {
		Ok(c) => Ok(c),
		Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
	}
}
