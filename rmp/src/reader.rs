
use crate::{decode::{self, bytes::BytesReadError, read_marker, read_str_from_slice, Bytes, MarkerReadError, RmpRead, RmpReadErr, ValueReadError}, errors::Error, Marker};

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
    FixStr(Marker,  &'a str ),
    Str8 (Marker, &'a str ),
    Str16 (Marker,&'a str  ),
    Str32(Marker, &'a str ),
    Bin8(Marker, &'a [u8]),
    Bin16(Marker,&'a [u8] ),
    Bin32(Marker,&'a [u8] ),
    FixArray(Marker,&'a [u8] ),
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

    // pub fn test(){
    //     let buf = [65u8; 5];
    //     // let my_str = alloc::str::from_utf8(buf);
    //     let my_str = core::str::from_utf8(&buf);

    // }

    pub fn read<R>(&mut self) -> Result<ReadResult<'a>, ValueReadError<R>>
    where R: RmpRead + decode::RmpReadErr,
     ValueReadError<R>: From<MarkerReadError<BytesReadError>>,
     ValueReadError<R>: From<ValueReadError<BytesReadError>>,     
     ValueReadError<R>: From<BytesReadError>
    {
        let marker = read_marker(&mut self.bytes)?;
        match marker{
            Marker::FixPos(val) => Ok(ReadResult::FixPos(marker,val)),
            Marker::FixNeg(val) =>  Ok(ReadResult::FixNeg(marker,val)),
            Marker::Null =>  Ok(ReadResult::Null(marker)),
            Marker::True =>  Ok(ReadResult::True(marker,true)),
            Marker::False => Ok(ReadResult::False(marker,false)),
            Marker::U8 => Ok(ReadResult::U8(marker,self.bytes.read_data_u8()?)),
            Marker::U16 => Ok(ReadResult::U16(marker,self.bytes.read_data_u16()?)),
            Marker::U32 => Ok(ReadResult::U32(marker,self.bytes.read_data_u32()?)),
            Marker::U64 =>Ok(ReadResult::U64(marker,self.bytes.read_data_u64()?)),
            Marker::I8 => Ok(ReadResult::I8(marker,self.bytes.read_data_i8()?)),
            Marker::I16 => Ok(ReadResult::I16(marker,self.bytes.read_data_i16()?)),
            Marker::I32 =>Ok(ReadResult::I32(marker,self.bytes.read_data_i32()?)),
            Marker::I64 => Ok(ReadResult::I64(marker,self.bytes.read_data_i64()?)),
            Marker::F32 => Ok(ReadResult::F32(marker,self.bytes.read_data_f32()?)),
            Marker::F64 => Ok(ReadResult::F64(marker,self.bytes.read_data_f64()?)),
            Marker::FixStr(val) => Ok(ReadResult::FixStr(marker, 
                core::str::from_utf8(self.bytes.read_exact_ref(val as usize)?).unwrap())),
            Marker::Str8 => {
                let len:u8  = self.bytes.read_data_u8()?;
                Ok(ReadResult::Str8(marker, 
                core::str::from_utf8(self.bytes.read_exact_ref(len as usize)?).unwrap()))
            },
            Marker::Str16 => {
                let len:u16  = self.bytes.read_data_u16()?;
                Ok(ReadResult::Str16(marker, 
                core::str::from_utf8(self.bytes.read_exact_ref(len as usize)?).unwrap()))
            },
            Marker::Str32 =>{
                let len:u32  = self.bytes.read_data_u32()?;
                Ok(ReadResult::Str32(marker, 
                core::str::from_utf8(self.bytes.read_exact_ref(len as usize)?).unwrap()))
            },
            Marker::Bin8 =>  {
                let len:u8  = self.bytes.read_data_u8()?;
                Ok(ReadResult::Bin8(marker, 
                self.bytes.read_exact_ref(len as usize)?))
            },
            Marker::Bin16 => {
                let len:u16  = self.bytes.read_data_u16()?;
                Ok(ReadResult::Bin16(marker, 
                self.bytes.read_exact_ref(len as usize)?))
            },
            Marker::Bin32 =>{
                let len:u32  = self.bytes.read_data_u32()?;
                Ok(ReadResult::Bin32(marker, 
                self.bytes.read_exact_ref(len as usize)?))
            },
            Marker::FixArray(len) => {
                if len > 0{
                    let mut reader = Reader::new(self.bytes.remaining_slice());
                    for _ in 0..len {
                        let _ = reader.read()?;
                    }

                    Ok(ReadResult::FixArray(marker,&self.bytes.read_exact_ref(reader.bytes.position() as usize)?))
                } else {
                    Ok(ReadResult::FixArray(marker,&self.bytes.remaining_slice()[..0]))
                }
            },
            Marker::Array16 => {
                let len:u16  = self.bytes.read_data_u16()?;
                if len > 0{
                    let mut reader = Reader::new(self.bytes.remaining_slice());
                    for _ in 0..len {
                        let _ = reader.read()?;
                    }

                    Ok(ReadResult::Array16(marker,&self.bytes.read_exact_ref(reader.bytes.position() as usize)?))
                } else {
                    Ok(ReadResult::Array16(marker,&self.bytes.remaining_slice()[..0]))
                }
            },
            Marker::Array32 => {
                let len:u32  = self.bytes.read_data_u32()?;
                if len > 0{
                    let mut reader = Reader::new(self.bytes.remaining_slice());
                    for _ in 0..len {
                        let _ = reader.read()?;
                    }

                    Ok(ReadResult::Array32(marker,&self.bytes.read_exact_ref(reader.bytes.position() as usize)?))
                } else {
                    Ok(ReadResult::Array32(marker,&self.bytes.remaining_slice()[..0]))
                }
            },
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