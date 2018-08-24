// Copyright 2015 The coio Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate mio;

extern crate coio;

use clap::{App, Arg};

use coio::net::tcp::TcpListener;
use coio::Scheduler;

fn main() {
    env_logger::init().unwrap();

    let matches = App::new("tcp-echo")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Y. T. Chung <zonyitoo@gmail.com>")
        .arg(
            Arg::with_name("BIND")
                .short("b")
                .long("bind")
                .takes_value(true)
                .required(true)
                .help("Listening on this address"),
        ).arg(
            Arg::with_name("THREADS")
                .short("t")
                .long("threads")
                .takes_value(true)
                .help("Number of threads"),
        ).get_matches();

    let bind_addr = matches.value_of("BIND").unwrap().to_owned();

    Scheduler::new()
        .with_workers(matches.value_of("THREADS").unwrap_or("1").parse().unwrap())
        .run(move || {
            let server = TcpListener::bind(&bind_addr[..]).unwrap();

            info!("Listening on {:?}", server.local_addr().unwrap());

            for stream in server.incoming() {
                use std::io::{Read, Write};

                let (mut stream, addr) = stream.unwrap();
                info!("Accept connection: {:?}", addr);

                Scheduler::spawn(move || {
                    let mut buf = [0; 1024 * 16];

                    loop {
                        debug!("Trying to Read...");
                        match stream.read(&mut buf) {
                            Ok(0) => {
                                debug!("EOF received, going to close");
                                break;
                            }
                            Ok(len) => {
                                info!("Read {} bytes, echo back!", len);
                                stream.write_all(&buf[0..len]).unwrap();
                            }
                            Err(err) => {
                                panic!("Error occurs: {:?}", err);
                            }
                        }
                    }

                    info!("{:?} closed", addr);
                });
            }
        }).unwrap();
}
