use crate::{
    encode::{write_map_len, RmpWrite, ValueWriteError},
    errors, Marker,
};

use core::fmt::Debug;
use num_traits::cast::FromPrimitive;

// pub type WriteRequestIter<'a> = &'a mut dyn Iterator<Item= WriteRequest<'a>>;

#[derive(Debug)]
pub enum WriteRequest<'a, T: FnMut(u32) -> Self>
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
    Array(u32, T),
    /// Stores the Marker, the number of object tuples, and the slice with the object tuples in the map
    // Map( MapReader<'a>),
    Ext(i8, &'a [u8]),
}

// impl<'a> WriteRequest<'a> {}

// impl<'a> Debug for RWriteRequestIter<'a>{
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         todo!()
//     }
// }

pub fn write_request<'a, W: RmpWrite, T: FnMut(u32) -> WriteRequest<'a, T>>(
    wr: &mut W,
    data: WriteRequest<'a, T>,
) {
}

fn write_map<W: RmpWrite, F: FnMut(&mut W, u32) -> Result<(), errors::Error>>(
    wr: &mut W,
    len: u32,
    mut func: F,
) -> Result<(), errors::Error> {
    write_map_len(wr, len)?;

    for index in 0..len {
        func(wr, index)?;
    }
    Ok(())
}
