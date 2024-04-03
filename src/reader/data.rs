use crate::edf_file::{Header, SignalHeader};
use crate::error::{Result, Error, ErrorKind};
use std::fs::File;
use std::io::Read;
use ndarray::prelude::*;

/// TODO check if it works with odd data formats i.e. different
/// TODO maybe break up into functions
/// amount of samples per record
pub fn read_data(mut f: & File, s_header: &SignalHeader, header: &Header) -> Result<Array2<f64>> {

	let total_samples : u64 = s_header.samples_in_record.iter().sum();
	let n_read_i16s =  total_samples as usize * header.records_len.unwrap();

	let raw_data_array = read_raw_data_array(&f, n_read_i16s as usize)?;

	let mut data_array = convert_raw_data2data(
			raw_data_array, &header, &s_header)?;

	scale_data(&mut data_array, &s_header);

	// TODO handle errors
	// TODO n_singals samples_in_record and should be usize?
	Ok(data_array)
}

fn read_raw_data_array(mut f: &File, n_read_i16s : usize) -> Result<Array1<f64>> {
	let raw_data = read_raw_data(&f, n_read_i16s)?;

	let data_vec : Vec<f64> = raw_data.iter()
		.map(|&x| x as f64)
		.collect();
	let data_array : Array1<f64> = ArrayBase::from_vec(data_vec);

	Ok(data_array)
}

fn read_raw_data(mut f: &File, n_read_i16s : usize) -> Result<Vec<i16>> {
	if n_read_i16s % 2 != 0 {
		return Err(Error::new(ErrorKind::ParseError))
	}

	let mut raw_data : Vec<i16> = Vec::new();
	raw_data.resize(n_read_i16s, 0);

	// The unsafe block only results in a panic if the
	// number of elemets in n_read_i16s is not a multiple of 2
	// this is checked by the first condition of this function
	let mut raw_data_u8 : &mut [u8] = unsafe {
		std::slice::from_raw_parts_mut(
			raw_data.as_ptr() as *mut u8,
			raw_data.len() * 2
		)
	};
	f.read_exact(&mut raw_data_u8)?;

	return Ok(raw_data)
}

fn convert_raw_data2data(
	raw_data_array : Array1<f64>,
	header : &Header,
	s_header: &SignalHeader
) -> Result<Array2<f64>> {
	// TODO create an example file with an unequal amounts of samples per
	// record for testing and then implement acordingly
	// For it is unclear how to handle this
	// currently the data is zero padded if a signal has less
	// number of samples as the signal with the most amount of samples
	//
	let n_records = header.records_len.unwrap();

	// TODO create error message for this instance
	// in genral there has to be an greement on
	// if the file eaven should be loaded if there is no data present
	// I (@mmechtenberg) whould argue the file readin should fail if no
	// data is present
	let max_n_samples_per_record = *s_header.samples_in_record.iter().max()
			.expect("No samples indicated in the singal header");

	let max_n_samples = max_n_samples_per_record as usize * n_records;

	let shape = (header.signals_len as usize, max_n_samples);

	let mut data_a : Array2<f64> = ArrayBase::zeros(shape);

	let mut l_idx :usize = 0;
	for idx_record in 0..n_records{
		for idx_signal in 0..header.signals_len as usize{
			let n_samples2read = s_header
									.samples_in_record[idx_signal] as usize;

			let u_idx : usize = l_idx + n_samples2read;

			let lower = n_samples2read * idx_record;
			let upper = n_samples2read * (idx_record + 1);
			let sample_range = lower..upper;

			data_a.slice_mut(s![idx_signal, sample_range])
				  .assign( &raw_data_array.slice(s![l_idx..u_idx]));

			l_idx = u_idx;
		}
	}

	Ok(data_a)
}


/// Scales the digital data to the physical range.
/// The scaling operation is performed in place.
fn scale_data(data : &mut Array2<f64>, signal_header :& SignalHeader) {
	let sh = signal_header;
	let l = sh.physical_min.shape()[0];

	let scale = (&sh.physical_min - &sh.physical_max)
				/ (&sh.digital_min - &sh.digital_max);
	let offset = &sh.physical_min - &scale * &sh.digital_min;

	// do not reshape before scale and offset are set
	let scale  = scale.into_shape((l,1)).unwrap();
	let offset = offset.into_shape((l,1)).unwrap();

	*data *= &scale;
	*data += &offset;
}

