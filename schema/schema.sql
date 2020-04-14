CREATE TABLE IF NOT EXISTS covid_stats(
    id BIGSERIAL PRIMARY KEY,
    deaths BIGINT,
    total_cases BIGINT,
    recovered BIGINT,
    pdp BIGINT,
    at_date CHAR(20) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT NOW(),
    existed BOOL DEFAULT false
);

CREATE TABLE IF NOT EXISTS covid_daily(
    id BIGSERIAL PRIMARY KEY,
    day BIGINT,
    date TEXT NOT NULL UNIQUE,
    new_cases_per_day BIGINT,
    cumulative_cases BIGINT,
    under_treatment BIGINT,
    under_treatment_per_day BIGINT,
    under_treatment_percentage DOUBLE PRECISION,
    recovered BIGINT,
    recovered_per_day BIGINT,
    recovered_percentage DOUBLE PRECISION,
    deaths BIGINT,
    deaths_per_day BIGINT, 
    deaths_percentage DOUBLE PRECISION,
    latest_update TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    existed BOOL DEFAULT false
);

CREATE TABLE IF NOT EXISTS covid_province(
    id BIGSERIAL PRIMARY KEY,
    province_id BIGINT,
    date TEXT,
    provinsi TEXT,
    positif BIGINT,
    sembuh BIGINT,
    meninggal BIGINT,
    prov_and_date TEXT NOT NULL UNIQUE,
    existed BOOL DEFAULT false
);