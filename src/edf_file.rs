use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::fmt;

pub struct EdfFile {
    pub header : Header
}

pub struct Header {
	pub patient_info: String,
	pub recording_id: String,
	/// The start date and time of the recording/
	pub start_datetime: NaiveDateTime,
	// The number of bytes in the header.
	pub size: usize,
	pub reserved: String,
	// The number of records. If unknown (value is -1), then it is `None`.
	pub records_len: Option<usize>,
	// The duration of a a record in seconds.
	pub duration: usize,
	// The number of signals in the record
	pub signals_len: u32,
}

impl Header {
	pub fn new(
		patient_info: String,
		recording_id: String,
		start_date: NaiveDate,
		start_time: NaiveTime,
		size: usize,
		reserved: String,
		records_len: Option<usize>,
		duration: usize,
		signals_len: u32,
	) -> Self {
		let start_datetime = NaiveDateTime::new(start_date, start_time);
		Self {
			patient_info,
			recording_id,
			start_datetime,
			size,
			reserved,
			records_len,
			duration,
			signals_len,
		}
	}
}

impl fmt::Display for Header {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let records_len = match self.records_len {
			None => "-1".to_string(),
			Some(v) => v.to_string(),
		};

		write!(
			f,
			"\n## Header\n{}\nRecording ID: {}\nStart Time: {}\nSize of header: {} B\nReserved: {}\n{} data records\n{} seconds\n{} signals",
			self.patient_info,
			self.recording_id,
			self.start_datetime,
			self.size,
			self.reserved,
			records_len,
			self.duration,
			self.signals_len
		)
	}
}