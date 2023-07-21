use std::io::ErrorKind;
use pi_atom::Atom;
use wasm_bindgen::JsValue;

use crate::loadFile;

#[derive(Debug, Clone)]
pub struct FileLoadErr(JsValue);

// path可能是本地路径， 也可能是网络路径，
pub async fn load_from_url(path: &Atom) -> Result<Vec<u8>, FileLoadErr> {
	match loadFile(path.get_hash() as u32).await {
		Ok(r) => {
			Ok(js_sys::Uint8Array::from(r).to_vec())
		},
		Err(e) => Err(FileLoadErr(e))
	}
}