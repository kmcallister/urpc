#![feature(unsafe_destructor, slice_patterns)]

extern crate rustc_serialize as rustc_serialize;

extern crate unix_socket;

#[macro_use]
extern crate urpc;

use std::thread;
use unix_socket::UnixStream;

urpc! {
    pub interface sort {
        fn sort(xs: Vec<u8>) -> Vec<u8> {}
    }
}

struct LocalSort;

impl sort::Methods for LocalSort {
    fn sort(&mut self, mut xs: Vec<u8>) -> urpc::Result<Vec<u8>> {
        xs.sort();
        Ok(xs)
    }
}

#[test]
fn local() {
    use sort::Methods;

    assert_eq!(vec![0, 3, 5, 6, 7, 8, 9],
        LocalSort.sort(vec![8, 6, 7, 5, 3, 0, 9]).unwrap());
    assert_eq!(vec![0; 10000],
        LocalSort.sort(vec![0; 10000]).unwrap());
}

#[test]
fn socket() {
    use sort::Methods;

    let [s1, s2] = UnixStream::unnamed().unwrap();
    let thread = thread::scoped(move || {
        let _ = sort::serve(LocalSort, s1);
    });

    let mut client = sort::Client::new(s2);
    assert_eq!(vec![0, 3, 5, 6, 7, 8, 9],
        client.sort(vec![8, 6, 7, 5, 3, 0, 9]).unwrap());
    assert_eq!(vec![0; 10000],
        client.sort(vec![0; 10000]).unwrap());

    drop(client);
    thread.join();
}
