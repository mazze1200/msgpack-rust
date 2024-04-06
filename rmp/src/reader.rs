use crate::{decode::{self, bytes::BytesReadError, read_marker, Bytes, MarkerReadError, RmpRead, RmpReadErr, ValueReadError}, errors::Error, Marker};

pub enum ReadResult<'a>{
    FixPos(Marker,u8),
    FixNeg(Marker, i8),
    Null (Marker),
    True(Marker,bool ),
    False (Marker,bool ),
    U8 (Marker, u8),
    U16(Marker,u16 ),
    U32(Marker, u32),
    U64(Marker, u64),
    I8(Marker, i8),
    I16(Marker,i16 ),
    I32 (Marker, i32),
    I64 (Marker, i64),
    F32 (Marker, f32),
    F64(Marker, f64),
    FixStr(Marker, u8),
    Str8 (Marker, &'a str ),
    Str16 (Marker,&'a str  ),
    Str32(Marker, &'a str ),
    Bin8(Marker, &'a [u8]),
    Bin16(Marker,&'a [u8] ),
    Bin32(Marker,&'a [u8] ),
    FixArray(Marker,u8 ),
    Array16 (Marker, &'a [u8] ),
    Array32 (Marker, &'a [u8] ),
    FixMap(Marker,u8 ),
    Map16 (Marker,&'a [u8]  ),
    Map32 (Marker,&'a [u8]  ),
    FixExt1 (Marker, i8, &'a [u8;1] ),
    FixExt2 (Marker,  i8, &'a [u8;2]),
    FixExt4 (Marker,  i8, &'a [u8;4]),
    FixExt8 (Marker,  i8, &'a [u8;8]),
    FixExt16 (Marker, i8, &'a [u8;16] ),
    Ext8 (Marker, i8, &'a [u8]),
    Ext16 (Marker,i8, &'a [u8] ),
    Ext32 (Marker,i8, &'a [u8] ),
}

struct Reader<'a> {
    bytes: Bytes<'a>
}

impl<'a> Reader<'a>{
    pub fn new(buf: &'a [u8]) -> Self {
        Reader{
            bytes: Bytes::new(buf)
        }
    }

    pub fn read<R>(&mut self) -> Result<ReadResult<'a>, ValueReadError<R>>
    where R: RmpRead + decode::RmpReadErr,
     ValueReadError<R>: From<MarkerReadError<BytesReadError>>
     {
        let marker = read_marker(&mut self.bytes)?;
        match marker{
            Marker::FixPos(val) => Ok(ReadResult::FixPos(marker,val)),
            Marker::FixNeg(val) =>  Ok(ReadResult::FixNeg(marker,val)),
            Marker::Null =>  Ok(ReadResult::Null(marker)),
            Marker::True =>  Ok(ReadResult::True(marker,true)),
            Marker::False => Ok(ReadResult::False(marker,false)),
            Marker::U8 =>todo!(),
            // Marker::U8 => Ok(ReadResult::U8(marker,self.bytes.read_u8())),
            Marker::U16 => todo!(),
            Marker::U32 => todo!(),
            Marker::U64 => todo!(),
            Marker::I8 => todo!(),
            Marker::I16 => todo!(),
            Marker::I32 => todo!(),
            Marker::I64 => todo!(),
            Marker::F32 => todo!(),
            Marker::F64 => todo!(),
            Marker::FixStr(_) => todo!(),
            Marker::Str8 => todo!(),
            Marker::Str16 => todo!(),
            Marker::Str32 => todo!(),
            Marker::Bin8 => todo!(),
            Marker::Bin16 => todo!(),
            Marker::Bin32 => todo!(),
            Marker::FixArray(_) => todo!(),
            Marker::Array16 => todo!(),
            Marker::Array32 => todo!(),
            Marker::FixMap(_) => todo!(),
            Marker::Map16 => todo!(),
            Marker::Map32 => todo!(),
            Marker::FixExt1 => todo!(),
            Marker::FixExt2 => todo!(),
            Marker::FixExt4 => todo!(),
            Marker::FixExt8 => todo!(),
            Marker::FixExt16 => todo!(),
            Marker::Ext8 => todo!(),
            Marker::Ext16 => todo!(),
            Marker::Ext32 => todo!(),
            Marker::Reserved => todo!(),
        }


        // Ok(Marker::from_u8(0u8))
    }
}