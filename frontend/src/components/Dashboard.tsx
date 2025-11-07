import { useEffect, useMemo, useState } from "react";
import ControlsBar from "./ControlsBar";
import KpiCards from "./KpiCards";
import ChartsGrid from "./ChartsGrid";
import {
	fetchSeries,
	fetchTotalsRange,
	fetchCountries,
	type SeriesPoint,
} from "../services/api";
import MapPanel from "./MapPanel";

// Minimal static coords for common countries; in a real app we'd fetch geo data
const COUNTRY_COORDS: Record<string, { lat: number; lon: number }> = {
	France: { lat: 46.2276, lon: 2.2137 },
	USA: { lat: 37.0902, lon: -95.7129 },
	Brazil: { lat: -14.235, lon: -51.9253 },
	India: { lat: 20.5937, lon: 78.9629 },
};

export default function Dashboard() {
	const [place, setPlace] = useState<string>("World");
	const [places, setPlaces] = useState<string[]>(["World"]);
	const [start, setStart] = useState<string>("2021-01-01");
	const [end, setEnd] = useState<string>("2021-01-30");

	// Mobile layout preference: 'charts' | 'map'
	const [mobileOrder, setMobileOrder] = useState<"charts" | "map">(() => {
		if (typeof window !== "undefined") {
			return (
				(localStorage.getItem("mobileOrder") as "charts" | "map") || "charts"
			);
		}
		return "charts";
	});

	// Mobile view preference: 'kpi' | 'map'
	const [mobileView, setMobileView] = useState<"kpi" | "map">(() => {
		if (typeof window !== "undefined") {
			return (localStorage.getItem("mobileView") as "kpi" | "map") || "kpi";
		}
		return "kpi";
	});

	const [isMobile, setIsMobile] = useState<boolean>(() =>
		typeof window !== "undefined"
			? window.matchMedia && window.matchMedia("(max-width: 800px)").matches
			: false,
	);

	const [series, setSeries] = useState<SeriesPoint[]>([]);
	const [totals, setTotals] = useState({
		confirmed: 0,
		recovered: 0,
		deaths: 0,
	});
	const [mapPoints, setMapPoints] = useState<
		Array<{ lat: number; lon: number; value: number; place?: string }>
	>([]);
	const [loading, setLoading] = useState<boolean>(true);
	const [error, setError] = useState<string | null>(null);

	// Persist mobile order preference
	useEffect(() => {
		try {
			localStorage.setItem("mobileOrder", mobileOrder);
		} catch {
			// ignore storage issues
		}
	}, [mobileOrder]);

	// Persist mobile view preference
	useEffect(() => {
		try {
			localStorage.setItem("mobileView", mobileView);
		} catch {
			// ignore
		}
	}, [mobileView]);

	// Track viewport to decide whether to render single-page mobile views
	useEffect(() => {
		if (typeof window === "undefined" || !window.matchMedia) return;
		const mq = window.matchMedia("(max-width: 800px)");
		const handler = (e: MediaQueryListEvent | MediaQueryList) => {
			setIsMobile("matches" in e ? e.matches : (e as MediaQueryList).matches);
		};
		// Initial sync and subscribe
		handler(mq as unknown as MediaQueryList);
		mq.addEventListener?.(
			"change",
			handler as (e: MediaQueryListEvent) => void,
		);
		return () => {
			mq.removeEventListener?.(
				"change",
				handler as (e: MediaQueryListEvent) => void,
			);
		};
	}, []);

	useEffect(() => {
		let mounted = true;
		setLoading(true);
		setError(null);
		Promise.all([
			fetchSeries(start, end, place === "World" ? undefined : place),
			fetchTotalsRange(start, end, place === "World" ? undefined : place),
			fetchCountries(),
		])
			.then(([s, t, countries]) => {
				if (!mounted) return;
				setSeries(s);
				setTotals({
					confirmed: t.confirmed,
					recovered: t.recovered,
					deaths: t.deaths,
				});
				setPlaces(Array.from(new Set(["World", ...(countries || [])])));

				// Build map points: if World selected, show each place from fetched places
				if (place === "World") {
					const tasks = (countries || []).map(async (c) => {
						try {
							// Use the range endpoint to get totals for the timespan
							const totals = await fetchTotalsRange(start, end, c);
							const coords = COUNTRY_COORDS[c];
							if (coords)
								return {
									lat: coords.lat,
									lon: coords.lon,
									value: totals.confirmed,
									place: c,
								};
						} catch {
							// ignore per-country failures
						}
						return null;
					});
					Promise.all(tasks).then((res) => {
						if (!mounted) return;
						const pts = res.filter(Boolean) as Array<{
							lat: number;
							lon: number;
							value: number;
							place?: string;
						}>;
						setMapPoints(pts || []);
					});
				} else {
					// single-country: show single point if we have coords
					const coords = COUNTRY_COORDS[place];
					if (coords) {
						setMapPoints([
							{ lat: coords.lat, lon: coords.lon, value: t.confirmed, place },
						]);
					} else {
						setMapPoints([]);
					}
				}
			})
			.catch((err) => {
				setError(err instanceof Error ? err.message : String(err));
			})
			.finally(() => {
				if (mounted) setLoading(false);
			});
		return () => {
			mounted = false;
		};
	}, [place, start, end]);

	const lineSeries = useMemo(
		() => series.map((d) => ({ date: d.date, value: d.confirmed })),
		[series],
	);
	const stacked = useMemo(
		() =>
			series.map((d) => ({
				date: d.date,
				confirmed: d.confirmed,
				deaths: d.deaths,
				recovered: d.recovered,
			})),
		[series],
	);

	return (
		<div className="dashboard-root">
			<ControlsBar
				place={place}
				places={places}
				start={start}
				end={end}
				mobileOrder={mobileOrder}
				onMobileOrderChange={setMobileOrder}
				mobileView={mobileView}
				onMobileViewChange={setMobileView}
				onPlaceChange={setPlace}
				onStartChange={setStart}
				onEndChange={setEnd}
			/>

			<main
				className={`dashboard-main-grid ${mobileOrder === "map" ? "mobile-map-first" : "mobile-charts-first"}`}
			>
				{loading ? (
					<p>Loading data…</p>
				) : error ? (
					<p style={{ color: "#ff6b6b" }}>Error: {error}</p>
				) : isMobile ? (
					mobileView === "map" ? (
						<MapPanel points={mapPoints} />
					) : (
						<>
							<KpiCards totals={totals} />
							<ChartsGrid line={lineSeries} stacked={stacked} />
						</>
					)
				) : (
					<>
						<KpiCards totals={totals} />
						<ChartsGrid line={lineSeries} stacked={stacked} />
						<MapPanel points={mapPoints} />
					</>
				)}
			</main>
		</div>
	);
}
