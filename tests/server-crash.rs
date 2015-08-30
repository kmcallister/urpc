extern crate rustc_serialize as rustc_serialize;

extern crate unix_socket;
extern crate libc;

#[macro_use]
extern crate urpc;

use unix_socket::UnixStream;

urpc! {
    pub interface oops {
        fn oops(x: u8) -> u8 {}
    }
}

struct Whoops;

impl oops::Methods for Whoops {
    fn oops(&mut self, x: u8) -> urpc::Result<u8> {
        assert_eq!(x, 42);
        unsafe {
            libc::_exit(1);
        }
    }
}

#[test]
fn server_crash() {
    use oops::Methods;

    let (s1, s2) = UnixStream::unnamed().unwrap();

    let pid = unsafe { libc::fork() };
    assert!(pid >= 0);

    match pid {
        0 => {
            drop(s2);
            oops::serve(Whoops, s1).unwrap();
        }
        _ => {
            drop(s1);
            let mut client = oops::Client::new(s2);
            assert!(client.oops(42).is_err());
        }
    }
}
