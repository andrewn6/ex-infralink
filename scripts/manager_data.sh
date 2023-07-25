#!/bin/bash

DATABASE_URL="db_url"

SQL_COMMANDS=$(cat <<SQL
CREATE TABLE IF NOT EXISTS Providers (
    id SERIAL PRIMARY KEY,
    provider TEXT NOT NULL,
    region TEXT NOT NULL,
    instance_count INT NOT NULL,
    UNIQUE(provider, region)
);

INSERT INTO Providers (provider, region, instance_count)
VALUES ('vultr', 'us-west', 1)
ON CONFLICT (provider, region) DO NOTHING;

INSERT INTO Providers (provider, region, instance_count)
VALUES ('vultr', 'us-east', 1)
ON CONFLICT (provider, region) DO NOTHING;

INSERT INTO Providers (provider, region, instance_count)
VALUES ('hetzner', 'eu-west', 1)
ON CONFLICT (provider, region) DO NOTHING;

INSERT INTO Providers (provider, region, instance_count)
VALUES ('hetzner', 'eu-central', 1)
ON CONFLICT (provider, region) DO NOTHING;
SQL
)

# Execute the SQL commands using the psql tool
echo "$SQL_COMMANDS" | psql "$DATABASE_URL"