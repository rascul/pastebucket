extern crate chrono;
extern crate handlebars_iron;
extern crate iron;
extern crate logger;
extern crate mount;
extern crate params;
extern crate persistent;
extern crate router;
extern crate rustc_serialize;
extern crate staticfile;
extern crate toml;

mod config;
mod file;
mod routes;

use std::error::Error;
use std::fs::create_dir;
use std::path::{Path, PathBuf};

use handlebars_iron::{DirectorySource, HandlebarsEngine};
use iron::prelude::{Chain, Iron};
use logger::Logger;
use mount::Mount;
use persistent::Read;
use staticfile::Static;

use config::*;

fn main() {
	let config = load("config.toml");
	
	let dir = PathBuf::from(config.paths.data.clone());
	if !dir.exists() {
		match create_dir(dir) {
			Ok(_) => {},
			Err(e) => {
				println!("couldn't create data directory: {}: {}",
					config.paths.data,
					e.description()
				);
				std::process::exit(1);
			},
		}
	} else if !dir.is_dir() {
		println!("data directory isn't a directory: {}", config.paths.data);
		std::process::exit(2);
	}
	
	let router = routes::build();
	
	let mut mount = Mount::new();
	mount.mount("/", router);
	mount.mount("/assets", Static::new(Path::new(config.paths.assets.as_str())));
	
	let mut handlebars = HandlebarsEngine::new();
	handlebars.add(Box::new(DirectorySource::new(config.paths.templates.as_str(), ".hbs")));
	
	if let Err(r) = handlebars.reload() {
		println!("{}", r.description());
		std::process::exit(3);
	}
	
	let (logger_before, logger_after) = Logger::new(None);
	
	let mut chain = Chain::new(mount);
	chain.link_before(logger_before);
	chain.link_before(Read::<Config>::one(config.clone()));
	chain.link_after(handlebars);
	chain.link_after(logger_after);
	
	match Iron::new(chain).http(config.site.serve.as_str()) {
		Ok(_) => println!("listening on http://{}", config.site.serve),
		Err(e) => {
			println!("{}", e.description());
			std::process::exit(4);
		}
	};
}
