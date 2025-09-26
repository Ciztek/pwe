# EpiCovid Frontend (dev)

This folder contains the React + Vite frontend for the EpiCovid dashboard.

Quick start (run from `frontend`):

## Install dependencies

```bash
pnpm install
# or npm install
```

## Run the dev server

```bash
pnpm run dev
# or npm run dev
```

What I changed

- Added `recharts` and `date-fns` to `package.json` for charts and date formatting
- Added a small mock API in `src/services/mockApi.ts` that returns synthetic Covid time series
- New components: `src/components/Dashboard.tsx` and `src/components/Charts.tsx`

Notes for future API integration

- The mock service returns an array with { date, place, confirmed, deaths, recovered }.
  When the real backend API is available, replace `fetchMockCovidSeries()` with a real fetch
  and adapt fields if needed. The components expect ISO `date` strings and numeric values.

<!-- Truncated template content. See project root README for more details. -->

## Packaging: Desktop (Tauri) and Windows installer

- The Tauri config (`src-tauri/tauri.conf.json`) is set to build platform bundles. For Windows, it uses NSIS and embeds the WebView2 offline installer so end users won't see the WebView2Loader.dll missing error.
- By default, the installer targets the current user (no admin required). You can switch to machine-wide installs by changing `bundle.windows.nsis.installMode` if needed.

### Build on Windows (recommended)

Run the native build orchestrator from this `frontend` folder:

```bash
./scripts/build-native.sh tauri-windows
```

Artifacts are collected under `.build/tauri/windows/`:

- NSIS installer (recommended for distribution)
- Raw exe (for quick tests; still requires the WebView2 runtime)

If you run the raw `.exe` directly and see a WebView2 missing prompt, install the WebView2 Runtime or use the NSIS installer which bootstraps it automatically.

### Cross-building from Linux

Cross-building Windows installers from Linux is best-effort. Install `nsis` so the bundler can invoke `makensis`:

```bash
sudo apt update && sudo apt install -y nsis
./scripts/build-native.sh tauri-windows
```

If bundling fails, the script will still copy any produced `.exe` into `.build/tauri/windows/raw/`.

