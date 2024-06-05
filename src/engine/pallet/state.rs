pub struct RuntimeVersion {
    spec_version: i32,
    transaction_version: i32,
}

impl RuntimeVersion {
    pub fn new(spec_version: i32, transaction_version: i32) -> Self {
        Self {
            spec_version,
            transaction_version,
        }
    }

    // Getters
    pub fn spec_version(&self) -> i32 {
        self.spec_version
    }

    pub fn transaction_version(&self) -> i32 {
        self.transaction_version
    }
}


pub struct State {

}

impl State {
    pub fn new() -> Self {
        State {  }
    }
}