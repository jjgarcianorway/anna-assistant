// v0.0.649: Settings Decoder Tests (Phase 225)
// Tests for decoder functionality

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::settings_decoder::*;

    #[test]
    fn test_decoding_format_display() {
        assert_eq!(format!("{}", DecodingFormat::Json), "json");
        assert_eq!(format!("{}", DecodingFormat::Toml), "toml");
    }

    #[test]
    fn test_decoding_mode_display() {
        assert_eq!(format!("{}", DecodingMode::Strict), "strict");
        assert_eq!(format!("{}", DecodingMode::Lenient), "lenient");
    }

    #[test]
    fn test_config_new() {
        let c = DecoderConfig::new(DecodingFormat::Json);
        assert_eq!(c.mode, DecodingMode::Strict);
    }

    #[test]
    fn test_config_builder() {
        let c = DecoderConfig::new(DecodingFormat::Toml)
            .mode(DecodingMode::Lenient)
            .allow_unknown(true);
        assert_eq!(c.mode, DecodingMode::Lenient);
        assert!(c.allow_unknown);
    }

    #[test]
    fn test_error_new() {
        let e = DecodeError::new("test error").at(10);
        assert_eq!(e.position, Some(10));
    }

    #[test]
    fn test_result_success() {
        let mut values = HashMap::new();
        values.insert("key".to_string(), "value".to_string());
        let r = DecodeResult::success(values, DecodingFormat::Json);
        assert!(r.success);
        assert_eq!(r.value_count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = DecoderStats::default();
        s.record(DecodingFormat::Json, true, 5);
        s.record(DecodingFormat::Json, false, 0);
        assert_eq!(s.total_decodes, 2);
        assert_eq!(s.successful, 1);
    }

    #[test]
    fn test_decoder_new() {
        let d = SettingsDecoder::new(DecoderConfig::new(DecodingFormat::Json));
        assert_eq!(d.result_count(), 0);
    }

    #[test]
    fn test_decoder_decode_json() {
        let mut d = SettingsDecoder::new(DecoderConfig::new(DecodingFormat::Json));
        let r = d.decode(r#"{"key":"value"}"#);
        assert!(r.success);
        assert_eq!(r.values.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_decoder_decode_toml() {
        let mut d = SettingsDecoder::new(DecoderConfig::new(DecodingFormat::Toml));
        let r = d.decode("key = \"value\"");
        assert!(r.success);
        assert_eq!(r.values.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsDecoderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsDecoderRegistry::new();
        r.register("dec1", SettingsDecoder::new(DecoderConfig::new(DecodingFormat::Json)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_decoder_query() {
        assert!(is_decoder_query("settings decoder"));
        assert!(!is_decoder_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = decoder_fun_fact();
        assert!(fact.contains("decoder"));
    }
}
