use std::cell::Cell;

#[derive(Debug, Default)]
pub struct Timer {
    elapsed: Cell<f32>,
    intervals: Vec<f32>,
    total: f32,
    pub index: Cell<usize>,
}

impl Timer {
    pub fn new(intervals: Vec<f32>) -> Self {
        Self {
            total: intervals.iter().sum(),
            elapsed: 0.0.into(),
            intervals,
            index: 0.into(),
        }
    }

    pub fn tick(&self, delta: f32) -> bool {
        if self.intervals.is_empty() || self.total == 0.0 {
            return false;
        }
        let prev = self.index.get();
        self.elapsed.set((self.elapsed.get() + delta) % self.total);

        while self.elapsed.get() >= self.intervals[self.index.get()] {
            self.elapsed.set(self.elapsed.get() - self.intervals[self.index.get()]);
            self.index.set((self.index.get() + 1) % self.intervals.len())
        }

        self.index.get() != prev
    }

    pub fn progress(&self) -> f32 {
        if self.intervals.is_empty() || self.intervals[self.index.get()] == 0.0 {
            return 0.0;
        }
        self.elapsed.get() / self.intervals[self.index.get()]
    }
}
