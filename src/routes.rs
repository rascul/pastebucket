use std::collections::BTreeMap;

use handlebars_iron::Template;
use iron::modifiers::Redirect;
use iron::headers::ContentType;
use iron::mime::{Mime, SubLevel, TopLevel};
use iron::prelude::*;
use iron::{status, Url};
use params::{Params, Value};
use router::Router;
use rustc_serialize::json::{Json, ToJson};

use config::Config;
use file::{load, store};

#[derive(Default)]
#[derive(RustcEncodable)]
struct PasteParams {
	params: BTreeMap<String, String>,
	paste: BTreeMap<usize, String>,
}

// not an exact representation of the struct in json, but this better suits the need
impl ToJson for PasteParams {
	fn to_json(&self) -> Json {
		let mut json = BTreeMap::new();
		for (key, value) in self.params.clone() {
			json.insert(key, value.to_json());
		}
		let mut lines = Vec::new();
		for (key, value) in self.paste.clone() {
			let mut line = BTreeMap::new();
			line.insert("index".to_string(), key.to_json());
			line.insert("line".to_string(), value.to_json());
			lines.push(line);
		}
		json.insert("paste".to_string(), lines.to_json());
		json.insert("index_width".to_string(), format!("{}", lines.len()).len().to_json());
		Json::Object(json)
	}
}

pub fn build(config: Config) -> Router {
	let mut router = Router::new();
	
	let c = config.clone();
	router.get("/", move |req: &mut Request, | {
		index_get(req, c.clone())
	}, "index");
	
	let c = config.clone();
	router.post("/", move |req: &mut Request| {
		index_post(req, c.clone())
	}, "index");
	
	let c = config.clone();
	router.get("/:id", move |req: &mut Request| {
		show_paste(req, c.clone())
	}, "show_paste");
	
	let c = config.clone();
	router.get("/:id/raw", move |req: &mut Request| {
		show_raw(req, c.clone())
	}, "show_raw");
	
	let c = config.clone();
	router.get("/:id/edit", move |req: &mut Request| {
		edit_paste(req, c.clone())
	}, "edit_paste");
	
	router
}

fn index_get(_: &mut Request, config: Config) -> IronResult<Response> {
	let mut res = Response::new();
	let mut params: BTreeMap<String, String> = BTreeMap::new();
	params.insert("url".to_string(), config.site.url);
	res.set_mut(Template::new("index", params)).set_mut(status::Ok);
	Ok(res)
}

fn index_post(req: &mut Request, config: Config) -> IronResult<Response> {
	let mut res = Response::new();
	
	let mut params:BTreeMap<String, String> = BTreeMap::new();
	params.insert("url".to_string(), config.site.url.clone());
	
	let map = match req.get_ref::<Params>() {
		Ok(r) => r,
		Err(_) => {
			params.insert("code".to_string(), "400".to_string());
			params.insert("description".to_string(), "Bad Request: Invalid POST data".to_string());
			res.set_mut(Template::new("error", params)).set_mut(status::BadRequest);
			return Ok(res);
		},
	};
	
	if let Some(&Value::String(ref text)) = map.get("text") {
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
	} else if let Some(&Value::String(ref paste)) = map.get("paste") {
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

fn show_paste(req: &mut Request, config: Config) -> IronResult<Response> {
	let mut res = Response::new();
	let mut params: BTreeMap<String, String> = BTreeMap::new();
	params.insert("url".to_string(), config.site.url.clone());
	let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
	
	if let Ok(paste) = load(config.paths.data.clone(), id.to_string()) {
		let mut params: PasteParams = Default::default();
		params.params.insert("url".to_string(), config.site.url.clone());
		params.params.insert("id".to_string(), id.to_string());
		
		let lines: Vec<String> = paste.lines().map(|s| s.to_owned()).collect();
		for i in 0..lines.len() {
			params.paste.insert(i + 1, lines[i].clone());
		}
		
		res.set_mut(Template::new("paste", params)).set_mut(status::Ok);
		return Ok(res);
	} else {
		params.insert("code".to_string(), "404".to_string());
		params.insert(
			"description".to_string(),
			"Not Found: Requested ID doesn't exist".to_string()
		);
		res.set_mut(Template::new("error", params)).set_mut(status::NotFound);
		return Ok(res);
	}
}

fn show_raw(req: &mut Request, config: Config) -> IronResult<Response> {
	let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
	
	if let Ok(paste) = load(config.paths.data.clone(), id.to_string()) {
		let mut res = Response::with((status::Ok, paste));
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

fn edit_paste(req: &mut Request, config: Config) -> IronResult<Response> {
	let mut res = Response::new();
	let mut params: BTreeMap<String, String> = BTreeMap::new();
	params.insert("url".to_string(), config.site.url.clone());
	let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
	
	if let Ok(paste) = load(config.paths.data.clone(), id.to_string()) {
		let mut params: PasteParams = Default::default();
		params.params.insert("url".to_string(), config.site.url.clone());
		params.params.insert("id".to_string(), id.to_string());
		
		let lines: Vec<String> = paste.lines().map(|s| s.to_owned()).collect();
		for i in 0..lines.len() {
			params.paste.insert(i + 1, lines[i].clone());
		}
		
		res.set_mut(Template::new("edit", params)).set_mut(status::Ok);
		return Ok(res);
	} else {
		params.insert("code".to_string(), "404".to_string());
		params.insert(
			"description".to_string(),
			"Not Found: Requested ID doesn't exist".to_string()
		);
		res.set_mut(Template::new("error", params)).set_mut(status::NotFound);
		return Ok(res);
	}
}
