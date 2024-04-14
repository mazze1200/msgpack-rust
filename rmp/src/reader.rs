use core::{ marker, str::Utf8Error};

use crate::{
    decode::{self, read_marker, Bytes, RmpRead},
    errors, Marker,
};

#[derive(Debug)]
pub enum ReadResult<'a> {
    Null(Marker),
    Bool(Marker, bool),
    UInt(Marker, u64),
    // U8(Marker, u8),
    // U16(Marker, u16),
    // U32(Marker, u32),
    // U64(Marker, u64),
    IInt(Marker, i64),
    // I8(Marker, i8),
    // I16(Marker, i16),
    // I32(Marker, i32),
    // I64(Marker, i64),
    Float(Marker, f64),
    // F32(Marker, f32),
    // F64(Marker, f64),
    Str(Marker, Result<&'a str, (Utf8Error, &'a [u8])>),
    Bin(Marker, &'a [u8]),
    /// Stores the Marker, the number of object, and the slice with the array of objects
    Array(Marker, u32, &'a [u8]),
    /// Stores the Marker, the number of object tuples, and the slice with the object tuples in the map
    Map(Marker, MapReader<'a>),
    Ext(Marker, i8, &'a [u8]),
}

#[derive(Debug)]
pub struct Reader<'a> {
    bytes: Bytes<'a>,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader {
            bytes: Bytes::new(buf),
        }
    }

    /// The MessagePack spec states that is allowed to have invalid utf8 sequences and the API shall
    /// offer access to the raw buffer in this case.
    /// https://github.com/msgpack/msgpack/blob/8aa09e2a6a9180a49fc62ecfefe149f063cc5e4b/spec.md?plain=1#L69
    fn try_convert_str(buffer: &'a [u8]) -> Result<&'a str, (Utf8Error, &'a [u8])> {
        match core::str::from_utf8(buffer) {
            Ok(res) => Ok(res),
            Err(err) => Err((err, buffer)),
        }
    }

    /// This function is a zero copy implementation to retrieve the values out of the slice.
    /// It is zero copy but it still has to know/measure the size of of each object, which in turn means
    /// Array32 or Map32 can be expensive since it has to iterate over all the objects in the array/map.
    ///
    /// Returns Result<Option<ReadResult<_>>, _> to show if the buffer has been completly read and is in valid state: Ok(None)
    /// If the buffer ends e.g. after a Marker and the connected data cannot be read an error is returned.
    fn read(&mut self) -> Result<Option<ReadResult<'a>>, errors::Error> {
        match read_marker(&mut self.bytes) {
            Ok(marker) => {
                let res = match marker {
                    Marker::FixPos(val) => ReadResult::UInt(marker, val as u64),
                    Marker::FixNeg(val) => ReadResult::IInt(marker, val as i64),
                    Marker::Null => ReadResult::Null(marker),
                    Marker::True => ReadResult::Bool(marker, true),
                    Marker::False => ReadResult::Bool(marker, false),
                    Marker::U8 => ReadResult::UInt(marker, self.bytes.read_data_u8()? as u64),
                    Marker::U16 => ReadResult::UInt(marker, self.bytes.read_data_u16()?as u64),
                    Marker::U32 => ReadResult::UInt(marker, self.bytes.read_data_u32()?as u64),
                    Marker::U64 => ReadResult::UInt(marker, self.bytes.read_data_u64()?as u64),
                    Marker::I8 => ReadResult::IInt(marker, self.bytes.read_data_i8()?as i64),
                    Marker::I16 => ReadResult::IInt(marker, self.bytes.read_data_i16()?as i64),
                    Marker::I32 => ReadResult::IInt(marker, self.bytes.read_data_i32()?as i64),
                    Marker::I64 => ReadResult::IInt(marker, self.bytes.read_data_i64()?as i64),
                    Marker::F32 => ReadResult::Float(marker, self.bytes.read_data_f32()? as f64),
                    Marker::F64 => ReadResult::Float(marker, self.bytes.read_data_f64()?),
                    Marker::FixStr(val) => ReadResult::Str(
                        marker,
                        Reader::try_convert_str(self.bytes.read_exact_ref(val as usize)?),
                    ),
                    Marker::Str8 => {
                        let len: u8 = self.bytes.read_data_u8()?;
                        ReadResult::Str(
                            marker,
                            Reader::try_convert_str(self.bytes.read_exact_ref(len as usize)?),
                        )
                    }
                    Marker::Str16 => {
                        let len: u16 = self.bytes.read_data_u16()?;
                        ReadResult::Str(
                            marker,
                            Reader::try_convert_str(self.bytes.read_exact_ref(len as usize)?),
                        )
                    }
                    Marker::Str32 => {
                        let len: u32 = self.bytes.read_data_u32()?;
                        ReadResult::Str(
                            marker,
                            Reader::try_convert_str(self.bytes.read_exact_ref(len as usize)?),
                        )
                    }
                    Marker::Bin8 => {
                        let len: u8 = self.bytes.read_data_u8()?;
                        ReadResult::Bin(marker, self.bytes.read_exact_ref(len as usize)?)
                    }
                    Marker::Bin16 => {
                        let len: u16 = self.bytes.read_data_u16()?;
                        ReadResult::Bin(marker, self.bytes.read_exact_ref(len as usize)?)
                    }
                    Marker::Bin32 => {
                        let len: u32 = self.bytes.read_data_u32()?;
                        ReadResult::Bin(marker, self.bytes.read_exact_ref(len as usize)?)
                    }
                    Marker::FixArray(count) => {
                        if count > 0 {
                            let mut reader = Reader::new(self.bytes.remaining_slice());
                            for _ in 0..count {
                                let _ = reader.read()?;
                            }

                            ReadResult::Array(
                                marker,
                                count as u32,
                                &self
                                    .bytes
                                    .read_exact_ref(reader.bytes.position() as usize)?,
                            )
                        } else {
                            ReadResult::Array(
                                marker,
                                count as u32,
                                &self.bytes.remaining_slice()[..0],
                            )
                        }
                    }
                    Marker::Array16 => {
                        let count: u16 = self.bytes.read_data_u16()?;
                        if count > 0 {
                            let mut reader = Reader::new(self.bytes.remaining_slice());
                            for _ in 0..count {
                                let _ = reader.read()?;
                            }

                            ReadResult::Array(
                                marker,
                                count as u32,
                                &self
                                    .bytes
                                    .read_exact_ref(reader.bytes.position() as usize)?,
                            )
                        } else {
                            ReadResult::Array(
                                marker,
                                count as u32,
                                &self.bytes.remaining_slice()[..0],
                            )
                        }
                    }
                    Marker::Array32 => {
                        let count: u32 = self.bytes.read_data_u32()?;
                        if count > 0 {
                            let mut reader = Reader::new(self.bytes.remaining_slice());
                            for _ in 0..count {
                                let _ = reader.read()?;
                            }

                            ReadResult::Array(
                                marker,
                                count as u32,
                                &self
                                    .bytes
                                    .read_exact_ref(reader.bytes.position() as usize)?,
                            )
                        } else {
                            ReadResult::Array(
                                marker,
                                count as u32,
                                &self.bytes.remaining_slice()[..0],
                            )
                        }
                    }
                    Marker::FixMap(count) => {
                        if count > 0 {
                            let mut reader = Reader::new(self.bytes.remaining_slice());
                            for _ in 0..count {
                                let _ = reader.read()?;
                                let _ = reader.read()?;
                            }

                            ReadResult::Map(
                                marker,
                                MapReader::new(count as u32, &self
                                    .bytes
                                    .read_exact_ref(reader.bytes.position() as usize)?)                                                   
                            )
                        } else {
                            ReadResult::Map(
                                marker,
                                MapReader::new( 0,  &self.bytes.remaining_slice()[..0])                                                
                            )
                        }
                    }
                    Marker::Map16 => {
                        let count: u16 = self.bytes.read_data_u16()?;
                        if count > 0 {
                            let mut reader = Reader::new(self.bytes.remaining_slice());
                            for _ in 0..count {
                                let _ = reader.read()?;
                                let _ = reader.read()?;
                            }

                            ReadResult::Map(
                                marker,
                                MapReader::new(count as u32, &self
                                    .bytes
                                    .read_exact_ref(reader.bytes.position() as usize)?)                                                   
                            )
                        } else {
                            ReadResult::Map(
                                marker,
                                MapReader::new( 0,  &self.bytes.remaining_slice()[..0])                                                
                            )
                        }
                    }
                    Marker::Map32 => {
                        let count: u32 = self.bytes.read_data_u32()?;
                        if count > 0 {
                            let mut reader = Reader::new(self.bytes.remaining_slice());
                            for _ in 0..count {
                                let _ = reader.read()?;
                                let _ = reader.read()?;
                            }
                            ReadResult::Map(
                                marker,
                                MapReader::new(count as u32, &self
                                    .bytes
                                    .read_exact_ref(reader.bytes.position() as usize)?)                                                   
                            )
                        } else {
                            ReadResult::Map(
                                marker,
                                MapReader::new( 0,  &self.bytes.remaining_slice()[..0])                                                
                            )
                        }
                    }
                    Marker::FixExt1 => {
                        let ext_type: i8 = self.bytes.read_data_i8()?;
                        ReadResult::Ext(marker, ext_type, self.bytes.read_exact_ref(1usize)?)
                    }
                    Marker::FixExt2 => {
                        let ext_type: i8 = self.bytes.read_data_i8()?;
                        ReadResult::Ext(marker, ext_type, self.bytes.read_exact_ref(2usize)?)
                    }
                    Marker::FixExt4 => {
                        let ext_type: i8 = self.bytes.read_data_i8()?;
                        ReadResult::Ext(marker, ext_type, self.bytes.read_exact_ref(4usize)?)
                    }
                    Marker::FixExt8 => {
                        let ext_type: i8 = self.bytes.read_data_i8()?;
                        ReadResult::Ext(marker, ext_type, self.bytes.read_exact_ref(8usize)?)
                    }
                    Marker::FixExt16 => {
                        let ext_type: i8 = self.bytes.read_data_i8()?;
                        ReadResult::Ext(marker, ext_type, self.bytes.read_exact_ref(16usize)?)
                    }
                    Marker::Ext8 => {
                        let len: u8 = self.bytes.read_data_u8()?;
                        let ext_type: i8 = self.bytes.read_data_i8()?;
                        ReadResult::Ext(marker, ext_type, self.bytes.read_exact_ref(len as usize)?)
                    }
                    Marker::Ext16 => {
                        let len: u16 = self.bytes.read_data_u16()?;
                        let ext_type: i8 = self.bytes.read_data_i8()?;
                        ReadResult::Ext(marker, ext_type, self.bytes.read_exact_ref(len as usize)?)
                    }
                    Marker::Ext32 => {
                        let len: u32 = self.bytes.read_data_u32()?;
                        let ext_type: i8 = self.bytes.read_data_i8()?;
                        ReadResult::Ext(marker, ext_type, self.bytes.read_exact_ref(len as usize)?)
                    }
                    Marker::Reserved => unimplemented!(),
                };

                Ok(Some(res))
            }
            Err(err) => match err.0 {
                crate::decode::bytes::BytesReadError::InsufficientBytes {
                    expected: _,
                    actual: _,
                    position: _,
                } => return Ok(None),
            },
        }
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<ReadResult<'a>, errors::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read() {
            Ok(res) => match res {
                Some(val) => Some(Ok(val)),
                None => None,
            },
            Err(err) => Some(Err(err)),
        }
    }
}

#[cfg(not(feature = "std"))]
impl From<Utf8Error> for errors::Error{
    fn from(_value: Utf8Error) -> Self {
        errors::Error::Utf8Error
    }
}

#[cfg(not(feature = "std"))]
impl From<(Utf8Error, &[u8])> for errors::Error{
    fn from((error,_): (Utf8Error, &[u8])) -> Self {
        error.into()
    }
}

pub enum MapReadError {
    MapEmpty,
    ParseError,
}

impl From<decode::Error> for MapReadError {
    fn from(_value: decode::Error) -> Self {
        MapReadError::ParseError
    }
}

#[derive(Debug)]
pub struct MapReader<'a> {
    reader: Reader<'a>,
    count: u32,
    index: u32,
}

#[derive(Debug)]
pub struct InvalidMarker {}

impl From<InvalidMarker> for errors::Error{
    fn from(_value: InvalidMarker) -> Self {
        errors::Error::InvalidMarker
    }
}

impl<'a> MapReader<'a> {
    pub fn new(count: u32, buffer: &'a [u8]) -> Self {
      MapReader {
                reader: Reader::new(buffer),
                count,
                index: 0,
            }
    }

    pub fn len(&self) -> usize{
        self.count as usize
    }

    fn read_map(&mut self) -> Result<(ReadResult<'a>, ReadResult<'a>), MapReadError> {
        let key = self.reader.next().ok_or(MapReadError::MapEmpty)??;
        let val = self.reader.next().ok_or(MapReadError::ParseError)??;

        Ok((key, val))
    }
}

impl<'a> Iterator for MapReader<'a> {
    type Item = Result<(ReadResult<'a>, ReadResult<'a>), errors::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.count {
            let res = self.read_map();
            match res {
                Ok((key,val)) => {
                    self.index += 1;
                    Some(Ok((key,val)))
                },
                Err(err) => {
                    self.index = self.count;
                    match err {
                        MapReadError::MapEmpty => Some(Err(errors::Error::MapMissingElementsError)),
                        MapReadError::ParseError => Some(Err(errors::Error::ValueReadError)),
                    }
                },
            }
        } else {
            None
        }
    }
}
