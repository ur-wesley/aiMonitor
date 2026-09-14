using UsageTray.Models;

namespace UsageTray.Services;

public sealed class UsageStore : IUsageStore
{
    private UsageSnapshot _current = UsageSnapshot.Empty;

    public UsageSnapshot Current => _current;

    public event EventHandler? Changed;

    public void Update(UsageSnapshot snapshot)
    {
        _current = snapshot;
        Changed?.Invoke(this, EventArgs.Empty);
    }
}
