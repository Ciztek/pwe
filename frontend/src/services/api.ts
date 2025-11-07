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

// New backend places response: { continents: [ { id, name, countries: [ { id, name, states: [ { id, name, counties?: [ { id, name } ] } ] } ] } ] }
export type PlacesResponse = {
	continents: Array<{
		id: number;
		name: string;
		countries: Array<{
			id: number;
			name: string;
			states: Array<{
				id: number;
				name: string;
				counties?: Array<{ id: number; name: string }>;
			}>;
		}>;
	}>;
};

function getBaseUrl() {
	const env = (
		import.meta as unknown as { env?: { VITE_API_BASE_URL?: string } }
	).env;
	const fromEnv = env?.VITE_API_BASE_URL?.replace(/\/$/, "");
	if (fromEnv) return fromEnv;
	// Detect Capacitor Android and use emulator host alias if no env provided
	try {
		const w = window as unknown as {
			Capacitor?: { getPlatform?: () => string; platform?: string };
		};
		const cap = w?.Capacitor;
		const platform: string | undefined = cap?.getPlatform?.() || cap?.platform;
		if (platform === "android") {
			// For Android emulator: host machine is 10.0.2.2
			return "http://10.0.2.2:8000";
		}
	} catch {
		/* ignore */
	}
	// Fallback for web/desktop
	return "http://127.0.0.1:8000";
}

async function apiFetch<T>(path: string): Promise<T> {
	const base = getBaseUrl();
	const url = `${base}${path}`;
	try {
		const res = await fetch(url);
		if (!res.ok) {
			const text = await res.text().catch(() => "");
			// Helpful for mobile 404 diagnostics
			console.warn(`[api] ${res.status} for`, url, text || res.statusText);
			throw new Error(`API ${res.status}: ${text || res.statusText}`);
		}
		return res.json() as Promise<T>;
	} catch (e) {
		console.warn(`[api] fetch failed for`, url, e);
		throw e;
	}
}

// Simple memo cache for daily totals to avoid refetching the same date/country
const dailyCache = new Map<string, Promise<DataOutput | null>>();

export async function fetchTotalsForDate(
	isoDate: string,
	country?: string,
): Promise<DataOutput | null> {
	const params = new URLSearchParams();
	params.set("date", isoDate);
	if (country && country !== "World") params.set("country", country);
	const path = `/filter/data?${params.toString()}`;
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
	const params = new URLSearchParams();
	params.set("start_date", start);
	params.set("end_date", end);
	if (country && country !== "World") params.set("country", country);
	return apiFetch<DataOutput>(`/filter/data?${params.toString()}`);
}

export async function fetchPlaces(): Promise<PlacesResponse> {
	return apiFetch<PlacesResponse>("/filter/places");
}

export async function fetchCountries(): Promise<string[]> {
	const po = await fetchPlaces();
	const names = new Set<string>();
	for (const cont of po?.continents ?? []) {
		for (const c of cont.countries ?? []) {
			if (c?.name) names.add(c.name);
		}
	}
	return Array.from(names).sort((a, b) => a.localeCompare(b));
}
