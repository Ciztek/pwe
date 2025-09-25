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
