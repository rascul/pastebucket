use handlebars_iron::Template;
use iron::modifiers::Redirect;
use iron::headers::ContentType;
use iron::mime::{Mime, SubLevel, TopLevel};
use iron::prelude::*;
use iron::{status, Url};
use params;
use persistent::Read;
use router::Router;

use config::Config;
use file::{load, store};
use paste::Paste;

pub fn build() -> Router {
	let mut router = Router::new();
	
	router.get("/", index_get, "index");
	router.post("/", index_post, "index");
	router.get("/:id", show_paste, "show_paste");
	router.get("/:id/raw", show_raw, "show_raw");
	router.get("/:id/edit", edit_paste, "edit_paste");
	
	router
}

fn index_get(req: &mut Request) -> IronResult<Response> {
	let config = req.get::<Read<Config>>().unwrap();
	let mut res = Response::new();
	let mut paste: Paste = Default::default();
	paste.add_param("url", config.site.url.clone());
	res.set_mut(Template::new("index", paste)).set_mut(status::Ok);
	Ok(res)
}

fn index_post(req: &mut Request) -> IronResult<Response> {
	let config = req.get::<Read<Config>>().unwrap();
	let mut res = Response::new();
	let mut paste: Paste = Default::default();
	
	paste.add_param("url", config.site.url.clone());
	
	let map = match req.get_ref::<params::Params>() {
		Ok(r) => r,
		Err(_) => {
			paste.add_param("code", "400");
			paste.add_param("description", "Bad Request: Invalid POST data");
			res.set_mut(Template::new("error", paste)).set_mut(status::BadRequest);
			return Ok(res);
		},
	};
	
	if let Some(&params::Value::String(ref text)) = map.get("text") {
		match store(config.paths.data.clone(), text.clone()) {
			Ok(id) => {
				let redirect_url = Url::parse(format!(
					"{}/{}", config.site.url.clone(), id
				).as_str()).unwrap();
				let res = Response::with((status::Found, Redirect(redirect_url)));
				return Ok(res);
			},
			Err(_) => {
				paste.add_param("code", "409");
				paste.add_param("description", "Conflict: Unable to generate unique ID");
				res.set_mut(Template::new("error", paste)).set_mut(status::Conflict);
				return Ok(res);
			},
		};
	} else if let Some(&params::Value::String(ref paste)) = map.get("paste") {
		match store(config.paths.data.clone(), paste.clone()) {
			Ok(id) => {
				let redirect_url = Url::parse(format!(
					"{}/{}", config.site.url.clone(), id
				).as_str()).unwrap().to_string();
				let print_url = format!("    {}\n", redirect_url);
				let mut res = Response::with((status::Ok, print_url));
				res.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])));
				return Ok(res);
			},
			Err(_) => {
				let mut res = Response::with((
					status::Conflict,
					"Error 409 Conflict: Unable to generate a unique ID".to_string()
				));
				res.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])));
				return Ok(res);
			},
		};
	} else {
		paste.add_param("code", "400");
		paste.add_param("description", "Bad Request: Invalid POST data");
		res.set_mut(Template::new("error", paste)).set_mut(status::BadRequest);
		return Ok(res);
	}
}

fn show_paste(req: &mut Request) -> IronResult<Response> {
	let config = req.get::<Read<Config>>().unwrap();
	let mut res = Response::new();
	let mut paste: Paste = Default::default();
	let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
	
	paste.add_param("url", config.site.url.clone());
	paste.add_param("id", id);
	
	if let Ok(raw) = load(config.paths.data.clone(), id.to_string()) {
		let lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
		for i in 0..lines.len() {
			paste.add_line(i + 1, lines[i].clone());
		}
		paste.add_param("index_width", format!("{}", format!("{}", lines.len()).len()));
		res.set_mut(Template::new("paste", paste)).set_mut(status::Ok);
		return Ok(res)
	} else {
		paste.add_param("code", "404");
		paste.add_param("description", "Not Found: Requested ID doesn't exist.");
		res.set_mut(Template::new("error", paste)).set_mut(status::NotFound);
		return Ok(res)
	}
}

fn show_raw(req: &mut Request) -> IronResult<Response> {
	let config = req.get::<Read<Config>>().unwrap();
	let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
	
	if let Ok(raw) = load(config.paths.data.clone(), id.to_string()) {
		let mut res = Response::with((status::Ok, raw));
		res.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])));
		return Ok(res);
	} else {
		let mut res = Response::new();
		let mut paste: Paste = Default::default();
		paste.add_param("url", config.site.url.clone());
		paste.add_param("code", "404");
		paste.add_param("description", "Not Found: Requested ID doesn't exist.");
		res.set_mut(Template::new("error", paste)).set_mut(status::NotFound);
		return Ok(res);
	}
}

fn edit_paste(req: &mut Request) -> IronResult<Response> {
	let config = req.get::<Read<Config>>().unwrap();
	let mut res = Response::new();
	let mut paste: Paste = Default::default();
	let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
	
	paste.add_param("url", config.site.url.clone());
	paste.add_param("id", id);
	
	if let Ok(raw) = load(config.paths.data.clone(), id.to_string()) {
		let lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
		for i in 0..lines.len() {
			paste.add_line(i + 1, lines[i].clone());
		}
		paste.add_param("index_width", format!("{}", format!("{}", lines.len()).len()));
		res.set_mut(Template::new("edit", paste)).set_mut(status::Ok);
		return Ok(res)
	} else {
		paste.add_param("code", "404");
		paste.add_param(
			"description",
			"Not Found: Requested ID doesn't exist."
		);
		res.set_mut(Template::new("error", paste)).set_mut(status::NotFound);
		return Ok(res)
	}
}
