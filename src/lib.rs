extern crate rustc_serialize as rustc_serialize;
extern crate bincode;
extern crate bufstream;

use std::{io, fmt, error, result, convert};
use bincode::{EncodingError, DecodingError};

#[macro_export]
macro_rules! urpc {
    (
        pub interface $mod_name:ident {
            $(
                fn $fn_name:ident (
                    $(
                        $arg:ident: $arg_ty:ty
                    ),*
                ) -> $ret_ty:ty {}
            )*
        }
    ) => { pub mod $mod_name {
        pub trait Methods {
            $(
                fn $fn_name(&mut self, $( $arg: $arg_ty ),*)
                    -> $crate::Result<$ret_ty>;
            )*
        }

        #[allow(non_camel_case_types)]
        #[derive(RustcEncodable, RustcDecodable)]
        pub enum Request {
            _SHUTDOWN,
            $(
                $fn_name($( $arg_ty ),*),
            )*
        }

        pub struct Client<Stream: $crate::rt::Stream> {
            stream: $crate::rt::BufStream<Stream>,
        }

        impl<Stream> Client<Stream>
            where Stream: $crate::rt::Stream,
        {
            pub fn new(stream: Stream) -> Client<Stream> {
                Client {
                    stream: $crate::rt::BufStream::new(stream),
                }
            }
        }

        impl<Stream> Drop for Client<Stream>
            where Stream: $crate::rt::Stream,
        {
            fn drop(&mut self) {
                let _ = $crate::rt::send(&mut self.stream, &Request::_SHUTDOWN);
            }
        }

        impl<Stream> Methods for Client<Stream>
            where Stream: $crate::rt::Stream,
        {
            $(
                fn $fn_name(&mut self, $( $arg: $arg_ty ),*)
                    -> $crate::Result<$ret_ty>
                {
                    let req = Request::$fn_name($( $arg ),*);
                    try!($crate::rt::send(&mut self.stream, &req));
                    let ret = try!($crate::rt::recv(&mut self.stream));
                    Ok(ret)
                }
            )*
        }

        pub fn serve<Impl, Stream>(mut handler: Impl, mut stream: Stream)
            -> $crate::Result<()>
            where Stream: $crate::rt::Stream,
                  Impl: Methods,
        {
            loop {
                match try!($crate::rt::recv(&mut stream)) {
                    Request::_SHUTDOWN => return Ok(()),
                    $(
                        Request::$fn_name($( $arg ),*) => {
                            let ret = try!(handler.$fn_name($( $arg ),*));
                            try!($crate::rt::send(&mut stream, &ret));
                        }
                    )*
                };
            }
        }
    }}
}

// Run-time helpers for macro-generated code
#[doc(hidden)]
pub mod rt {
    use std::io::{Read, Write};
    use rustc_serialize::{Encodable, Decodable};
    use bincode::{self, SizeLimit};

    use super::Result;

    pub use bufstream::BufStream;

    pub trait Stream: Read + Write { }
    impl<T: Read + Write> Stream for T { }

    pub fn send<W, T>(w: &mut W, t: &T) -> Result<()>
        where W: Write, T: Encodable
    {
        try!(bincode::encode_into(t, w, SizeLimit::Infinite));
        try!(w.flush());
        Ok(())
    }

    pub fn recv<R, T>(r: &mut R) -> Result<T>
        where R: Read, T: Decodable
    {
        let r = try!(bincode::decode_from(r, SizeLimit::Infinite));
        Ok(r)
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ProtocolError,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, fmt)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref e) => error::Error::description(e),
            Error::ProtocolError => "urpc protocol error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref e) => Some(e),
            Error::ProtocolError => None,
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl convert::From<EncodingError> for Error {
    fn from(e: EncodingError) -> Error {
        match e {
            EncodingError::IoError(e) => Error::IoError(e),
            EncodingError::SizeLimit => unreachable!(),
        }
    }
}

impl convert::From<DecodingError> for Error {
    fn from(e: DecodingError) -> Error {
        match e {
            DecodingError::IoError(e) => Error::IoError(e),
            DecodingError::InvalidEncoding(_) => Error::ProtocolError,
            DecodingError::SizeLimit => unreachable!(),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
