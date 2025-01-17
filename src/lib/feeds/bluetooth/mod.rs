use std::{fs, time::Duration};

use anyhow::{anyhow, Result};

struct State<'a> {
    prefix: &'a str,
    device_state: Option<DeviceState>,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            device_state: None,
        }
    }
}

impl<'a> crate::pipeline::State for State<'a> {
    type Event = Option<DeviceState>;

    fn update(
        &mut self,
        msg: Self::Event,
    ) -> Result<Option<Vec<crate::alert::Alert>>> {
        self.device_state = msg;
        Ok(None)
    }

    fn display<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        // TODO Distinguish between OffSoft and OffHard
        let symbol = match self.device_state {
            Some(DeviceState::NoDev) | None => "--",
            Some(DeviceState::On) => "on",
            Some(DeviceState::OffSoft | DeviceState::OffHard) => "off",
        };
        writeln!(buf, "{}{}", self.prefix, symbol)?;
        Ok(())
    }
}

#[derive(Debug)]
enum DeviceState {
    NoDev,
    OffHard,
    OffSoft,
    On,
}

impl DeviceState {
    // TODO Lookup number of connected devices.

    fn read() -> Result<Option<Self>> {
        // This method of device state lookup is taken from TLP bluetooth command.
        // TODO Checkout https://crates.io/crates/bluer
        let mut bt_state_opt: Option<Self> = None;
        for entry in fs::read_dir("/sys/class/rfkill/")? {
            let entry = entry?;
            let mut path_type = entry.path();
            let mut path_state = entry.path();
            path_type.push("type");
            path_state.push("state");
            if let "bluetooth" = fs::read_to_string(path_type)?.trim_end() {
                let state_byte: u8 =
                    fs::read_to_string(path_state)?.trim_end().parse()?;
                bt_state_opt = Some(Self::from_byte(state_byte)?);
                return Ok(bt_state_opt);
            }
        }
        Ok(bt_state_opt)
    }

    fn from_byte(b: u8) -> Result<Self> {
        match b {
            0 => Ok(Self::OffSoft),
            1 => Ok(Self::On),
            2 => Ok(Self::OffHard),
            254 => Ok(Self::NoDev),
            _ => Err(anyhow!("Invalid state byte: {:?}", b)),
        }
    }
}

pub fn run(prefix: &str, interval: Duration) -> Result<()> {
    use crate::clock;

    let events = clock::new(interval)
        .map(|clock::Tick| DeviceState::read())
        .filter_map(|result| match result {
            Err(err) => {
                tracing::error!("Failed to read device state: {:?}", err);
                None
            }
            Ok(dev_opt) => Some(dev_opt),
        });
    crate::pipeline::run_to_stdout(events, State::new(prefix))
}
