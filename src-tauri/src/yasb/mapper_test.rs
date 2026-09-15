#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::models::{ProviderUsage, UsageSnapshot, UsageWindowMetric};
    use crate::yasb;

    #[test]
    fn maps_cursor_fields() {
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
        assert_eq!(dto.cursor.total, Some(42.0));
        assert!((dto.max_used - 42.0).abs() < f64::EPSILON);
        assert_eq!(dto.status, "ok");
        assert!(dto.label.contains("C 42%"));
    }
}
