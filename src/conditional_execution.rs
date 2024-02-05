
#[derive(Debug)]
pub struct DetectChanges<T: PartialEq + Clone> {
    state: T,
}

impl<T: PartialEq + Clone> DetectChanges<T> {

    pub fn new(initial_state: T) -> Self {
        Self { state: initial_state }
    }

    pub fn state(&self) -> &T {
        &self.state
    }

    pub fn set_state(&mut self, state: T) {
        self.state = state
    }

    pub fn changed(&self, state: &T) -> bool {
        self.state != *state
    }

    pub fn note_change(&mut self, state: &T) -> bool {
        if self.changed(state) {
            self.set_state(state.clone());
            true
        }
        else { false }
    }
}


#[derive(Debug)]
pub struct Once {
    once: bool
}

impl Once {

    pub fn new() -> Self {
        Self { once: false }
    }

    pub fn call_once(&mut self, func: impl FnOnce()) {
        if !self.once {
            self.once = true;
            func();
        }
    }

    pub fn call_but_once(&mut self, func: impl FnOnce()) {
        if self.once { func() }
        else { self.once = true }
    }
}