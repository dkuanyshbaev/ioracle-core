mod machine;
mod wires;

use machine::{IOracle, IOracleWrapper};
use rand::distributions::{Distribution, Uniform};
use serialport::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime};
// use wires::*;

const LEDS_IN_LINE: i32 = 144;
const IORACLE_GATE: &str = "/tmp/ioracle.gate";
const IORACLE_OUT: &str = "/tmp/ioracle.out";

// // This is our state machine.
// struct IOracle<S> {
//     shared_value: usize,
//     state: S,
// }
//
// // The states
// struct Resting;
// struct Reading;
// struct Displaying;
//
// impl IOracle<Resting> {
//     fn new(shared_value: usize) -> Self {
//         println!("start in resting");
//         IOracle {
//             shared_value: shared_value,
//             state: Resting,
//         }
//     }
// }
//
// // Transitions between states
// impl From<IOracle<Resting>> for IOracle<Reading> {
//     fn from(val: IOracle<Resting>) -> IOracle<Reading> {
//         println!("resting -> reading");
//         IOracle {
//             shared_value: val.shared_value,
//             state: Reading,
//         }
//     }
// }
//
// impl From<IOracle<Reading>> for IOracle<Displaying> {
//     fn from(val: IOracle<Reading>) -> IOracle<Displaying> {
//         println!("reading -> displaying");
//         IOracle {
//             shared_value: val.shared_value,
//             state: Displaying,
//         }
//     }
// }
//
// impl From<IOracle<Displaying>> for IOracle<Resting> {
//     fn from(val: IOracle<Displaying>) -> IOracle<Resting> {
//         println!("displaying -> resting");
//         IOracle {
//             shared_value: val.shared_value,
//             state: Resting,
//         }
//     }
// }
//
// // Here is we're building an enum so we can contain this state machine in a parent.
// enum IOracleWrapper {
//     Resting(IOracle<Resting>),
//     Reading(IOracle<Reading>),
//     Displaying(IOracle<Displaying>),
// }
//
// // Defining a function which shifts the state along.
// impl IOracleWrapper {
//     fn step(mut self) -> Self {
//         self = match self {
//             IOracleWrapper::Resting(val) => IOracleWrapper::Reading(val.into()),
//             IOracleWrapper::Reading(val) => IOracleWrapper::Displaying(val.into()),
//             IOracleWrapper::Displaying(val) => IOracleWrapper::Resting(val.into()),
//         };
//         self
//     }
// }

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

    let s = SerialPortSettings {
        baud_rate: 9600,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_secs(1),
    };

    let mut state = IOracleWrapper::Resting(IOracle::new(0));

    loop {
        match state {
            IOracleWrapper::Resting(_) => {
                // println!("resting and listeninig");

                if let Ok(_) = listener.set_nonblocking(true) {
                    for stream in listener.incoming() {
                        println!("new stream");
                        match stream {
                            Ok(stream) => {
                                let stream_reader = BufReader::new(stream);
                                for line in stream_reader.lines() {
                                    println!("new line");
                                    let l = line.unwrap();
                                    println!("{}", l);

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
                                //     serialport::open_with_settings("/dev/ttyACM0", &s)
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
            IOracleWrapper::Reading(_) => {
                // --------------------------------------------------
                println!("New reading.");

                if let Some(mut controller) = wires::build_controller() {
                    let yao = controller.leds_mut(0);

                    for num in 0..LEDS_IN_LINE * 6 {
                        yao[num as usize] = [0, 0, 0, 0];
                    }

                    if let Err(e) = controller.render() {
                        println!("{:?}", e);
                    };

                    //---------------------------------------------------

                    let line1 =
                        wires::read(2, "1".to_string(), "500".to_string(), "10".to_string());
                    println!("line1 = {}", line1);
                    wires::render(line1, 6, &mut controller, &"rgb(51, 0, 180)".to_string());
                    thread::sleep(Duration::from_secs(3));

                    let line2 =
                        wires::read(2, "1".to_string(), "500".to_string(), "10".to_string());
                    println!("line2 = {}", line2);
                    wires::render(line2, 1, &mut controller, &"rgb(51, 0, 180)".to_string());
                    thread::sleep(Duration::from_secs(3));

                    let line3 =
                        wires::read(2, "1".to_string(), "500".to_string(), "10".to_string());
                    println!("line3 = {}", line3);
                    wires::render(line3, 2, &mut controller, &"rgb(51, 0, 180)".to_string());
                    thread::sleep(Duration::from_secs(3));

                    // pub fn render_first(&self, settings: &Binding, controller: &mut Controller) {
                    // reaction
                    // get related lines
                    // get related trigram

                    let line4 =
                        wires::read(2, "1".to_string(), "500".to_string(), "10".to_string());
                    println!("line4 = {}", line4);
                    wires::render(line4, 3, &mut controller, &"rgb(51, 0, 180)".to_string());
                    thread::sleep(Duration::from_secs(3));

                    let line5 =
                        wires::read(2, "1".to_string(), "500".to_string(), "10".to_string());
                    println!("line5 = {}", line5);
                    wires::render(line5, 4, &mut controller, &"rgb(51, 0, 180)".to_string());
                    thread::sleep(Duration::from_secs(3));

                    let line6 =
                        wires::read(2, "1".to_string(), "500".to_string(), "10".to_string());
                    println!("line6 = {}", line6);
                    wires::render(line6, 5, &mut controller, &"rgb(51, 0, 180)".to_string());
                    thread::sleep(Duration::from_secs(3));
                    //---------------------------------------------------

                    // reaction
                    // get related lines
                    // get related trigram

                    // reset pins
                    // return hex + rel
                }

                state = state.step();
            }
            IOracleWrapper::Displaying(_) => {
                println!("displaying now");

                if let Some(mut controller) = wires::build_controller() {
                    let yao = controller.leds_mut(0);

                    for num in 0..LEDS_IN_LINE * 6 {
                        yao[num as usize] = [255, 255, 255, 0];
                    }

                    if let Err(e) = controller.render() {
                        println!("{:?}", e);
                    };
                }

                state = state.step();

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
            }
        };
    }
}
