use std::borrow::Cow;
use std::time::{Duration, Instant};

/// AppState holds the internal state of the x32_tapw TUI application.
pub struct AppState {
    pub ip: String,
    pub slot: u8,
    pub channel: u8,
    pub threshold: f32,
    pub is_auto: bool,
    pub last_tap: Option<Instant>,
    pub was_above_threshold: bool,
    pub current_delay_ms: Option<i32>,
    pub logs: Vec<Cow<'static, str>>,
    pub is_connected: bool,
    pub active_input: InputMode,
    pub should_quit: bool,
    pub delay_type: String,

    // IP input editing state
    pub ip_input: String,
    pub slot_input: String,
    pub ch_input: String,
    pub sens_input: String,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    EditingIp,
    EditingSlot,
    EditingChannel,
    EditingSensitivity,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ip: "192.168.0.64".to_string(),
            slot: 1,
            channel: 1,
            threshold: 0.5,
            is_auto: false,
            last_tap: None,
            was_above_threshold: false,
            current_delay_ms: None,
            logs: Vec::new(),
            is_connected: false,
            active_input: InputMode::Normal,
            should_quit: false,
            delay_type: "Check".to_string(),

            ip_input: "192.168.0.64".to_string(),
            slot_input: "1".to_string(),
            ch_input: "1".to_string(),
            sens_input: "0.5".to_string(),
        }
    }

    pub fn log(&mut self, msg: String) {
        self.logs.push(Cow::Owned(msg));
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn calculate_fval(delta_ms: f32) -> f32 {
        (delta_ms / 3000.0).clamp(0.0, 1.0)
    }

    pub fn handle_manual_tap(&mut self, now: Instant) -> Option<f32> {
        if let Some(last) = self.last_tap {
            let delta = now.duration_since(last);
            let delta_ms = delta.as_millis() as f32;
            let f_val = Self::calculate_fval(delta_ms);
            self.current_delay_ms = Some((f_val * 3000.0) as i32);
            self.last_tap = Some(now);
            Some(f_val)
        } else {
            self.last_tap = Some(now);
            None
        }
    }

    pub fn parse_meter_blob(data: &[u8]) -> Option<f32> {
        if data.len() >= 16 {
            let mut f_bytes = [0u8; 4];
            f_bytes.copy_from_slice(&data[12..16]);
            Some(f32::from_le_bytes(f_bytes))
        } else {
            None
        }
    }

    pub fn process_meter_data(&mut self, level: f32, now: Instant) -> Option<f32> {
        if level > self.threshold {
            if !self.was_above_threshold {
                self.was_above_threshold = true;
                if let Some(last) = self.last_tap {
                    let delta = now.duration_since(last);
                    let delta_ms = delta.as_millis() as f32;

                    // Minimum resolution is 60ms to avoid rapid-fire updates
                    if delta_ms > 60.0 {
                        let f_val = Self::calculate_fval(delta_ms);
                        self.current_delay_ms = Some((f_val * 3000.0) as i32);
                        self.last_tap = Some(now);
                        return Some(f_val);
                    }
                } else {
                    self.last_tap = Some(now);
                }
            }
        } else {
            self.was_above_threshold = false;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fval() {
        assert_eq!(AppState::calculate_fval(0.0), 0.0);
        assert_eq!(AppState::calculate_fval(1500.0), 0.5);
        assert_eq!(AppState::calculate_fval(3000.0), 1.0);
        assert_eq!(AppState::calculate_fval(6000.0), 1.0);
        assert_eq!(AppState::calculate_fval(-100.0), 0.0);
    }

    #[test]
    fn test_manual_tap() {
        let mut app = AppState::new();
        let now = Instant::now();
        assert_eq!(app.handle_manual_tap(now), None);
        assert_eq!(app.last_tap, Some(now));

        let now2 = now + Duration::from_millis(1500);
        let fval = app.handle_manual_tap(now2);
        assert_eq!(fval, Some(0.5));
        assert_eq!(app.current_delay_ms, Some(1500));
        assert_eq!(app.last_tap, Some(now2));
    }

    #[test]
    fn test_parse_meter_blob() {
        let mut blob = vec![0u8; 16];
        let level = 0.75f32;
        blob[12..16].copy_from_slice(&level.to_le_bytes());
        assert_eq!(AppState::parse_meter_blob(&blob), Some(0.75));

        let short_blob = vec![0u8; 10];
        assert_eq!(AppState::parse_meter_blob(&short_blob), None);
    }

    #[test]
    fn test_process_meter_data() {
        let mut app = AppState::new();
        app.threshold = 0.5;
        let start = Instant::now();

        assert_eq!(app.process_meter_data(0.8, start), None);
        assert!(app.was_above_threshold);
        assert_eq!(app.last_tap, Some(start));

        assert_eq!(app.process_meter_data(0.9, start + Duration::from_millis(10)), None);
        assert_eq!(app.process_meter_data(0.2, start + Duration::from_millis(20)), None);
        assert!(!app.was_above_threshold);

        let fval = app.process_meter_data(0.8, start + Duration::from_millis(1000));
        assert!(app.was_above_threshold);
        assert!(fval.is_some());
        let val = fval.unwrap();
        assert!((val - 0.3333333).abs() < 0.0001);
        assert_eq!(app.current_delay_ms, Some(1000));

        app.was_above_threshold = false;
        assert_eq!(app.process_meter_data(0.8, start + Duration::from_millis(1020)), None);
    }
}
