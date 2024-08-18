
use crate::time::{Instant, Duration};


#[derive(Debug, Clone)]
pub struct NormInterval {
  pub instant: Instant,
  pub duration: Duration,
}

impl NormInterval {

    pub fn new(duration: Duration) -> Self {
        Self { instant: Instant::now(), duration }
    }

    pub fn from_secs(duration_secs: f64) -> Self {
        Self::new(Duration::from_secs_f64(duration_secs))
    }

    pub fn elapsed(&self) -> f64 {
        let now = Instant::now();
        if now >= self.instant {
            (now - self.instant).as_secs_f64() / self.duration.as_secs_f64()
        } else {
            -((self.instant - now).as_secs_f64() / self.duration.as_secs_f64())
        }
    }

    pub fn advance_by(&mut self, times: f64) {
        if let Some(instant) = {
            if times.is_sign_positive() {
                self.instant.checked_add(self.duration.mul_f64(times))
            } else {
                self.instant.checked_sub(self.duration.mul_f64(-times))
            }
        } {
            self.instant = instant
        }
    }

    pub fn advance_by_full_elapsed(&mut self) -> f64 {
        let elapsed = self.elapsed();
        if !(0.0..1.0).contains(&elapsed) {
            self.advance_by(elapsed.floor());
        }
        elapsed
    }
}



#[derive(Debug, Clone)]
pub struct StepInterval {
    pub next: Instant,
    pub duration: Duration,
}

impl StepInterval {

    pub fn new(duration: Duration) -> Self {
        Self { next: Instant::now() + duration, duration }
    }

    pub fn from_secs(duration_secs: f64) -> Self {
        Self::new(Duration::from_secs_f64(duration_secs))
    }

    pub fn elapsed(&self) -> i64 {
        let now = Instant::now();
        if now >= self.next {
            ((now - self.next).as_nanos() / self.duration.as_nanos()) as i64 + 1
        } else {
            -(((self.next - now).as_nanos() / self.duration.as_nanos()) as i64)
        }
    }

    pub fn step_by(&mut self, times: i64) {
        if let Some(instant) = {
            if times.is_positive() {
                self.next.checked_add(self.duration.mul_f64(times as f64))
            } else {
                self.next.checked_sub(self.duration.mul_f64(-times as f64))
            }
        } {
            self.next = instant
        }
    }

    pub fn step_if_elapsed(&mut self) -> i64 {
        let elapsed = self.elapsed();
        if elapsed >= 1 { self.step_by(elapsed) }
        elapsed
    }

    pub fn step_next(&mut self) -> i64 {
        let elapsed = self.elapsed();
        // step so that now < next <= now + elapsed.frac
        self.step_by(if elapsed >= 1 { elapsed } else { elapsed - 1 });
        elapsed
    }
}



#[derive(Debug, Clone)]
pub struct IntervalCounter {
    pub count: usize,
    pub interval: StepInterval,
}

#[derive(Debug, Clone, Copy)]
pub struct IntervalCount {
    pub count: usize,
    pub times_per_sec: f64,
}

impl IntervalCounter {

    pub fn new(duration: Duration) -> Self {
        Self { count: 0, interval: StepInterval::new(duration) }
    }

    pub fn from_secs(duration_secs: f64) -> Self {
        Self::new(Duration::from_secs_f64(duration_secs))
    }

    pub fn add(&mut self) {
        self.count += 1;
    }

    pub fn count(&mut self) -> Option<IntervalCount> {
        if self.interval.step_if_elapsed() >= 1 {

            let counted = IntervalCount {
                times_per_sec: self.count as f64 / self.interval.duration.as_secs_f64() ,
                count: self.count,
            };

            self.count = 0;

            Some(counted)
        }
        else { None }
    }
}