#[cfg(test)]
mod mapper_test;

use chrono::Local;

use crate::models::{
    ProviderUsage, UsageSnapshot, YasbAntigravityDto, YasbCursorDto, YasbExportDto, YasbOpenCodeGoDto,
};

pub fn to_dto(snapshot: &UsageSnapshot) -> YasbExportDto {
    let mut dto = YasbExportDto {
        cursor: YasbCursorDto::default(),
        opencode_go: YasbOpenCodeGoDto::default(),
        antigravity: YasbAntigravityDto::default(),
        max_used: 0.0,
        status: "ok".into(),
        fetched_at: snapshot.fetched_at.with_timezone(&Local).format("%m/%d/%Y %I:%M %p").to_string(),
        label: String::new(),
        label_alt: String::new(),
        tooltip: String::new(),
        status_class: "ok".into(),
    };

    let mut max_used: f64 = 0.0;
    let mut min_remaining: f64 = 100.0;

    for provider in &snapshot.providers {
        match provider.provider_id.as_str() {
            "cursor" => {
                map_cursor(provider, &mut dto.cursor);
                if let Some(total) = dto.cursor.total {
                    max_used = max_used.max(total);
                }
            }
            "opencode_go" => {
                map_opencode(provider, &mut dto.opencode_go);
                for window in &provider.windows {
                    if !window.is_remaining_percent {
                        max_used = max_used.max(window.value);
                    }
                }
            }
            "antigravity" => {
                map_antigravity(provider, &mut dto.antigravity);
                for window in &provider.windows {
                    if window.is_remaining_percent {
                        min_remaining = min_remaining.min(window.value);
                    }
                }
            }
            _ => {}
        }
    }

    dto.max_used = max_used;
    dto.status = if snapshot.providers.iter().any(|p| p.error.is_some()) {
        "partial".into()
    } else {
        "ok".into()
    };
    dto.status_class = status_class(max_used, min_remaining);
    dto.label = build_label(&dto);
    dto.label_alt = build_label_alt(&dto);
    dto.tooltip = build_tooltip(&dto);
    round_numbers(&mut dto);
    dto
}

fn map_cursor(provider: &ProviderUsage, dto: &mut YasbCursorDto) {
    dto.error.clone_from(&provider.error);
    if provider.error.is_some() {
        return;
    }
    dto.total = get_value(provider, "Total");
    dto.auto = get_value(provider, "Auto");
    dto.api = get_value(provider, "API");
    dto.resets_at = get_reset(provider).map(|r| {
        r.with_timezone(&Local)
            .format("%m/%d/%Y %I:%M %p")
            .to_string()
    });
}

fn map_opencode(provider: &ProviderUsage, dto: &mut YasbOpenCodeGoDto) {
    dto.error.clone_from(&provider.error);
    if provider.error.is_some() {
        return;
    }
    dto.rolling = get_value(provider, "5h");
    dto.weekly = get_value(provider, "Weekly");
    dto.monthly = get_value(provider, "Monthly");
}

fn map_antigravity(provider: &ProviderUsage, dto: &mut YasbAntigravityDto) {
    dto.error.clone_from(&provider.error);
    if provider.error.is_some() {
        return;
    }
    dto.gemini_5h = get_value(provider, "Gemini 5h");
    dto.gemini_weekly = get_value(provider, "Gemini Weekly");
    dto.claude_5h = get_value(provider, "Claude/GPT 5h");
    dto.claude_weekly = get_value(provider, "Claude/GPT Weekly");
}

fn get_value(provider: &ProviderUsage, label: &str) -> Option<f64> {
    provider
        .windows
        .iter()
        .find(|w| w.label.eq_ignore_ascii_case(label))
        .map(|w| w.value)
}

fn get_reset(provider: &ProviderUsage) -> Option<chrono::DateTime<chrono::Utc>> {
    provider.windows.iter().find_map(|w| w.resets_at)
}

fn status_class(max_used: f64, min_remaining: f64) -> String {
    if max_used >= 95.0 || min_remaining <= 5.0 {
        "critical".into()
    } else if max_used >= 80.0 || min_remaining <= 20.0 {
        "near-limit".into()
    } else {
        "ok".into()
    }
}

fn round_numbers(dto: &mut YasbExportDto) {
    dto.cursor.total = dto.cursor.total.map(f64::round);
    dto.cursor.auto = dto.cursor.auto.map(f64::round);
    dto.cursor.api = dto.cursor.api.map(f64::round);
    dto.opencode_go.rolling = dto.opencode_go.rolling.map(f64::round);
    dto.opencode_go.weekly = dto.opencode_go.weekly.map(f64::round);
    dto.opencode_go.monthly = dto.opencode_go.monthly.map(f64::round);
    dto.antigravity.gemini_5h = dto.antigravity.gemini_5h.map(f64::round);
    dto.antigravity.gemini_weekly = dto.antigravity.gemini_weekly.map(f64::round);
    dto.antigravity.claude_5h = dto.antigravity.claude_5h.map(f64::round);
    dto.antigravity.claude_weekly = dto.antigravity.claude_weekly.map(f64::round);
}

fn format_used(value: Option<f64>, error: Option<&str>) -> String {
    if error.is_some() {
        "—".into()
    } else if let Some(v) = value {
        format!("{:.0}%", v.round())
    } else {
        "—".into()
    }
}

fn format_remaining(value: Option<f64>, error: Option<&str>) -> String {
    format_used(value, error)
}

fn build_label(dto: &YasbExportDto) -> String {
    format!(
        "C {} · G {} · AG {}",
        format_used(dto.cursor.total, dto.cursor.error.as_deref()),
        format_used(dto.opencode_go.rolling, dto.opencode_go.error.as_deref()),
        format_remaining(dto.antigravity.gemini_5h, dto.antigravity.error.as_deref()),
    )
}

fn build_label_alt(dto: &YasbExportDto) -> String {
    let cursor = if dto.cursor.error.is_some() {
        "Cursor —".into()
    } else {
        format!(
            "Cursor {} / auto {}",
            format_used(dto.cursor.total, None),
            format_used(dto.cursor.auto, None),
        )
    };

    let go = if dto.opencode_go.error.is_some() {
        "Go —".into()
    } else {
        format!(
            "Go 5h {} / wk {}",
            format_used(dto.opencode_go.rolling, None),
            format_used(dto.opencode_go.weekly, None),
        )
    };

    let ag = if dto.antigravity.error.is_some() {
        "AG —".into()
    } else {
        format!(
            "AG G5h {} / C5h {}",
            format_remaining(dto.antigravity.gemini_5h, None),
            format_remaining(dto.antigravity.claude_5h, None),
        )
    };

    format!("{cursor} · {go} · {ag}")
}

fn build_tooltip(dto: &YasbExportDto) -> String {
    let mut lines = Vec::new();

    if let Some(err) = &dto.cursor.error {
        lines.push(format!("Cursor: {err}"));
    } else {
        lines.push(format!("Cursor total: {}", format_used(dto.cursor.total, None)));
        lines.push(format!(
            "Cursor auto: {} · API: {}",
            format_used(dto.cursor.auto, None),
            format_used(dto.cursor.api, None),
        ));
        if let Some(reset) = &dto.cursor.resets_at {
            lines.push(format!("Cursor resets: {reset}"));
        }
    }

    if let Some(err) = &dto.opencode_go.error {
        lines.push(format!("OpenCode Go: {err}"));
    } else {
        lines.push(format!(
            "OpenCode Go 5h: {}",
            format_used(dto.opencode_go.rolling, None),
        ));
        lines.push(format!(
            "OpenCode Go weekly: {} · monthly: {}",
            format_used(dto.opencode_go.weekly, None),
            format_used(dto.opencode_go.monthly, None),
        ));
    }

    if let Some(err) = &dto.antigravity.error {
        lines.push(format!("Antigravity: {err}"));
    } else {
        lines.push(format!(
            "Antigravity Gemini 5h: {} left",
            format_remaining(dto.antigravity.gemini_5h, None),
        ));
        lines.push(format!(
            "Antigravity Gemini weekly: {} left",
            format_remaining(dto.antigravity.gemini_weekly, None),
        ));
        lines.push(format!(
            "Antigravity Claude/GPT 5h: {} left",
            format_remaining(dto.antigravity.claude_5h, None),
        ));
        lines.push(format!(
            "Antigravity Claude/GPT weekly: {} left",
            format_remaining(dto.antigravity.claude_weekly, None),
        ));
    }

    lines.push(format!("Updated: {}", dto.fetched_at));
    lines.join("\n")
}
