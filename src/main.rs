mod machine;
mod wires;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Duration;
use std::{process, thread};

const IORACLE_IN: &str = "/tmp/ioracle.in";
const IORACLE_OUT: &str = "/tmp/ioracle.out";

fn main() {
    let socket = Path::new(IORACLE_IN);
    if socket.exists() {
        if let Err(error) = std::fs::remove_file(IORACLE_IN) {
            println!("{}", error);
            process::exit(1);
        };
    }

    let listener = UnixListener::bind(IORACLE_IN).unwrap_or_else(|error| {
        println!("{}", error);
        process::exit(1);
    });

    let mut ioracle = machine::IOracleWrapper::Resting(machine::IOracle::new());

    loop {
        match ioracle {
            machine::IOracleWrapper::Resting(_) => {
                if let Ok(_) = listener.set_nonblocking(true) {
                    for stream in listener.incoming() {
                        // println!("new stream");
                        match stream {
                            Ok(stream) => {
                                let stream_reader = BufReader::new(stream);
                                for line in stream_reader.lines() {
                                    // println!("new line");
                                    if let Ok(line) = line {
                                        if line == "read" {
                                            ioracle = ioracle.step();
                                            break;
                                        }
                                    };
                                }
                            }
                            Err(_) => {
                                // println!("LED update");
                                if let Some(mut controller) = wires::build_controller() {
                                    wires::render_resting(&mut controller);
                                };
                            }
                        }
                        break;
                    }
                }
            }
            // machine::IOracleWrapper::Reading(_) => {
            machine::IOracleWrapper::Reading(ref mut v) => {
                println!("---------------{:?}", v.hexagram);
                if let Some(mut controller) = wires::build_controller() {
                    wires::reading(&mut controller);
                }
                v.hexagram = "111111".to_string();
                ioracle = ioracle.step();
            }
            machine::IOracleWrapper::Displaying(ref v) => {
                println!("displaying now");
                println!("---------------{:?}", v.hexagram);

                match UnixStream::connect(IORACLE_OUT) {
                    Ok(mut stream) => {
                        match stream.write_all(b"100100") {
                            Ok(_) => {
                                println!("result is send");
                            }
                            Err(e) => println!("{:?}", e),
                        };
                    }
                    Err(e) => println!("{:?}", e),
                };
                thread::sleep(Duration::from_secs(8));

                ioracle = ioracle.step();
            }
        };
    }
}
