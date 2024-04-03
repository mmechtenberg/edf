use crate::error::{Error, ErrorKind,HeaderError,Result};
use chrono::{NaiveDate, Datelike, NaiveTime};
use std::fs::File;
use std::io::Read;
use crate::edf_file::Header;

/// Reads and validates the header.
pub fn read_header(f: &File) -> Result<Header> {
	read_version(&f)?;
	let patient_info = read_patient_info(f)?;
	let recording_id = read_recording_id(f)?;
	let start_date = read_start_date(f)?;
	let start_time = read_start_time(f)?;
	let size = read_header_size(f)?;
	let reserved = read_reserved(f)?;
	let records_len = read_records_len(f)?;
	let duration = read_duration(f)?;
	let signals_len = read_signals_len(f)?;
	Ok(Header::new(
		patient_info,
		recording_id,
		start_date,
		start_time,
		size,
		reserved,
		records_len,
		duration,
		signals_len,
	))
}

/// Reads and validate the version.
///
/// Bytes from 0–80 are the version. The version is always 0.
fn read_version(mut f: &File) -> Result<()> {
	let mut buffer = [0; 8];
	f.read_exact(&mut buffer)?;
	if buffer[0] != 48 {
		return Err(Error::new(ErrorKind::Header(HeaderError::Version)));
	}
	for i in buffer.into_iter().skip(1) {
		if i != 32 {
			return Err(Error::new(ErrorKind::Header(HeaderError::Version)));
		}
	}
	Ok(())
}

/// Reads patient information.
fn read_patient_info(mut f: &File) -> Result<String> {
	let mut buffer = [0; 80];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	Ok(s)
}

/// Reads recording information.
fn read_recording_id(mut f: &File) -> Result<String> {
	let mut buffer = [0; 80];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	Ok(s)
}

/// Reads the start date of the recording.
fn read_start_date(mut f: &File) -> Result<NaiveDate> {
	let mut buffer = [0; 8];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	let date = parse_start_date(s).expect("Invalid start time");
	Ok(date)
}

/// Reads the start time of the recording.
fn read_start_time(mut f: &File) -> Result<NaiveTime> {
	let mut buffer = [0; 8];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	let time = NaiveTime::parse_from_str(&s, "%H.%M.%S").expect("Invalid start time");
	Ok(time)
}

/// Reads the number of bytes.
fn read_header_size(mut f: &File) -> Result<usize> {
	let mut buffer = [0; 8];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	let n = s.trim_end().parse().expect("Could not parse header size");
	Ok(n)
}

// Parse the start date from a string.
fn parse_start_date(s: String) -> std::result::Result<NaiveDate, chrono::ParseError> {
	let date = NaiveDate::parse_from_str(&s, "%d.%m.%y")?;
	// The spec specifies a clipping date of 1985.
	let date = if date.year() < 1985 {
		date.with_year(date.year() + 100)
	} else {
		Some(date)
	}
	.unwrap();
	Ok(date)
}

/// Reads the reserved block.
fn read_reserved(mut f: &File) -> Result<String> {
	let mut buffer = [0; 44];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	Ok(s)
}

/// Reads the number of records.
fn read_records_len(mut f: &File) -> Result<Option<usize>> {
	let mut buffer = [0; 8];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	let n = s
		.trim_end()
		.parse::<isize>()
		.expect("Could not parse number of records");
	if n == -1 {
		Ok(None)
	} else if n > 0 {
		Ok(Some(n as usize))
	} else {
		panic!("Record length cannot be negative");
	}
}

/// Reads the duration of a data record.
///
/// The spec recommends that it is a whole number of seconds.
fn read_duration(mut f: &File) -> Result<usize> {
	let mut buffer = [0; 8];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	let s = s.trim_end();
	// Check to see if there is a trailing decimal.
	let split = s.split_once(".");
	let n = match split {
		None => s,
		Some((characteristic, mantissa)) => match mantissa.parse::<u8>() {
			Ok(v) => match v {
				// The trailing decimals were just zeroes. Continue.
				0 => characteristic,
				_ => panic!("Unimplemented parsing of float durations"),
			},
			Err(_) => panic!("Could not parse mantissa of duration"),
		},
	}
	.parse()
	.expect("Could not parse duration");
	Ok(n)
}

/// Reads the number of signals in the data record.
fn read_signals_len(mut f: &File) -> Result<u32> {
	let mut buffer = [0; 4];
	f.read_exact(&mut buffer)?;
	let s = String::from_utf8(buffer.to_vec())?;
	let n = s
		.trim_end()
		.parse()
		.expect("Could not parse number of signals");
	Ok(n)
}

#[cfg(test)]
mod tests {
	use chrono::NaiveDate;

	// Check that month and date are in the right order.
	#[test]
	fn parse_start_date_simple() {
		let s = String::from("31.01.01");
		assert_eq!(
			super::parse_start_date(s),
			Ok(NaiveDate::from_ymd(2001, 1, 31))
		);
	}

	#[test]
	fn parse_start_date_y2k() {
		let s = String::from("01.01.00");
		assert_eq!(
			super::parse_start_date(s),
			Ok(NaiveDate::from_ymd(2000, 1, 1))
		);
	}

	#[test]
	fn parse_start_date_before_clip() {
		let s = String::from("01.01.85");
		assert_eq!(
			super::parse_start_date(s),
			Ok(NaiveDate::from_ymd(1985, 1, 1))
		);
	}

	#[test]
	fn parse_start_date_after_clip() {
		let s = String::from("31.12.84");
		assert_eq!(
			super::parse_start_date(s),
			Ok(NaiveDate::from_ymd(2084, 12, 31))
		);
	}
}
