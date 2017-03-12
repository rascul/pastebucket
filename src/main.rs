extern crate chrono;
extern crate handlebars_iron;
extern crate iron;
extern crate logger;
extern crate mount;
extern crate params;
extern crate persistent;
extern crate router;
#[macro_use]
extern crate serde_derive;
extern crate staticfile;
extern crate toml;

mod config;
mod file;
mod paste;
mod routes;

use std::env::var;
use std::error::Error;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
#[cfg(feature = "watch")]
use std::sync::Arc;

use handlebars_iron::{DirectorySource, HandlebarsEngine};
#[cfg(feature = "watch")]
use handlebars_iron::Watchable;

use iron::prelude::{Chain, Iron};
use logger::Logger;
use mount::Mount;
use persistent::Read;
use staticfile::Static;

use config::*;

#[cfg(feature = "watch")]
fn load_templates(path: String) -> Arc<HandlebarsEngine> {
	let mut handlebars = HandlebarsEngine::new();
	handlebars.add(Box::new(DirectorySource::new(path.as_str(), ".hbs")));
	
	if let Err(r) = handlebars.reload() {
		println!("{}", r.description());
		std::process::exit(3);
	}
	
	let handlebars_ref = Arc::new(handlebars);
	handlebars_ref.watch(path.as_str());
	
	handlebars_ref
}

#[cfg(not(feature = "watch"))]
fn load_templates(path: String) -> HandlebarsEngine {
	let mut handlebars = HandlebarsEngine::new();
	handlebars.add(Box::new(DirectorySource::new(path.as_str(), ".hbs")));
	
	if let Err(r) = handlebars.reload() {
		println!("{}", r.description());
		std::process::exit(3);
	}
	
	handlebars
}

fn main() {
	let config = match load("config.toml") {
		Ok(c) => c,
		Err(e) => {
			println!("couldn't load config: config.toml: {}", e.description());
			std::process::exit(5)
		},
	};
	
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
	
	let (logger_before, logger_after) = Logger::new(None);
	
	let mut chain = Chain::new(mount);
	chain.link_before(logger_before);
	chain.link_before(Read::<Config>::one(config.clone()));
	chain.link_after(load_templates(config.paths.templates.clone()));
	chain.link_after(logger_after);
	
	let serve = if var("HOST").is_ok() && var("PORT").is_ok() {
		format!("{}:{}", var("HOST").unwrap(), var("PORT").unwrap())
	} else {
		config.site.serve.clone()
	};
	
	match Iron::new(chain).http(config.site.serve.as_str()) {
		Ok(_) => println!("listening on http://{}", serve),
		Err(e) => {
			println!("{}", e.description());
			std::process::exit(4);
		}
	};
}

	