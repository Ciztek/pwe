import { isCapacitor } from "../hooks/usePlatform";
export type DataOutput = {
	place?: string | null;
	date?: string | null;
	date_range?: string | null;
	confirmed: number;
	deaths: number;
	recovered: number;
};

export type SeriesPoint = {
	date: string;
	confirmed: number;
	deaths: number;
	recovered: number;
};

export type PlaceOutput = {
	countries: string[];
	state: string[];
	us_counties: string[];
};

function getBaseUrl() {
	// Allow runtime override without rebuild (useful on mobile): localStorage.apiBaseUrl
	try {
		const fromStorage = localStorage.getItem("apiBaseUrl");
		if (fromStorage) return fromStorage.replace(/\/$/, "");
	} catch {
		// ignore storage errors
	}
	const env = (
		import.meta as unknown as { env?: { VITE_API_BASE_URL?: string } }
	).env;
	const fromEnv = env?.VITE_API_BASE_URL?.replace(/\/$/, "");
	return fromEnv || "http://127.0.0.1:8000/api";
}

async function apiFetch<T>(path: string): Promise<T> {
	const url = `${getBaseUrl()}${path}`;
	if (isCapacitor()) {
		// Use native HTTP to bypass CORS and handle self-signed/cleartext cases better.
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const w = window as unknown as any;
		const Http = w?.Capacitor?.Plugins?.Http || w?.Capacitor?.Http;
		if (Http && typeof Http.get === "function") {
			const res = await Http.get({ url });
			if (res.status < 200 || res.status >= 300) {
				const text =
					typeof res.data === "string"
						? res.data
						: JSON.stringify(res.data ?? {});
				throw new Error(`API ${res.status}: ${text}`);
			}
			return res.data as T;
		}
		// Fallback if Http plugin unavailable
		// CapacitorHttp returns { data, status, headers }
		// Note: fetch may still hit CORS on device if backend doesn't allow capacitor://localhost
	}
	const res = await fetch(url, { credentials: "include" });
	if (!res.ok) {
		const text = await res.text().catch(() => "");
		throw new Error(`API ${res.status}: ${text || res.statusText}`);
	}
	return res.json() as Promise<T>;
}

// Simple memo cache for daily totals to avoid refetching the same date/country
const dailyCache = new Map<string, Promise<DataOutput | null>>();

export async function fetchTotalsForDate(
	isoDate: string,
	country?: string,
): Promise<DataOutput | null> {
	const path = country
		? `/data/${isoDate}/${encodeURIComponent(country)}`
		: `/data/${isoDate}`;
	const key = `${isoDate}|${country ?? "_all"}`;
	if (!dailyCache.has(key)) {
		dailyCache.set(
			key,
			(async () => {
				try {
					return await apiFetch<DataOutput>(path);
				} catch (err) {
					const msg = err instanceof Error ? err.message : String(err || "");
					if (msg.startsWith("API 404")) return null;
					throw err;
				}
			})(),
		);
	}
	try {
		return await dailyCache.get(key)!;
	} catch (err) {
		const msg = err instanceof Error ? err.message : String(err || "");
		if (msg.startsWith("API 404")) return null;
		throw err;
	}
}

export async function fetchSeries(
	start: string,
	end: string,
	country?: string,
): Promise<SeriesPoint[]> {
	const s = new Date(start);
	const e = new Date(end);
	const totalDays = Math.max(
		1,
		Math.ceil((e.getTime() - s.getTime()) / (24 * 3600 * 1000)) + 1,
	);

	// For short ranges, fetch daily new cases to draw a proper line.
	if (totalDays <= 31) {
		const dates: string[] = [];
		for (let i = 0; i < totalDays; i++) {
			const d = new Date(s);
			d.setDate(s.getDate() + i);
			dates.push(d.toISOString().slice(0, 10));
		}
		const points = await Promise.all(
			dates.map(async (iso) => {
				const d = await fetchTotalsForDate(iso, country);
				if (!d) return null;
				return {
					date: iso,
					confirmed: d.confirmed,
					deaths: d.deaths,
					recovered: d.recovered,
				} as SeriesPoint;
			}),
		);
		return points.filter(Boolean) as SeriesPoint[];
	}

	// For longer ranges, build an evenly bucketed series using the range endpoint.
	// This avoids per-day calls while producing a proper multi-point line.
	// We compute disjoint buckets that exactly cover [start, end] with no overlap.
	const targetPoints = Math.min(60, Math.max(20, Math.floor(totalDays / 7))); // ~weekly up to 60 points

	const buckets: Array<{ a: string; b: string }> = [];
	for (let i = 0; i < targetPoints; i++) {
		const aIndex = Math.floor((i * totalDays) / targetPoints);
		const bIndex = Math.floor(((i + 1) * totalDays) / targetPoints) - 1;
		const aDate = new Date(s);
		aDate.setDate(s.getDate() + aIndex);
		const bDate = new Date(s);
		bDate.setDate(s.getDate() + Math.max(aIndex, bIndex));
		if (bDate > e) bDate.setTime(e.getTime());
		const aIso = aDate.toISOString().slice(0, 10);
		const bIso = bDate.toISOString().slice(0, 10);
		// Ensure ascending and within [start, end]
		if (aIso <= bIso) buckets.push({ a: aIso, b: bIso });
	}

	const points = await Promise.all(
		buckets.map(async (c) => {
			const t = await fetchTotalsRange(c.a, c.b, country);
			const p: SeriesPoint = {
				date: c.b,
				confirmed: t.confirmed,
				deaths: t.deaths,
				recovered: t.recovered,
			};
			return p;
		}),
	);
	return points;
}

export async function fetchTotalsRange(
	start: string,
	end: string,
	country?: string,
): Promise<DataOutput> {
	const path = country
		? `/data/${start}/${end}/${encodeURIComponent(country)}`
		: `/data/${start}/${end}`;
	return apiFetch<DataOutput>(path);
}

export async function fetchPlaces(): Promise<PlaceOutput> {
	return apiFetch<PlaceOutput>("/places");
}

export async function fetchCountries(): Promise<string[]> {
	const po = await fetchPlaces();
	return Array.isArray(po?.countries) ? po.countries : [];
}
