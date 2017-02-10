use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

use iron::typemap::Key;
use toml;

#[derive(Clone)]
pub struct Site {
	pub url: String,
	pub serve: String,
}

#[derive(Clone)]
pub struct Paths {
	pub templates: String,
	pub assets: String,
	pub data: String,
}

#[derive(Clone)]
pub struct Config {
	pub site: Site,
	pub paths: Paths,
}

impl Config {
	pub fn new() -> Self {
		Config {
			site: Site {
				url: "http://localhost:3000".to_string(),
				serve: "localhost:3000".to_string(),
			},
			paths: Paths {
				templates: "templates".to_string(),
				assets: "assets".to_string(),
				data: "data".to_string(),
			},
		}
	}
}

impl Key for Config {
	type Value = Config;
}

pub fn load<T: AsRef<Path>>(path: T) -> Config {
	fn read<T: AsRef<Path>>(path: T) -> Result<String, io::Error> {
		let mut file = fs::File::open(path.as_ref())?;
		let mut buf = String::new();
		file.read_to_string(&mut buf)?;
		Ok(buf)
	}

	fn parse_toml(buf: String) -> Config {
		fn lookup<T: AsRef<str>>(table: toml::Value, key: T) -> Option<String> {
			if let Some(t) = table.lookup(key.as_ref()) {
				if let Some(v) = t.as_str() {
					return Some(v.to_string());
				}
			}
			None
		}
		
		let mut config = Config::new();
		let mut parser = toml::Parser::new(buf.as_str());
		let toml = parser.parse();

		if toml.is_none() {
			return config;
		}

		let toml = toml.unwrap();

		if let Some(t) = toml.get("site") {
			if let Some(url) = lookup(t.to_owned(), "url") {
				config.site.url = url;
			}
			if let Some(serve) = lookup(t.to_owned(), "serve") {
				config.site.serve = serve;
			}
		}
		if let Some(t) = toml.get("paths") {
			if let Some(templates) = lookup(t.to_owned(), "templates") {
				config.paths.templates = templates;
			}
			if let Some(assets) = lookup(t.to_owned(), "assets") {
				config.paths.assets = assets;
			}
			if let Some(data) = lookup(t.to_owned(), "data") {
				config.paths.data = data;
			}
		}

		config
	}
	
	match read(path) {
		Ok(buf) => parse_toml(buf),
		Err(_) => Config::new(),
	}
}


