use fsm_rs::fsm;

use crate::peripherals;
use crate::peripherals::Shared;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TriggerMode {
    SAFE,
    SEMI,
    AUTO,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PowerON {}

impl PowerON {
    pub fn entry(&self) -> Result<(), &'static str> {
        unreachable!()
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        Ok(State::Ready(Ready {}))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct POSTError {}

impl POSTError {
    pub fn entry(&self) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ready {}

impl Ready {
    pub fn entry(&self) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryVoltageLow {}

impl BatteryVoltageLow {
    pub fn entry(&self) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Overcurrent {}

impl Overcurrent {
    pub fn entry(&self) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Preloading {}

impl Preloading {
    pub fn entry(&self) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Safe {}

impl Safe {
    pub fn entry(&self) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HalfAutoNFire {}

impl HalfAutoNFire {
    pub fn entry(&self) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FullAutoFire {}

impl FullAutoFire {
    pub fn entry(&self) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn exit(&self) -> Result<State, &'static str> {
        unreachable!()
    }
}

// event
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Post {}

impl Post {
    pub fn on(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryVoltageChange {}

impl BatteryVoltageChange {
    pub fn on(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemCurrentChange {}

impl SystemCurrentChange {
    pub fn on(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PullHalfTrigger {}

impl PullHalfTrigger {
    pub fn on(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PullFullTrigger {}

impl PullFullTrigger {
    pub fn on(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseTrigger {}

impl ReleaseTrigger {
    pub fn on(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Default)]
pub struct FireControl {
    shard: Option<Mutex<RefCell<Shared>>>,
}

impl Default for State {
    fn default() -> Self {
        State::PowerON(PowerON {})
    }
}

fsm! {
    Context = FireControl;

    States {
        PowerON = PowerON,
        POSTError = POSTError,
        Ready = Ready,
        BatteryVoltageLow = BatteryVoltageLow,
        Overcurrent = Overcurrent,
        Preloading = Preloading,
        Safe = Safe,
        HalfAutoNFire = HalfAutoNFire,
        FullAutoFire = FullAutoFire,
    }

    Events {
        POST = Post,
        BatteryVoltageChange = BatteryVoltageChange,
        SystemCurrentChange = SystemCurrentChange,
        PullHalfTrigger = PullHalfTrigger,
        PullFullTrigger = PullFullTrigger,
        ReleaseTrigger = ReleaseTrigger,
    }

    Transitions {
        POST [
            PowerON => Ready,
            PowerON => POSTError,
        ],
        BatteryVoltageChange [
            Ready => BatteryVoltageLow,
            Safe => BatteryVoltageLow,
            Preloading => BatteryVoltageLow,
            HalfAutoNFire => BatteryVoltageLow,
            FullAutoFire => BatteryVoltageLow,
        ],
        SystemCurrentChange [
            Preloading => Overcurrent,
            HalfAutoNFire => Overcurrent,
            FullAutoFire => Overcurrent,
        ],
        PullHalfTrigger [
            Ready => Preloading,
        ],
        PullFullTrigger [
            Preloading => Safe,
            Preloading => HalfAutoNFire,
            Preloading => FullAutoFire,
        ],
        ReleaseTrigger [
            Safe => Ready,
            HalfAutoNFire => Ready,
            FullAutoFire => Ready,
        ],
    }
}
