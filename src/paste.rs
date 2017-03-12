use std::collections::BTreeMap;

#[derive(Default, Deserialize, Serialize)]
pub struct Paste {
	pub params: BTreeMap<String, String>,
	pub paste: Vec<Line>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Line {
	pub index: usize,
	pub line: String,
}

impl Paste {
	pub fn add_param(&mut self, key: String, value: String) {
		self.params.insert(key, value);
	}
	
	pub fn add_line(&mut self, index: usize, line: String) {
		self.paste.push(Line {
			index: index,
			line: line,
		});
	}
}
