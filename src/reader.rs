use crate::edf_file::Header;
use crate::error::Result;
use std::fs::File;
use std::path::Path;

mod header;


pub struct Reader;

impl Reader {
	pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Header> {
		let f = File::open(path)?;
		let hdr = header::read_header(&f)?;
		Ok(hdr)
	}
}


