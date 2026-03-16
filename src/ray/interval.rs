#[derive(Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    pub fn contains(&self, value: f64) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn surrounds(&self, value: f64) -> bool {
        self.min < value && value < self.max
    }

    pub fn clamp(&self, value: f64) -> f64 {
        value.max(self.min).min(self.max)
    }

    pub fn intersect(&self, other: &Interval) -> Option<Interval> {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);

        if min < max {
            Some(Interval { min, max })
        } else {
            None
        }
    }
}

pub const INFINITY_INTERVAL: Interval = Interval {
    min: f64::NEG_INFINITY,
    max: f64::INFINITY,
};

pub const EMPTY_INTERVAL: Interval = Interval {
    min: f64::INFINITY,
    max: f64::NEG_INFINITY,
};
