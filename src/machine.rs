// This is our state machine.
pub struct IOracle<S> {
    shared_value: usize,
    state: S,
}

// The states
pub struct Resting;
pub struct Reading;
pub struct Displaying;

// Initial state
impl IOracle<Resting> {
    pub fn new(shared_value: usize) -> Self {
        println!("start in resting");
        IOracle {
            shared_value: shared_value,
            state: Resting,
        }
    }
}

// Transitions between states
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

pub enum IOracleWrapper {
    Resting(IOracle<Resting>),
    Reading(IOracle<Reading>),
    Displaying(IOracle<Displaying>),
}

impl IOracleWrapper {
    pub fn step(mut self) -> Self {
        self = match self {
            IOracleWrapper::Resting(val) => IOracleWrapper::Reading(val.into()),
            IOracleWrapper::Reading(val) => IOracleWrapper::Displaying(val.into()),
            IOracleWrapper::Displaying(val) => IOracleWrapper::Resting(val.into()),
        };
        self
    }
}
