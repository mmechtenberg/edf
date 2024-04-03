use crate::edf_file::{Header, EdfFile};
use crate::error::Result;
use std::fs::File;
use std::path::Path;

mod header;
mod signal_header;
mod data;


pub struct Reader;

impl Reader {
	pub fn from_path<P: AsRef<Path>>(path: P) -> Result<EdfFile> {
		let f = File::open(path)?;
		let header = header::read_header(&f)?;
		let signal_header = signal_header::read_signal_header(&f, header.signals_len as usize)?;
		let data = data::read_data(&f, &signal_header, &header)?;

		Ok(EdfFile::new(header, signal_header, data))
	}
}


