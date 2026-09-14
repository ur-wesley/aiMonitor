using aiMonitor.Models;

namespace aiMonitor.Services;

public interface IUsageStore
{
    UsageSnapshot Current { get; }

    event EventHandler? Changed;

    void Update(UsageSnapshot snapshot);
}
