mod machine;
mod wires;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Duration;
use std::{fs, process, thread};

const IORACLE_SEND: &str = "/tmp/ioracle.send";
const IORACLE_RETURN: &str = "/tmp/ioracle.return";

fn main() {
    // check socket
    if Path::new(IORACLE_SEND).exists() {
        if let Err(error) = fs::remove_file(IORACLE_SEND) {
            println!("{}", error);
            process::exit(1);
        };
    }

    // try to connect
    let listener = UnixListener::bind(IORACLE_SEND).unwrap_or_else(|error| {
        println!("{}", error);
        process::exit(1);
    });

    // create machine @ resting state
    let mut ioracle = machine::IOracleWrapper::Resting(machine::IOracle::new());

    // listen and react
    loop {
        match ioracle {
            machine::IOracleWrapper::Resting(_) => {
                if let Ok(_) = listener.set_nonblocking(true) {
                    // waiting for message
                    for stream in listener.incoming() {
                        if let Ok(stream) = stream {
                            let stream_reader = BufReader::new(stream);
                            for line in stream_reader.lines() {
                                if let Ok(line) = line {
                                    if line == "read" {
                                        // reset LEDs ???
                                        if let Some(mut controller) = wires::build_controller(50) {
                                            wires::render_resting(&mut controller);
                                        };

                                        // wating for user ???
                                        thread::sleep(Duration::from_secs(5));

                                        ioracle = ioracle.step();
                                        break;
                                    }
                                };
                            }
                        }
                        break;
                    }
                }
            }
            machine::IOracleWrapper::Reading(ref mut v) => {
                if let Some(mut controller) = wires::build_controller(255) {
                    let (hexagram, related) = wires::reading(&mut controller);
                    v.hexagram = hexagram;
                    v.related = related;
                }
                ioracle = ioracle.step();
            }
            machine::IOracleWrapper::Displaying(ref v) => {
                match UnixStream::connect(IORACLE_RETURN) {
                    Ok(mut stream) => {
                        let result = format!("{}|{}", &v.hexagram, &v.related).into_bytes();
                        if let Err(error) = stream.write_all(&result) {
                            println!("Can't write to RETURN stream: {:?}", error);
                        };
                    }
                    Err(error) => println!("Can't connect to RETURN socket: {:?}", error),
                };

                thread::sleep(Duration::from_secs(5)); //need 100
                if let Some(mut controller) = wires::build_controller(50) {
                    wires::render_resting(&mut controller);
                };
                ioracle = ioracle.step();
            }
        };
    }
}
