extern crate xml_rpc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::{net, thread};
use xml_rpc::{Server, Client};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TestStruct {
    pub foo: i32,
    pub bar: String,
}

pub fn main() {
    let socket = net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut server = Server::new();
    server.register_simple("echo", |v: TestStruct| Ok(v));
    server.register_simple("double", |v: TestStruct| {
        Ok(TestStruct {
            foo: v.foo * 2,
            bar: format!("{0}{0}", v.bar),
        })
    });
    thread::spawn(move || server.run(&socket).unwrap());
    let mut client = Client::new().unwrap();
    let req = TestStruct {
        foo: 42,
        bar: "baz".into(),
    };
    println!("Sending: {:?}", req);
    let uri = "http://localhost:8080/".parse().unwrap();
    let res: Result<Result<TestStruct, _>, _> = client.call(&uri, "echo", req.clone());
    println!("Echo Received: {:?}", res);
    let res: Result<Result<TestStruct, _>, _> = client.call(&uri, "double", req.clone());
    println!("Double Received: {:?}", res);
    let res: Result<Result<TestStruct, _>, _> = client.call(&uri, "invalid", req.clone());
    println!("Invalid Received: {:?}", res);
}
