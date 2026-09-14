using Microsoft.Data.Sqlite;

namespace aiMonitor.Services.Auth;

public static class SqliteTokenReader
{
    public static async Task<string?> ReadValueAsync(string dbPath, string key, CancellationToken ct = default)
    {
        if (!File.Exists(dbPath))
            return null;

        await using var connection = new SqliteConnection($"Data Source={dbPath};Mode=ReadOnly");
        await connection.OpenAsync(ct).ConfigureAwait(false);

        await using var command = connection.CreateCommand();
        command.CommandText = "SELECT value FROM ItemTable WHERE key = $key LIMIT 1";
        command.Parameters.AddWithValue("$key", key);

        var result = await command.ExecuteScalarAsync(ct).ConfigureAwait(false);
        return result as string;
    }
}
