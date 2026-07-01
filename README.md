# Garmin Dashboard

A self-hosted dashboard for your own Garmin Connect data export — activities, daily wellness (steps, stress, body battery), and sleep.

- **Backend**: Rust ([axum](https://github.com/tokio-rs/axum) + [sqlx](https://github.com/launchbadge/sqlx)/SQLite) — imports a Garmin Connect export into a local database and serves a REST API.
- **Frontend**: React + Vite + Tailwind, charts via [recharts](https://recharts.org).

## Getting your data

Request a data export from Garmin Connect: **Settings → Account Management → Export Your Data** on [connect.garmin.com](https://connect.garmin.com). Garmin emails you a zip once it's ready (can take a while). Unzip it somewhere, e.g. `~/Downloads/garmin-export/`.

The importer walks the export directory recursively and picks up:

| Source | Contents |
|---|---|
| `SummarizedActivities.json` | Activity list |
| `*.fit` | Individual activity + GPS/HR/power track |
| `*.gpx` | Individual activity + GPS track |
| `DI_CONNECT/DI-Connect-Aggregator/UDSFile_*.json` | Daily steps, calories, resting HR, stress, body battery, intensity minutes |
| `DI_CONNECT/DI-Connect-Wellness/*_sleepData.json` | Sleep sessions (stages, score, timing) |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+

## Usage

**1. Import your export:**

```bash
cd backend
cargo run --release -- import ~/Downloads/garmin-export
```

This creates `backend/garmin.db` (SQLite). Files are hashed, so re-running the import is safe — already-imported files are skipped.

**2. Run the backend:**

```bash
cargo run --release -- serve --port 3001 --db garmin.db
```

**3. Run the frontend** (separate terminal):

```bash
cd frontend
npm install
npm run dev
```

Open the printed Vite URL (usually `http://localhost:5173`) — API calls are proxied to `localhost:3001`.

For a single deployable binary, run `npm run build` in `frontend/` first; the backend serves the built `frontend/dist` directly, so you only need to run `cargo run --release -- serve` afterward.

## Notes

- `backend/garmin.db` is gitignored — it contains your personal health and location data and should never be committed.
- Import is idempotent; drop `backend/garmin.db` and re-run `import` any time to rebuild from scratch.
