CREATE TABLE IF NOT EXISTS imported_files (
  id          INTEGER PRIMARY KEY,
  file_path   TEXT NOT NULL,
  file_hash   TEXT NOT NULL UNIQUE,
  imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS activities (
  id               INTEGER PRIMARY KEY,
  garmin_id        TEXT NOT NULL UNIQUE,
  name             TEXT,
  activity_type    TEXT,
  sport            TEXT,
  start_time       TEXT NOT NULL,
  duration_secs    REAL,
  distance_meters  REAL,
  calories         INTEGER,
  avg_hr           INTEGER,
  max_hr           INTEGER,
  avg_speed        REAL,
  elevation_gain   REAL,
  avg_cadence      INTEGER,
  avg_power        INTEGER,
  vo2max           REAL,
  source_file_hash TEXT
);

CREATE TABLE IF NOT EXISTS activity_records (
  id          INTEGER PRIMARY KEY,
  activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
  timestamp   TEXT,
  lat         REAL,
  lon         REAL,
  altitude    REAL,
  heart_rate  INTEGER,
  cadence     INTEGER,
  speed       REAL,
  power       INTEGER,
  distance    REAL
);

CREATE INDEX IF NOT EXISTS idx_activity_records_activity_id ON activity_records(activity_id);

CREATE TABLE IF NOT EXISTS daily_wellness (
  date              TEXT NOT NULL PRIMARY KEY,
  steps             INTEGER,
  floors            INTEGER,
  active_calories   INTEGER,
  resting_hr        INTEGER,
  avg_stress        INTEGER,
  body_battery_low  INTEGER,
  body_battery_high INTEGER,
  sleep_seconds     INTEGER,
  sleep_score       INTEGER,
  hrv_night_avg     REAL,
  spo2_avg          REAL,
  intensity_minutes INTEGER
);

CREATE TABLE IF NOT EXISTS sleep_sessions (
  id               INTEGER PRIMARY KEY,
  date             TEXT NOT NULL UNIQUE,
  sleep_start      TEXT,
  sleep_end        TEXT,
  total_sleep_secs INTEGER,
  deep_sleep_secs  INTEGER,
  light_sleep_secs INTEGER,
  rem_sleep_secs   INTEGER,
  awake_secs       INTEGER,
  sleep_score      INTEGER
);
