
use crate::error::*;
use nosleep::{NoSleep, NoSleepType};

pub struct WakeLock {
    nosleep: NoSleep,
    active: bool,
}

impl WakeLock {

    pub fn new() -> Res<Self> {
        Ok(Self { nosleep: NoSleep::new().convert()?, active: false })
    }

    pub fn is_active(&self) -> bool { self.active }

    pub fn request(&mut self) -> Res<()> {
        self.nosleep.start(NoSleepType::PreventUserIdleDisplaySleep).convert()?;
        self.active = true;
        Ok(())
    }

    pub fn release(&mut self) -> Res<()> {
        // let res = self.nosleep.stop().convert();
        *self = Self::new()?;
        // res
        Ok(())
    }
}