mod contact;

//use contact::Contact;

const TICKS_PER_SECOND: usize = 1000;
const ONE_HOUR: usize = TICKS_PER_SECOND * 60 * 60;

//tickSize = number of milliseconds
pub struct Simulation {
    start_ticks: usize,
    end_ticks: usize,
    ticks: usize,
    tick_size: usize,
    running: bool,
}

// constructors
impl Simulation {
    pub fn new() -> Self {
        Simulation {
            start_ticks: 0,
            end_ticks: ONE_HOUR, 
            ticks: 0,
            tick_size: 1000,
            running: true,
        }
    } 
}

// impls
impl Simulation {
    pub fn tick(&mut self) -> bool {
        if !self.running { return false }

        self.increment_tick()
    }

    fn increment_tick(&mut self) -> bool {
        self.ticks += self.tick_size;

        if self.ticks >= self.end_ticks {
            self.ticks = self.end_ticks;
            self.running = false
        }
        true
    }
}
