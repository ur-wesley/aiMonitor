namespace UsageTray.Models;

public sealed record UsageWindowMetric(
    string Label,
    double Value,
    bool IsRemainingPercent,
    DateTimeOffset? ResetsAt);
