#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use crate::config::Config;

    #[test]
    fn test_default_config_creation() {
        let config_path = "default_config.json";
        let _ = fs::remove_file(config_path);

        let config = if let Ok(config_data) = fs::read_to_string(config_path) {
            serde_json::from_str(&config_data).unwrap()
        } else {
            let config = Config::default();
            fs::write(config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
            config
        };

        assert_eq!(config.verbose, false);
        assert_eq!(config.x32_ip, "192.168.1.64");

        let _ = fs::remove_file(config_path);
    }
}
