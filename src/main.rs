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
                // waiting for message
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
                                            // reset LEDs
                                            println!("LED init");
                                            if let Some(mut controller) =
                                                wires::build_controller(50)
                                            {
                                                wires::render_resting(&mut controller);
                                            };

                                            // new readings
                                            ioracle = ioracle.step();
                                            break;
                                        }
                                    };
                                }
                            }
                            Err(_) => {
                                // println!("LED update");
                                // thread::sleep(Duration::from_secs(1));
                                // if let Some(mut controller) = wires::build_controller(50) {
                                //     wires::render_resting(&mut controller);
                                // };
                            }
                        }
                        break;
                    }
                }
            }
            machine::IOracleWrapper::Reading(ref mut v) => {
                if let Some(mut controller) = wires::build_controller(255) {
                    let (h, r) = wires::reading(&mut controller);
                    v.hexagram = h;
                    v.related = r;
                }

                ioracle = ioracle.step();
            }
            machine::IOracleWrapper::Displaying(ref v) => {
                println!("---------------{:?}", v.hexagram);
                println!("---------------{:?}", v.related);

                match UnixStream::connect(IORACLE_OUT) {
                    Ok(mut stream) => {
                        let result = format!("{}|{}", &v.hexagram, &v.related).into_bytes();
                        if let Err(error) = stream.write_all(&result) {
                            println!("Can't write to stream: {:?}", error);
                        };
                    }
                    Err(error) => println!("Can't connect to output socket: {:?}", error),
                };

                // let start = SystemTime::now();
                // if let Some(mut controller) = wires::build_controller() {
                //     // loop {
                //     //     if let Ok(d) = start.elapsed() {
                //     //         if d > Duration::from_secs(120) {
                //     //             break;
                //     //         };
                //     //     }
                //     //
                //     //     wires::display_yao(&mut controller, &v.hexagram, &v.related);
                //     //     wires::display_li(&mut controller);
                //     //
                //     //     if let Err(e) = controller.render() {
                //     //         println!("{:?}", e);
                //     //     };
                //     // }
                //     wires::display_yao(&mut controller, &v.hexagram, &v.related);
                //     // wires::display_li(&mut controller);
                //
                //     if let Err(e) = controller.render() {
                //         println!("{:?}", e);
                //     };
                // }
                thread::sleep(Duration::from_secs(20));
                if let Some(mut controller) = wires::build_controller(255) {
                    wires::display_rel(&mut controller, &v.hexagram, &v.related);
                }
                thread::sleep(Duration::from_secs(80));

                println!("LED re-init");
                if let Some(mut controller) = wires::build_controller(50) {
                    wires::render_resting(&mut controller);
                };

                ioracle = ioracle.step();
            }
        };
    }
}
