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
	pub fn add_param<S1: Into<String>, S2: Into<String>>(&mut self, key: S1, value: S2) {
		self.params.insert(key.into(), value.into());
	}
	
	pub fn add_line<S: Into<String>>(&mut self, index: usize, line: S) {
		self.paste.push(Line {
			index: index,
			line: line.into(),
		});
	}
}
