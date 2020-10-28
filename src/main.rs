mod machine;
mod wires;

use rand::distributions::{Distribution, Uniform};
// use serialport::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;
use std::time::Duration;

// const LEDS_IN_LINE: i32 = 144;
const IORACLE_GATE: &str = "/tmp/ioracle.gate";
const IORACLE_OUT: &str = "/tmp/ioracle.out";

fn main() {
    let socket = Path::new(IORACLE_GATE);
    if socket.exists() {
        if let Err(error) = std::fs::remove_file(IORACLE_GATE) {
            println!("{}", error);
            std::process::exit(1);
        };
    }

    let listener = UnixListener::bind(IORACLE_GATE).unwrap_or_else(|error| {
        println!("{}", error);
        std::process::exit(1);
    });

    // let serial_port_settings = SerialPortSettings {
    //     baud_rate: 9600,
    //     data_bits: DataBits::Eight,
    //     flow_control: FlowControl::None,
    //     parity: Parity::None,
    //     stop_bits: StopBits::One,
    //     timeout: Duration::from_secs(1),
    // };

    let mut state = machine::IOracleWrapper::Resting(machine::IOracle::new(0));

    loop {
        match state {
            machine::IOracleWrapper::Resting(_) => {
                if let Ok(_) = listener.set_nonblocking(true) {
                    for stream in listener.incoming() {
                        // println!("new stream");
                        match stream {
                            Ok(stream) => {
                                let stream_reader = BufReader::new(stream);
                                for line in stream_reader.lines() {
                                    // println!("new line");
                                    let l = line.unwrap();
                                    // println!("{}", l);

                                    if l == "read" {
                                        state = state.step();
                                        break;
                                    }
                                }
                            }
                            Err(_) => {
                                println!("LED update");
                                // --------------------------------------------------
                                // let mut serial_value;
                                // if let Ok(mut port) =
                                //     serialport::open_with_settings("/dev/ttyACM0",
                                //     &serial_port_settings)
                                // {
                                //     let mut serial_buf: Vec<u8> = vec![0; 512];
                                //
                                //     match port.read(serial_buf.as_mut_slice()) {
                                //         Ok(t) => {
                                //             serial_value = wires::get_val(&serial_buf[..t]);
                                //         }
                                //         Err(e) => eprintln!("{:?}", e),
                                //     }
                                // }
                                // --------------------------------------------------
                                if let Some(mut controller) = wires::build_controller() {
                                    let mut rng1 = rand::thread_rng();
                                    let mut rng2 = rand::thread_rng();

                                    let yao = controller.leds_mut(0);
                                    let red_range = Uniform::from(54..255);

                                    let mut k;
                                    for i in 0..yao.len() - 1 {
                                        k = i * 9;
                                        // !!!???
                                        if k > yao.len() - 9 {
                                            k = yao.len() - 9;
                                        }
                                        for j in k..k + 9 {
                                            let r = red_range.sample(&mut rng1);
                                            let green_range = Uniform::from(0..r / 4);
                                            let g = green_range.sample(&mut rng2);
                                            yao[j as usize] = [0, g, r, 0];
                                        }
                                    }

                                    std::thread::sleep(Duration::from_millis(70));

                                    if let Err(e) = controller.render() {
                                        println!("Fire error: {:?}", e);
                                    }
                                };
                                // --------------------------------------------------
                            }
                        }
                        break;
                    }
                }
            }
            machine::IOracleWrapper::Reading(_) => {
                if let Some(mut controller) = wires::build_controller() {
                    wires::reading(&mut controller);
                }
                state = state.step();
            }
            machine::IOracleWrapper::Displaying(_) => {
                println!("displaying now");

                match UnixStream::connect(IORACLE_OUT) {
                    Ok(mut st) => {
                        match st.write_all(b"100100") {
                            Ok(_) => {
                                println!("result is send");
                            }
                            Err(e) => println!("{:?}", e),
                        };
                    }
                    Err(e) => println!("{:?}", e),
                };

                thread::sleep(Duration::from_secs(8));

                state = state.step();
            }
        };
    }
}
