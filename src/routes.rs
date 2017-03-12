use std::collections::BTreeMap;

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
	let mut params: BTreeMap<String, String> = BTreeMap::new();
	params.insert("url".to_string(), config.site.url.clone());
	res.set_mut(Template::new("index", params)).set_mut(status::Ok);
	Ok(res)
}

fn index_post(req: &mut Request) -> IronResult<Response> {
	let config = req.get::<Read<Config>>().unwrap();
	let mut res = Response::new();
	
	let mut params:BTreeMap<String, String> = BTreeMap::new();
	params.insert("url".to_string(), config.site.url.clone());
	
	let map = match req.get_ref::<params::Params>() {
		Ok(r) => r,
		Err(_) => {
			params.insert("code".to_string(), "400".to_string());
			params.insert("description".to_string(), "Bad Request: Invalid POST data".to_string());
			res.set_mut(Template::new("error", params)).set_mut(status::BadRequest);
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
				params.insert("code".to_string(), "409".to_string());
				params.insert(
					"description".to_string(),
					"Conflict: Unable to generate unique ID".to_string()
				);
				res.set_mut(Template::new("error", params)).set_mut(status::Conflict);
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
		params.insert("code".to_string(), "400".to_string());
		params.insert("description".to_string(), "Bad Request: Invalid POST data".to_string());
		res.set_mut(Template::new("error", params)).set_mut(status::BadRequest);
		return Ok(res);
	}
}

fn show_paste(req: &mut Request) -> IronResult<Response> {
	let config = req.get::<Read<Config>>().unwrap();
	let mut res = Response::new();
	let mut paste: Paste = Default::default();
	let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
	
	paste.add_param("url".to_string(), config.site.url.clone());
	paste.add_param("id".to_string(), id.to_string());
	
	if let Ok(raw) = load(config.paths.data.clone(), id.to_string()) {
		let lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
		for i in 0..lines.len() {
			paste.add_line(i + 1, lines[i].clone());
		}
		paste.add_param("index_width".to_string(), format!("{}", format!("{}", lines.len()).len()));
		res.set_mut(Template::new("paste", paste)).set_mut(status::Ok);
		return Ok(res)
	} else {
		paste.add_param("code".to_string(), "404".to_string());
		paste.add_param(
			"description".to_string(),
			"Not Found: Requested ID doesn't exist.".to_string()
		);
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
		let mut params: BTreeMap<String, String> = BTreeMap::new();
		params.insert("url".to_string(), config.site.url.clone());
		params.insert("code".to_string(), "404".to_string());
		params.insert(
			"description".to_string(),
			"Not Found: Requested ID doesn't exist".to_string()
		);
		res.set_mut(Template::new("error", params)).set_mut(status::NotFound);
		return Ok(res);
	}
}

fn edit_paste(req: &mut Request) -> IronResult<Response> {
	let config = req.get::<Read<Config>>().unwrap();
	let mut res = Response::new();
	let mut paste: Paste = Default::default();
	let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
	
	paste.add_param("url".to_string(), config.site.url.clone());
	paste.add_param("id".to_string(), id.to_string());
	
	if let Ok(raw) = load(config.paths.data.clone(), id.to_string()) {
		let lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
		for i in 0..lines.len() {
			paste.add_line(i + 1, lines[i].clone());
		}
		paste.add_param("index_width".to_string(), format!("{}", format!("{}", lines.len()).len()));
		res.set_mut(Template::new("edit", paste)).set_mut(status::Ok);
		return Ok(res)
	} else {
		paste.add_param("code".to_string(), "404".to_string());
		paste.add_param(
			"description".to_string(),
			"Not Found: Requested ID doesn't exist.".to_string()
		);
		res.set_mut(Template::new("error", paste)).set_mut(status::NotFound);
		return Ok(res)
	}
}
