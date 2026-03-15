use std::time::{Duration, Instant};

pub struct AppState {
    pub is_dimmed: bool,
    pub saved_lcd_bright: f32,
    pub saved_led_bright: f32,
    pub last_activity: Instant,
    pub timeout: Duration,
    pub is_connected: bool,
}

impl AppState {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            is_dimmed: false,
            saved_lcd_bright: Default::default(),
            saved_led_bright: Default::default(),
            last_activity: Instant::now(),
            timeout: Duration::from_secs(timeout_secs),
            is_connected: false,
        }
    }

    pub fn mark_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn should_dim(&self) -> bool {
        !self.is_dimmed && self.last_activity.elapsed() > self.timeout
    }

    pub fn should_restore(&self) -> bool {
        self.is_dimmed && self.last_activity.elapsed() <= self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = AppState::new(5);
        assert!(!state.is_dimmed);
        assert!(!state.should_dim());
        assert!(!state.should_restore());
    }

    #[test]
    fn test_should_dim_after_timeout() {
        let mut state = AppState::new(1);

        // Simulate time passing (in real app, Instant::now() moves)
        // For testing, we can artificially age the last_activity
        state.last_activity -= Duration::from_secs(2);

        assert!(state.should_dim());
        assert!(!state.should_restore());
    }

    #[test]
    fn test_should_restore_after_activity() {
        let mut state = AppState::new(1);
        state.is_dimmed = true;

        // Simulate recent activity
        state.mark_activity();

        assert!(!state.should_dim());
        assert!(state.should_restore());
    }
}
