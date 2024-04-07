use crate::{
    decode::RmpReadErr,
    encode::{write_map_len, write_marker, MarkerWriteError, RmpWrite, ValueWriteError},
    errors::{self, Error},
    Marker,
};

use core::fmt::Debug;
use num_traits::cast::FromPrimitive;

// pub type WriteRequestIter<'a> = &'a mut dyn Iterator<Item= WriteRequest<'a>>;

// type ArrayWrite = dyn FnMut(u32) -> WriteRequest<'a,A,M>;

#[derive(Debug)]
pub enum WriteRequest<'a>
// where T : Debug,
{
    Null,
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Str(&'a str),
    Bin(&'a [u8]),
    /// Stores the Marker, the number of object, and the slice with the array of objects
    Array(u32),
    /// Stores the Marker, the number of object tuples, and the slice with the object tuples in the map
    Map(u32),
    Ext(i8, u32),
}

impl<'a> WriteRequest<'a> {
    fn write_map_marker<W: RmpWrite>(writer: &mut W, count: u32) -> Result<(), Error>
    where
        Error: From<<W as RmpWrite>::Error>,
    {
        match count {
            0..=15 => write_marker(writer, Marker::FixMap(count as u8))?,
            16..=0xffff => {
                write_marker(writer, Marker::Map16)?;
                let buf = count as u16;
                let bytes = buf.to_be_bytes();
                writer.write_bytes(&bytes[..])?;
            }
            _ => {
                write_marker(writer, Marker::Map32)?;
                let buf = count as u32;
                let bytes = buf.to_be_bytes();
                writer.write_bytes(&bytes[..])?;
            }
        };

        Ok(())
    }

    pub fn write_request<W: RmpWrite>(&mut self, writer: &mut W) -> Result<(), Error>
    where
        Error: From<<W as RmpWrite>::Error>,
    {
        match self {
            WriteRequest::Null => write_marker(writer, Marker::Null)?,
            WriteRequest::Bool(val) => match val {
                true => write_marker(writer, Marker::True)?,
                false => write_marker(writer, Marker::False)?,
            },
            WriteRequest::U8(_) => todo!(),
            WriteRequest::U16(_) => todo!(),
            WriteRequest::U32(_) => todo!(),
            WriteRequest::U64(_) => todo!(),
            WriteRequest::I8(_) => todo!(),
            WriteRequest::I16(_) => todo!(),
            WriteRequest::I32(_) => todo!(),
            WriteRequest::I64(_) => todo!(),
            WriteRequest::F32(_) => todo!(),
            WriteRequest::F64(_) => todo!(),
            WriteRequest::Str(_) => todo!(),
            WriteRequest::Bin(_) => todo!(),
            WriteRequest::Array(_) => todo!(),
            WriteRequest::Map(count) => Self::write_map_marker(writer, *count)?,
            WriteRequest::Ext(ext_type, count) => todo!(),
        }

        todo!()
    }
}

// impl<'a> WriteRequest<'a> {}

// impl<'a> Debug for RWriteRequestIter<'a>{
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         todo!()
//     }
// }

// pub fn write_request<'a, W: RmpWrite, A: FnMut(u32) -> WriteRequest<'a,A,M>, M:FnMut(u32) -> WriteRequest<'a,A,M> >(
//     wr: &mut W,
// ) {
// }

fn write_map<W: RmpWrite, F: FnMut(&mut W, u32) -> Result<(), Error>>(
    wr: &mut W,
    len: u32,
    mut func: F,
) -> Result<(), Error> {
    write_map_len(wr, len)?;

    for index in 0..len {
        func(wr, index)?;
    }
    Ok(())
}
