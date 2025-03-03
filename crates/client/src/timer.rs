

#[derive(Clone, Copy, Default, Debug)]
pub enum Repeat {

    /// Finite number of repetitions
    Finite(u16),

    /// Looping
    #[default]
    Infinite,
}


#[derive(Clone, Debug, Default)]
pub struct Timer {
    pub index: usize,
    pub repeat: Repeat,
    pub elapsed: f32,
    pub intervals: Vec<f32>,
    pub total: f32,
    pub repeat_count: u16,
    pub complete: bool,
}

impl Timer {
    pub fn new(intervals: Vec<f32>) -> Self {
        Self {
            total: intervals.iter().sum(),
            elapsed: 0.0,
            intervals,
            index: 0,
            repeat: Repeat::Infinite,
            repeat_count: 0,
            complete: false,
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        if self.intervals.is_empty() || self.total == 0.0 {
            return false;
        }
        match self.repeat {
            Repeat::Finite(count) => {
                if self.repeat_count >= count {
                    return false;
                }
            }
            _ => {}
        }

        let prev = self.index;
        self.elapsed += delta;
        if self.elapsed >= self.total {
            self.repeat_count += (self.elapsed / self.total).trunc() as u16;
            self.elapsed %= self.total
        }

        while self.elapsed >= self.intervals[self.index] {
            self.elapsed = (self.elapsed - self.intervals[self.index]);
            self.index += 1;
            if self.index >= self.intervals.len() {
                self.repeat_count += 1;
                self.index = 0;
            }
        }

        match self.repeat {
            Repeat::Finite(count) => {
                if self.repeat_count >= count {
                    self.index = self.intervals.len() - 1;
                    self.elapsed  = self.intervals[self.index];
                    return false;
                }
            }
            _ => {}
        }

        true
    }

    pub fn progress(&self) -> f32 {
        if self.intervals.is_empty() || self.intervals[self.index] == 0.0 {
            return 0.0;
        }
        self.elapsed / self.intervals[self.index]
    }
}
