use ratatui::layout::Rect;
use std::borrow::Cow;
use std::time::Instant;
use x32_lib::MixerModel;

/// AppState holds the internal state of the x32_tapw TUI application.
pub struct AppState {
    #[allow(dead_code)]
    pub ip: String,
    pub model: MixerModel,
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

    // Input editing state
    pub ip_input: String,
    pub slot_input: String,
    pub ch_input: String,
    pub sens_input: String,

    // Formatted strings to avoid per-frame format allocations in the TUI loop
    pub display_ip_text: String,
    pub display_mode_text: String,
    pub display_slot_text: String,
    pub display_logs_text: String,

    pub cached_area: Rect,
    pub cached_chunks: Vec<Vec<Rect>>,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    EditingIp,
    EditingSlot,
    EditingChannel,
    EditingSensitivity,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::with_options(MixerModel::X32, "192.168.0.64", 1, 1, 0.5, false)
    }

    pub fn with_options(
        model: MixerModel,
        ip: &str,
        slot: u8,
        channel: u8,
        threshold: f32,
        is_auto: bool,
    ) -> Self {
        let mut state = Self {
            ip: ip.to_string(),
            model,
            slot,
            channel,
            threshold,
            is_auto,
            last_tap: None,
            was_above_threshold: false,
            current_delay_ms: None,
            logs: Vec::new(),
            is_connected: false,
            active_input: InputMode::Normal,
            should_quit: false,
            delay_type: "Check".to_string(),

            ip_input: ip.to_string(),
            slot_input: slot.to_string(),
            ch_input: channel.to_string(),
            sens_input: format!("{:.1}", threshold),

            display_ip_text: String::with_capacity(64),
            display_mode_text: String::with_capacity(64),
            display_slot_text: String::with_capacity(64),
            display_logs_text: String::with_capacity(1024),

            cached_area: Rect::default(),
            cached_chunks: Vec::new(),
        };
        state.update_display_strings();
        state
    }

    pub fn update_display_strings(&mut self) {
        self.display_ip_text.clear();
        use std::fmt::Write;
        let _ = write!(
            self.display_ip_text,
            "IP: {} ({})",
            self.ip_input, self.model
        );

        self.display_mode_text.clear();
        let _ = write!(
            self.display_mode_text,
            "Mode: {}\nCheck: {}",
            if self.is_auto { "Auto" } else { "Manual" },
            self.delay_type
        );

        self.display_slot_text.clear();
        let _ = write!(self.display_slot_text, "Delay Slot: {}", self.slot_input);
    }

    pub fn log(&mut self, msg: String) {
        self.logs.push(Cow::Owned(msg));
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }

        self.display_logs_text.clear();
        use std::fmt::Write;
        for log_msg in &self.logs {
            let _ = writeln!(self.display_logs_text, "{}", log_msg);
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
            if let Some(slice) = data.get(12..16) {
                f_bytes.copy_from_slice(slice);
            } else {
                return None;
            }
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
    use std::time::Duration;

    #[test]
    fn test_calculate_fval() {
        assert_eq!(AppState::calculate_fval(0.0), 0.0);
        assert_eq!(AppState::calculate_fval(1500.0), 0.5);
        assert_eq!(AppState::calculate_fval(3000.0), 1.0);
        assert_eq!(AppState::calculate_fval(6000.0), 1.0);
        assert_eq!(AppState::calculate_fval(-100.0), 0.0);

        // Edge cases
        assert!(AppState::calculate_fval(f32::NAN).is_nan());
        assert_eq!(AppState::calculate_fval(f32::INFINITY), 1.0);
        assert_eq!(AppState::calculate_fval(f32::NEG_INFINITY), 0.0);
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
    fn test_log() {
        let mut app = AppState::new();

        app.log("first log".to_string());
        assert_eq!(app.logs.len(), 1);
        assert_eq!(app.logs[0], "first log");

        for i in 0..101 {
            app.log(format!("log {}", i));
        }

        assert_eq!(app.logs.len(), 100);
        assert_eq!(app.logs[0], "log 1");
        assert_eq!(app.logs[99], "log 100");
    }

    #[test]
    fn test_process_meter_data() {
        let mut app = AppState::new();
        app.threshold = 0.5;
        let start = Instant::now();

        assert_eq!(app.process_meter_data(0.8, start), None);
        assert!(app.was_above_threshold);
        assert_eq!(app.last_tap, Some(start));

        assert_eq!(
            app.process_meter_data(0.9, start + Duration::from_millis(10)),
            None
        );
        assert_eq!(
            app.process_meter_data(0.2, start + Duration::from_millis(20)),
            None
        );
        assert!(!app.was_above_threshold);

        let fval = app.process_meter_data(0.8, start + Duration::from_millis(1000));
        assert!(app.was_above_threshold);
        assert!(fval.is_some());
        let val = fval.unwrap_or(0.0);
        assert!((val - 0.3333333).abs() < 0.0001);
        assert_eq!(app.current_delay_ms, Some(1000));

        app.was_above_threshold = false;
        assert_eq!(
            app.process_meter_data(0.8, start + Duration::from_millis(1020)),
            None
        );
    }

    #[test]
    fn test_with_options() {
        let app = AppState::with_options(MixerModel::Wing, "10.0.0.1", 3, 5, 0.7, true);
        assert_eq!(app.model, MixerModel::Wing);
        assert_eq!(app.ip_input, "10.0.0.1");
        assert_eq!(app.slot, 3);
        assert_eq!(app.channel, 5);
        assert_eq!(app.threshold, 0.7);
        assert!(app.is_auto);
        assert_eq!(app.display_ip_text, "IP: 10.0.0.1 (Wing)");
    }
}
