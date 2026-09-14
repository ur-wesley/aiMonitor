using UsageTray.Models;

namespace UsageTray.Services;

public interface IUsageStore
{
    UsageSnapshot Current { get; }

    event EventHandler? Changed;

    void Update(UsageSnapshot snapshot);
}
