#[derive(Debug, Default)]
pub struct Timer {
    elapsed: f32,
    intervals: Vec<f32>,
    total: f32,
    pub index: usize,
}

impl Timer {
    pub fn new(intervals: Vec<f32>) -> Self {
        Self {
            total: intervals.iter().sum(),
            elapsed: 0.0,
            intervals,
            index: 0,
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        if self.intervals.is_empty() || self.total == 0.0 {
            return false;
        }
        let prev = self.index;
        self.elapsed += delta;
        self.elapsed %= self.total;
        while self.elapsed >= self.intervals[self.index] {
            self.elapsed -= self.intervals[self.index];
            self.index = (self.index + 1) % self.intervals.len();
        }
        self.index != prev
    }

    pub fn progress(&self) -> f32 {
        if self.intervals.is_empty() || self.intervals[self.index] == 0.0 {
            return 0.0;
        }
        self.elapsed / self.intervals[self.index]
    }
}
