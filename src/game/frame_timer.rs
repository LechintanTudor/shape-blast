use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct FrameTimerConfig {
    pub fixed_update_interval: Duration,
    pub max_fixed_update_accumulator: Duration,
}

impl FrameTimerConfig {
    pub fn new(fixed_updates_per_second: f64, max_fixed_updates_per_frame: f64) -> Self {
        let fixed_update_interval = Duration::from_secs(1).div_f64(fixed_updates_per_second);

        let max_fixed_update_accumulator =
            fixed_update_interval.mul_f64(max_fixed_updates_per_frame);

        Self {
            fixed_update_interval,
            max_fixed_update_accumulator,
        }
    }
}

impl Default for FrameTimerConfig {
    fn default() -> Self {
        Self::new(60.0, 3.0)
    }
}

#[derive(Clone, Debug)]
pub struct FrameTimer {
    config: FrameTimerConfig,
    frame_start: Instant,
    last_frame_duration: Duration,
    fixed_update_accumulator: Duration,
}

impl FrameTimer {
    pub fn new(config: FrameTimerConfig) -> Self {
        Self {
            config,
            frame_start: Instant::now(),
            last_frame_duration: Duration::ZERO,
            fixed_update_accumulator: Duration::ZERO,
        }
    }

    pub fn start_frame(&mut self) {
        let now = Instant::now();
        self.last_frame_duration = now - self.frame_start;

        self.fixed_update_accumulator += self.last_frame_duration;
        if self.fixed_update_accumulator > self.config.max_fixed_update_accumulator {
            self.fixed_update_accumulator = self.config.max_fixed_update_accumulator;
        }
    }

    pub fn fixed_update(&mut self) -> bool {
        let Some(fixed_update_accumulator) = self
            .fixed_update_accumulator
            .checked_sub(self.config.fixed_update_interval)
        else {
            return false;
        };

        self.fixed_update_accumulator = fixed_update_accumulator;
        true
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
