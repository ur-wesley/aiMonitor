export type UsageWindowMetric = {
  readonly label: string;
  readonly value: number;
  readonly is_remaining_percent: boolean;
  readonly resets_at: string | null;
};

export type ProviderUsage = {
  readonly provider_id: string;
  readonly display_name: string;
  readonly windows: readonly UsageWindowMetric[];
  readonly error: string | null;
  readonly fetched_at: string;
};

export type UsageSnapshot = {
  readonly providers: readonly ProviderUsage[];
  readonly fetched_at: string;
};

export type AppSettings = {
  refresh_interval_seconds: number;
  local_api_enabled: boolean;
  local_api_port: number;
  opencode_go_api_key: string | null;
  cursor_state_db_path: string | null;
  antigravity_state_db_path: string | null;
  start_with_windows: boolean;
  low_usage_notifications_enabled: boolean;
};

export type CredentialHealth = {
  readonly cursor: string;
  readonly opencode_go: string;
  readonly antigravity: string;
};
