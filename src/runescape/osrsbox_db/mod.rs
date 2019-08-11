pub mod types;

use std::collections::BTreeMap;
use ureq;
use serde_json;
use serde_json::{Map, Value};
use std::io::Result as IoResult;
use super::RunescapeInt;

impl From<types::Slot> for String {
	fn from(slot: types::Slot) -> Self {
		serde_json::to_value(&slot).unwrap().as_str().unwrap().into()
	}
}

fn raw_request(slot: types::Slot) -> IoResult<Map<String, Value>> {
	let slot: String = slot.into();
	let url = format!("https://www.osrsbox.com/osrsbox-db/items-json-slot/items-{}.json", slot);
	match ureq::get(&url).call().into_json()? {
		Value::Object(m) => Ok(m),
		_                => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "non-object response from osrsbox")),
	}
}

fn map_to_items(map: Map<String, Value>) -> IoResult<BTreeMap<RunescapeInt, types::Item>> {
	let mut m = BTreeMap::new();
	// let mut v = Vec::with_capacity(map.len());
	for (k, v) in map.into_iter() {
		let k = k.parse::<RunescapeInt>();
		let k = match k {
			Ok(k) => k,
			Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
		};
		if let Some(_) = m.insert(k, serde_json::from_value(v)?) {
			unreachable!();
		}
	}
	Ok(m)
}

pub fn request(slot: types::Slot) -> IoResult<BTreeMap<RunescapeInt, types::Item>> {
	map_to_items(raw_request(slot)?)
}
