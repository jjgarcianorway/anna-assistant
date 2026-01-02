// v0.0.648: Settings Encoder (Phase 224)
// Tests for settings encoder

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_encoding_format_display() {
        assert_eq!(format!("{}", EncodingFormat::Json), "json");
        assert_eq!(format!("{}", EncodingFormat::Toml), "toml");
    }

    #[test]
    fn test_encoding_options_display() {
        assert_eq!(format!("{}", EncodingOptions::Compact), "compact");
        assert_eq!(format!("{}", EncodingOptions::Pretty), "pretty");
    }

    #[test]
    fn test_config_new() {
        let c = EncoderConfig::new(EncodingFormat::Json);
        assert!(!c.sort_keys);
    }

    #[test]
    fn test_config_builder() {
        let c = EncoderConfig::new(EncodingFormat::Toml)
            .options(EncodingOptions::Pretty)
            .sort_keys(true);
        assert_eq!(c.options, EncodingOptions::Pretty);
        assert!(c.sort_keys);
    }

    #[test]
    fn test_result_new() {
        let r = EncodeResult::new("{}", EncodingFormat::Json, EncodingOptions::Compact);
        assert_eq!(r.byte_size, 2);
    }

    #[test]
    fn test_result_empty() {
        let r = EncodeResult::new("", EncodingFormat::Json, EncodingOptions::Compact);
        assert!(r.is_empty());
    }

    #[test]
    fn test_stats_record() {
        let mut s = EncoderStats::default();
        s.record(EncodingFormat::Json, EncodingOptions::Compact, 100);
        s.record(EncodingFormat::Toml, EncodingOptions::Pretty, 200);
        assert_eq!(s.total_encodes, 2);
        assert_eq!(s.total_bytes, 300);
    }

    #[test]
    fn test_encoder_new() {
        let e = SettingsEncoder::new(EncoderConfig::new(EncodingFormat::Json));
        assert_eq!(e.result_count(), 0);
    }

    #[test]
    fn test_encoder_encode_json() {
        let mut e = SettingsEncoder::new(EncoderConfig::new(EncodingFormat::Json));
        let settings = vec![("key".to_string(), "value".to_string())];
        let r = e.encode(&settings);
        assert!(r.data.contains("\"key\":\"value\""));
    }

    #[test]
    fn test_encoder_encode_toml() {
        let mut e = SettingsEncoder::new(EncoderConfig::new(EncodingFormat::Toml));
        let settings = vec![("key".to_string(), "value".to_string())];
        let r = e.encode(&settings);
        assert!(r.data.contains("key = \"value\""));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsEncoderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsEncoderRegistry::new();
        r.register("enc1", SettingsEncoder::new(EncoderConfig::new(EncodingFormat::Json)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_encoder_query() {
        assert!(is_encoder_query("settings encoder"));
        assert!(!is_encoder_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = encoder_fun_fact();
        assert!(fact.contains("encoder"));
    }
}
