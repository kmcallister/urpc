#![feature(unsafe_destructor, slice_patterns)]

extern crate rustc_serialize as rustc_serialize;

extern crate unix_socket;

#[macro_use]
extern crate urpc;

use std::thread;
use unix_socket::UnixStream;

urpc! {
    pub interface math {
        fn factorial(n: u64) -> u64 {}
        fn collatz(n: u64) -> u64 {}
    }
}

struct LocalMath;

impl math::Methods for LocalMath {
    fn factorial(&mut self, n: u64) -> urpc::Result<u64> {
        fn f(n: u64) -> u64 {
            match n {
                0 | 1 => 1,
                _ => n * f(n-1),
            }
        }
        Ok(f(n))
    }

    fn collatz(&mut self, mut n: u64) -> urpc::Result<u64> {
        let mut count = 0;
        while n > 1 {
            n = match n % 2 {
                0 => n/2,
                _ => 3*n + 1,
            };
            count += 1;
        }
        Ok(count)
    }
}

#[test]
fn local() {
    use math::Methods;
    assert_eq!(720, LocalMath.factorial(6).unwrap());
    assert_eq!(111, LocalMath.collatz(27).unwrap());
}

#[test]
fn socket() {
    use math::Methods;

    let [s1, s2] = UnixStream::unnamed().unwrap();
    let thread = thread::scoped(move || {
        let _ = math::serve(LocalMath, s1);
    });

    let mut client = math::Client::new(s2);
    assert_eq!(720, client.factorial(6).unwrap());
    assert_eq!(111, client.collatz(27).unwrap());

    drop(client);
    thread.join();
}
