using Microsoft.Data.SqlClient;

namespace DotnetMcpServer.Data;

public class DatabaseConfig
{
    public static string GetConnectionString()
    {
        // 1. Explicit connection string override
        var directConn = Environment.GetEnvironmentVariable("DB_CONNECTION_STRING");
        if (!string.IsNullOrWhiteSpace(directConn))
        {
            return directConn;
        }

        // 2. Composable environment parameters
        var server = Environment.GetEnvironmentVariable("DB_SERVER") ?? "localhost";
        var database = Environment.GetEnvironmentVariable("DB_NAME") ?? "InternAgenticDB";
        var user = Environment.GetEnvironmentVariable("DB_USER");
        var password = Environment.GetEnvironmentVariable("DB_PASSWORD");
        var trustServerCertStr = Environment.GetEnvironmentVariable("DB_TRUST_SERVER_CERTIFICATE") ?? "true";
        bool.TryParse(trustServerCertStr, out bool trustServerCert);

        var builder = new SqlConnectionStringBuilder
        {
            DataSource = server,
            InitialCatalog = database,
            TrustServerCertificate = trustServerCert,
            IntegratedSecurity = string.IsNullOrWhiteSpace(user)
        };

        if (!string.IsNullOrWhiteSpace(user))
        {
            builder.UserID = user;
            builder.Password = password ?? string.Empty;
            builder.IntegratedSecurity = false;
        }

        return builder.ConnectionString;
    }
}
