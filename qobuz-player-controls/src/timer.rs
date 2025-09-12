use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Timer {
    start_time: Option<Instant>,
    elapsed: Duration,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self {
            start_time: None,
            elapsed: Duration::ZERO,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.start_time = None;
        self.elapsed = Duration::ZERO;
    }

    pub(crate) fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
    }

    pub(crate) fn pause(&mut self) {
        if let Some(start) = self.start_time {
            self.elapsed += start.elapsed();
            self.start_time = None;
        }
    }

    pub(crate) fn elapsed(&self) -> Duration {
        match self.start_time {
            Some(start) => self.elapsed + start.elapsed(),
            None => self.elapsed,
        }
    }

    pub(crate) fn set_time(&mut self, time: Duration) {
        self.elapsed = time;

        if self.start_time.is_some() {
            self.start_time = Some(Instant::now());
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn approx_eq(d1: Duration, d2: Duration, tolerance_ms: u64) -> bool {
        let diff = d1.abs_diff(d2);
        diff <= Duration::from_millis(tolerance_ms)
    }

    #[test]
    fn test_resume_accumulates_time() {
        let mut sw = Timer::new();

        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        sw.pause();

        let first_elapsed = sw.elapsed();
        assert!(first_elapsed >= Duration::from_millis(100));

        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        sw.pause();

        let total_elapsed = sw.elapsed();
        assert!(total_elapsed >= Duration::from_millis(200));
    }

    #[test]
    fn test_pause_stops_time_accumulation() {
        let mut sw = Timer::new();

        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        sw.pause();

        let paused_elapsed = sw.elapsed();
        std::thread::sleep(Duration::from_millis(100));

        let after_wait_elapsed = sw.elapsed();
        assert!(
            approx_eq(paused_elapsed, after_wait_elapsed, 10),
            "Elapsed time should not increase while paused"
        );
    }
}
