#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::models::{ProviderUsage, UsageSnapshot, UsageWindowMetric};
    use crate::yasb;

    #[test]
    fn maps_cursor_fields_as_leftover() {
        let snapshot = UsageSnapshot {
            fetched_at: Utc::now(),
            providers: vec![ProviderUsage {
                provider_id: "cursor".into(),
                display_name: "Cursor".into(),
                windows: vec![
                    UsageWindowMetric {
                        label: "Total".into(),
                        value: 42.0,
                        is_remaining_percent: false,
                        resets_at: None,
                    },
                    UsageWindowMetric {
                        label: "Auto".into(),
                        value: 10.0,
                        is_remaining_percent: false,
                        resets_at: None,
                    },
                    UsageWindowMetric {
                        label: "API".into(),
                        value: 5.0,
                        is_remaining_percent: false,
                        resets_at: None,
                    },
                ],
                error: None,
                fetched_at: Utc::now(),
            }],
        };

        let dto = yasb::to_dto(&snapshot);
        assert_eq!(dto.cursor.total, Some(58.0));
        assert_eq!(dto.cursor.auto, Some(90.0));
        assert_eq!(dto.cursor.api, Some(95.0));
        assert!((dto.max_used - 42.0).abs() < f64::EPSILON);
        assert_eq!(dto.status, "ok");
        assert!(dto.label.contains("C 58%"));
        assert!(dto.tooltip.contains("Cursor total: 58% left"));
    }

    #[test]
    fn maps_opencode_used_and_antigravity_remaining() {
        let snapshot = UsageSnapshot {
            fetched_at: Utc::now(),
            providers: vec![
                ProviderUsage {
                    provider_id: "opencode_go".into(),
                    display_name: "OpenCode Go".into(),
                    windows: vec![UsageWindowMetric {
                        label: "5h".into(),
                        value: 10.0,
                        is_remaining_percent: false,
                        resets_at: None,
                    }],
                    error: None,
                    fetched_at: Utc::now(),
                },
                ProviderUsage {
                    provider_id: "antigravity".into(),
                    display_name: "Antigravity".into(),
                    windows: vec![UsageWindowMetric {
                        label: "Gemini 5h".into(),
                        value: 88.0,
                        is_remaining_percent: true,
                        resets_at: None,
                    }],
                    error: None,
                    fetched_at: Utc::now(),
                },
            ],
        };

        let dto = yasb::to_dto(&snapshot);
        assert_eq!(dto.opencode_go.rolling, Some(90.0));
        assert_eq!(dto.antigravity.gemini_5h, Some(88.0));
        assert!((dto.max_used - 10.0).abs() < f64::EPSILON);
        assert!(dto.label.contains("G 90%"));
        assert!(dto.label.contains("AG 88%"));
        assert!(dto.tooltip.contains("OpenCode Go 5h: 90% left"));
        assert!(dto.tooltip.contains("Antigravity Gemini 5h: 88% left"));
    }
}
