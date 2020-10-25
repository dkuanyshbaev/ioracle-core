use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use std::time::Duration;

const IORACLE_GATE: &str = "/tmp/ioracle.gate";

// This is our state machine.
struct IOracle<S> {
    shared_value: usize,
    state: S,
}

// The states
struct Resting;
struct Reading;
struct Displaying;

// fn handle_client(stream: UnixStream) {
//     let stream = BufReader::new(stream);
//     for line in stream.lines() {
//         println!("{}", line.unwrap());
//     }
// }

fn main() {
    // if let Err(error) = std::fs::remove_file(IORACLE_GATE) {
    //     println!("{}", error);
    //     std::process::exit(1);
    // };
    // let listener = UnixListener::bind(IORACLE_GATE).unwrap_or_else(|error| {
    //     println!("{}", error);
    //     std::process::exit(1);
    // });

    use std::fs;
    use std::path::Path;
    let socket = Path::new(IORACLE_GATE);
    // Delete old socket if necessary
    if socket.exists() {
        // fs::unlink(&socket).unwrap();
        if let Err(error) = std::fs::remove_file(IORACLE_GATE) {
            println!("{}", error);
            std::process::exit(1);
        };
    }
    let listener = UnixListener::bind(IORACLE_GATE).unwrap_or_else(|error| {
        println!("{}", error);
        std::process::exit(1);
    });

    // let mut the_factory = Factory::new();
    let mut a = IOracleWrapper::Resting(IOracle::new(0));

    loop {
        // the_factory.i_oracle = the_factory.i_oracle.step();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // thread::spawn(|| handle_client(stream));
                    let stream = BufReader::new(stream);
                    for line in stream.lines() {
                        let l = line.unwrap();
                        println!("{}", l);

                        if l == "read" {
                            for i in 1..4 {
                                a = a.step();
                                thread::sleep(Duration::from_secs(1));
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("---- Error: {}", err);
                    break;
                }
            }
        }

        // for i in 1..4 {
        //     a = a.step();
        //     thread::sleep(Duration::from_secs(1));
        // }
        // thread::sleep(Duration::from_secs(4));
    }
}

// Our Machine starts in the 'Resting' state.
impl IOracle<Resting> {
    fn new(shared_value: usize) -> Self {
        println!("start in resting");
        IOracle {
            shared_value: shared_value,
            state: Resting,
            // here the work
        }
    }
}

// The following are the defined transitions between states.
impl From<IOracle<Resting>> for IOracle<Reading> {
    fn from(val: IOracle<Resting>) -> IOracle<Reading> {
        println!("resting -> reading");
        IOracle {
            shared_value: val.shared_value,
            state: Reading,
        }
    }
}

impl From<IOracle<Reading>> for IOracle<Displaying> {
    fn from(val: IOracle<Reading>) -> IOracle<Displaying> {
        println!("reading -> displaying");
        IOracle {
            shared_value: val.shared_value,
            state: Displaying,
        }
    }
}

impl From<IOracle<Displaying>> for IOracle<Resting> {
    fn from(val: IOracle<Displaying>) -> IOracle<Resting> {
        println!("displaying -> resting");
        IOracle {
            shared_value: val.shared_value,
            state: Resting,
        }
    }
}

// Here is we're building an enum so we can contain this state machine in a parent.
enum IOracleWrapper {
    Resting(IOracle<Resting>),
    Reading(IOracle<Reading>),
    Displaying(IOracle<Displaying>),
}

// Defining a function which shifts the state along.
impl IOracleWrapper {
    fn step(mut self) -> Self {
        self = match self {
            IOracleWrapper::Resting(val) => IOracleWrapper::Reading(val.into()),
            IOracleWrapper::Reading(val) => IOracleWrapper::Displaying(val.into()),
            IOracleWrapper::Displaying(val) => IOracleWrapper::Resting(val.into()),
        };
        self
    }
}

// The structure with a parent.
struct Factory {
    i_oracle: IOracleWrapper,
}

impl Factory {
    fn new() -> Self {
        Factory {
            i_oracle: IOracleWrapper::Resting(IOracle::new(0)),
        }
    }
}
