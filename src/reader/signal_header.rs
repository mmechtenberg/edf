use ndarray::prelude::*;
use std::fs::File;
use std::io::Read;
use crate::edf_file::SignalHeader;
use crate::error::{Result, Error, ErrorKind};

pub fn read_signal_header(mut f: &File, n_signals: usize) -> Result<SignalHeader> {
    let mut signal_header = SignalHeader::default();

    signal_header.labels = read_str_vec(&mut f, 16, n_signals)?;
    signal_header.transducer_type = read_str_vec(&mut f, 80, n_signals)?;
    signal_header.physical_dimension = read_str_vec(&mut f, 8, n_signals)?;

    signal_header.physical_min = ArrayBase::from_vec(
		read_integer_vec(&mut f, 8, n_signals)?);
    signal_header.physical_max = ArrayBase::from_vec(
		read_integer_vec(&mut f, 8, n_signals)?);
    signal_header.digital_min  = ArrayBase::from_vec(
		read_integer_vec(&mut f, 8, n_signals)?);
    signal_header.digital_max  = ArrayBase::from_vec(
		read_integer_vec(&mut f, 8, n_signals)?);

    signal_header.prefiltering = read_str_vec(&mut f, 80, n_signals)?;

    signal_header.samples_in_record = read_integer_vec(&mut f, 8, n_signals)?;

    signal_header.reserved_field = read_str_vec(&mut f, 32, n_signals)?;

    Ok(signal_header)
}

///
/// TODO deduplicatate code @read_integer_vec
fn read_str_vec(mut f: &File, n_chars: usize, n_reads: usize) -> Result<Vec<String>> {
    let mut read_into: Vec<String> = Vec::new();

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(n_chars, 0);

    for _idx in 0..n_reads {
        f.read_exact(&mut buffer)?;
        let s = String::from_utf8(buffer.clone())?;
        let s = s.trim_end();
        read_into.push(s.to_string());
    }

    Ok(read_into)
}

fn read_integer_vec<T>(mut f: &File, n_chars: usize, n_reads: usize) -> Result<Vec<T>>
where
    T: std::str::FromStr,
{
    let mut read_into: Vec<T> = Vec::new();

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(n_chars, 0);

    for _idx in 0..n_reads {
        f.read_exact(&mut buffer)?;
        let s = String::from_utf8(buffer.clone())?;
        let s = s.trim();
        let i_e = s.parse();
        // TODO fixup error handling
        //	    get it to work with `?`
        let i: T = match i_e {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::new(ErrorKind::ParseError)),
        }?;
        read_into.push(i);
    }
    Ok(read_into)
}
