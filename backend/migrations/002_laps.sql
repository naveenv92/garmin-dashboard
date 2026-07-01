CREATE TABLE IF NOT EXISTS activity_laps (
  id               INTEGER PRIMARY KEY,
  activity_id      INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
  lap_index        INTEGER NOT NULL,
  start_time       TEXT,
  end_time         TEXT,
  duration_secs    REAL,
  distance_meters  REAL,
  elevation_gain   REAL,
  elevation_loss   REAL,
  max_speed        REAL,
  avg_speed        REAL,
  avg_hr           INTEGER,
  max_hr           INTEGER,
  calories         INTEGER,
  min_altitude     REAL,
  max_altitude     REAL
);

CREATE INDEX IF NOT EXISTS idx_activity_laps_activity_id ON activity_laps(activity_id);
