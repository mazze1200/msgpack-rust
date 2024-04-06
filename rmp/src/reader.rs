use crate::{decode::{self, bytes::BytesReadError, read_marker, Bytes, MarkerReadError, RmpRead, RmpReadErr, ValueReadError}, errors::Error, Marker};

struct Reader<'a> {
    bytes: Bytes<'a>
}

impl<'a> Reader<'a>{
    pub fn new(buf: &'a [u8]) -> Self {
        Reader{
            bytes: Bytes::new(buf)
        }
    }

    pub fn read<R>(&mut self) -> Result<Marker, ValueReadError<R>>
    where R: RmpRead + decode::RmpReadErr,
     ValueReadError<R>: From<MarkerReadError<BytesReadError>> {
        let marker = read_marker(&mut self.bytes)?;



        Ok(Marker::from_u8(0u8))
    }
}