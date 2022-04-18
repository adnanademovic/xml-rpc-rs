extern crate xml_rpc;

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use xml_rpc::{call, Server};

fn main() {
    std::thread::spawn(|| {
        let mut server = Server::new();
        server.register_simple("foo", |()| {
            println!("foo start");
            std::thread::sleep(Duration::from_secs(5));
            println!("foo end");
            Ok(())
        });
        server.register_simple("bar", |()| {
            println!("bar start");
            std::thread::sleep(Duration::from_secs(5));
            println!("bar end");
            Ok(())
        });
        let bound_server = server
            .bind(&SocketAddr::new(
                IpAddr::V4("127.0.0.1".parse().unwrap()),
                5000,
            ))
            .unwrap();
        println!("Starting server!");
        bound_server.run();
    });

    std::thread::sleep(Duration::from_secs(1));

    let t1 = std::thread::spawn(|| {
        call::<_, _, ()>(&"http://127.0.0.1:5000".parse().unwrap(), "foo", ())
            .unwrap()
            .unwrap();
    });

    let t2 = std::thread::spawn(|| {
        call::<_, _, ()>(&"http://127.0.0.1:5000".parse().unwrap(), "bar", ())
            .unwrap()
            .unwrap();
    });

    let t3 = std::thread::spawn(|| {
        call::<_, _, ()>(&"http://127.0.0.1:5000".parse().unwrap(), "foo", ())
            .unwrap()
            .unwrap();
    });

    let t4 = std::thread::spawn(|| {
        call::<_, _, ()>(&"http://127.0.0.1:5000".parse().unwrap(), "bar", ())
            .unwrap()
            .unwrap();
    });

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
    t4.join().unwrap();
}
