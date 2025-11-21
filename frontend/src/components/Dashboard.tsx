import { useEffect, useMemo, useState } from "react";
import ControlsBar from "./ControlsBar";
import KpiCards from "./KpiCards";
import ChartsGrid from "./ChartsGrid";
import {
	fetchSeries,
	fetchTotalsRange,
	fetchCountries,
	fetchCountryCoords,
	clearApiCaches,
	type SeriesPoint,
} from "../services/api";
import MapPanel from "./MapPanel";
import Leaderboard, { type LeaderboardEntry } from "./Leaderboard";
import { COUNTRY_COORDS as FALLBACK_COUNTRY_COORDS } from "../data/countryCoords";

// Using an expanded fallback set from data/countryCoords

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

	// Mobile view preference: 'kpi' | 'map' | 'leaderboard'
	const [mobileView, setMobileView] = useState<"kpi" | "map" | "leaderboard">(
		() => {
			if (typeof window !== "undefined") {
				const v = localStorage.getItem("mobileView") as
					| "kpi"
					| "map"
					| "leaderboard"
					| null;
				return v === "map" || v === "leaderboard" ? v : "kpi";
			}
			return "kpi";
		},
	);

	const [isMobile, setIsMobile] = useState<boolean>(() =>
		typeof window !== "undefined"
			? window.matchMedia && window.matchMedia("(max-width: 800px)").matches
			: false,
	);

	const [series, setSeries] = useState<SeriesPoint[]>([]);
	// Chart display preferences
	const [chartMode, setChartMode] = useState<"cumulative" | "daily">(
		"cumulative",
	);
	const [chartScale, setChartScale] = useState<"linear" | "log">("linear");
	const [refreshToken, setRefreshToken] = useState<number>(0);
	const [totals, setTotals] = useState({
		confirmed: 0,
		deaths: 0,
	});
	const [mapPoints, setMapPoints] = useState<
		Array<{
			lat: number;
			lon: number;
			value: number;
			deaths?: number;
			place?: string;
		}>
	>([]);
	const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
	const [coordsMap, setCoordsMap] = useState<
		Record<string, { lat: number; lon: number }>
	>({});
	const [loading, setLoading] = useState<boolean>(true);
	const [error, setError] = useState<string | null>(null);
	const [worldProgress, setWorldProgress] = useState<number>(0);
	const [worldProgressActive, setWorldProgressActive] =
		useState<boolean>(false);

	// Normalize name and attempt to find coordinates from backend map or fallback
	function getCoordsForCountry(
		name: string,
		provided: Record<string, { lat: number; lon: number }>,
	): { lat: number; lon: number } | undefined {
		const direct = provided[name];
		if (direct) return direct;
		const fallbackDirect = FALLBACK_COUNTRY_COORDS[name];
		if (fallbackDirect) return fallbackDirect;
		// Alias map for common naming differences
		const alias: Record<string, string> = {
			"United States": "USA",
			"United States of America": "USA",
			US: "USA",
			"U.S.": "USA",
			"United Kingdom": "United Kingdom",
			UK: "United Kingdom",
			"U.A.E.": "United Arab Emirates",
			UAE: "United Arab Emirates",
			"Czech Republic": "Czechia",
			"Russian Federation": "Russia",
			"South Korea": "South Korea",
			"Korea, South": "South Korea",
			"Korea (South)": "South Korea",
			"South Africa": "South Africa",
			"Saudi Arabia": "Saudi Arabia",
			"New Zealand": "New Zealand",
			"Dominican Republic": "Dominican Republic",
			"Costa Rica": "Costa Rica",
		};
		const aliased = alias[name];
		if (aliased) {
			return (
				provided[aliased] ||
				FALLBACK_COUNTRY_COORDS[aliased] ||
				FALLBACK_COUNTRY_COORDS[name]
			);
		}
		// Try normalized key matching (remove spaces/punct, lowercase)
		const norm = (s: string) => s.replace(/[^a-z0-9]/gi, "").toLowerCase();
		const n = norm(name);
		for (const k of Object.keys(provided)) {
			if (norm(k) === n) return provided[k];
		}
		for (const k of Object.keys(FALLBACK_COUNTRY_COORDS)) {
			if (norm(k) === n) return FALLBACK_COUNTRY_COORDS[k];
		}
		return undefined;
	}

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

	// Fetch static lookup data (countries + coords) once or on refresh
	useEffect(() => {
		let mounted = true;
		console.log("[dashboard] init/refresh: fetching countries & coords");
		Promise.all([fetchCountries(), fetchCountryCoords()])
			.then(([countries, coords]) => {
				if (!mounted) return;
				setPlaces(Array.from(new Set(["World", ...(countries || [])])));
				setCoordsMap(coords || {});
				console.log(
					`[dashboard] loaded countries=${countries.length} coords=${Object.keys(coords || {}).length}`,
				);
			})
			.catch((err) =>
				console.warn("[dashboard] failed to load countries/coords", err),
			);
		return () => {
			mounted = false;
		};
	}, [refreshToken]);

	// Fetch time-series + totals + map points when parameters change, coordsMap ready, or refresh
	useEffect(() => {
		let mounted = true;
		setLoading(true);
		setError(null);
		console.log(
			`[dashboard] data fetch start place=${place} start=${start} end=${end} coordsReady=${Object.keys(coordsMap).length} view=${mobileView} refreshToken=${refreshToken}`,
		);
		Promise.all([
			fetchSeries(start, end, place === "World" ? undefined : place),
			fetchTotalsRange(start, end, place === "World" ? undefined : place),
		])
			.then(([s, t]) => {
				if (!mounted) return;
				setSeries(s);
				setTotals({
					confirmed: t.confirmed,
					deaths: t.deaths,
				});
				console.log(
					`[dashboard] series points=${s.length} totals confirmed=${t.confirmed}`,
				);
				// Always build world dataset for leaderboard regardless of current place
				{
					setWorldProgressActive(true);
					setWorldProgress(0);
					let countries = places.filter((c) => c !== "World");
					// Fallback if places not ready: use coords or fallback dataset keys
					if (countries.length === 0) {
						const fromCoords = Object.keys(coordsMap || {});
						const fromFallback = Object.keys(FALLBACK_COUNTRY_COORDS || {});
						countries = (fromCoords.length ? fromCoords : fromFallback).slice(
							0,
							200,
						);
					}
					// Process in small concurrent batches and update incrementally
					const concurrency = 8;
					let idx = 0;
					const acc: Array<{
						place: string;
						confirmed: number;
						deaths: number;
						coords?: { lat: number; lon: number } | undefined;
					}> = [];
					async function worker() {
						while (idx < countries.length) {
							const i = idx++;
							const c = countries[i];
							try {
								const totals = await fetchTotalsRange(start, end, c);
								const coords = getCoordsForCountry(c, coordsMap);
								acc.push({
									place: c,
									confirmed: totals.confirmed,
									deaths: totals.deaths,
									coords,
								});
								// update progress
								if (mounted) {
									const pct = Math.max(
										0,
										Math.min(
											100,
											Math.round((acc.length / countries.length) * 100),
										),
									);
									setWorldProgress(pct);
								}
								if (acc.length % 20 === 0 && mounted) {
									const entries: LeaderboardEntry[] = acc
										.slice()
										.sort((a, b) => b.confirmed - a.confirmed)
										.map((i) => ({
											place: i.place,
											confirmed: i.confirmed,
											deaths: i.deaths,
										}));
									setLeaderboard(entries);
									// incremental map point updates so map isn't empty
									if (place === "World") {
										const pts = acc
											.filter((i) => !!i.coords)
											.map((i) => ({
												lat: i.coords!.lat,
												lon: i.coords!.lon,
												value: i.confirmed,
												deaths: i.deaths,
												place: i.place,
											}));
										setMapPoints(pts);
									}
								}
							} catch {
								// ignore
							}
						}
					}
					Promise.all(
						Array.from(
							{ length: Math.min(concurrency, countries.length) },
							() => worker(),
						),
					).then(() => {
						if (!mounted) return;
						const entries: LeaderboardEntry[] = acc
							.slice()
							.sort((a, b) => b.confirmed - a.confirmed)
							.map((i) => ({
								place: i.place,
								confirmed: i.confirmed,
								deaths: i.deaths,
							}));
						setLeaderboard(entries);
						if (place === "World") {
							const pts = acc
								.filter((i) => !!i.coords)
								.map((i) => ({
									lat: i.coords!.lat,
									lon: i.coords!.lon,
									value: i.confirmed,
									deaths: i.deaths,
									place: i.place,
								}));
							setMapPoints(pts);
							console.log(
								`[dashboard] world dataset built countries=${acc.length}, map points=${pts.length}`,
							);
						}
						setWorldProgress(100);
						setTimeout(() => setWorldProgressActive(false), 400);
					});
				}
				// Single-place map point (non-world scope)
				if (place !== "World") {
					const coords = coordsMap[place] || FALLBACK_COUNTRY_COORDS[place];
					if (coords) {
						setMapPoints([
							{
								lat: coords.lat,
								lon: coords.lon,
								value: t.confirmed,
								deaths: t.deaths,
								place,
							},
						]);
						console.log("[dashboard] single map point rendered");
					} else {
						setMapPoints([]);
					}
				}
			})
			.catch((err) => {
				setError(err instanceof Error ? err.message : String(err));
				console.warn("[dashboard] data fetch error", err);
			})
			.finally(() => {
				if (mounted) setLoading(false);
				console.log("[dashboard] data fetch end");
			});
		return () => {
			mounted = false;
		};
	}, [place, start, end, coordsMap, places, mobileView, refreshToken]);

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
				chartMode={chartMode}
				onChartModeChange={setChartMode}
				chartScale={chartScale}
				onChartScaleChange={setChartScale}
				onRefresh={() => {
					clearApiCaches();
					setRefreshToken((r) => r + 1);
				}}
				mobileOrder={mobileOrder}
				onMobileOrderChange={setMobileOrder}
				mobileView={mobileView}
				onMobileViewChange={setMobileView}
				onPlaceChange={setPlace}
				onStartChange={setStart}
				onEndChange={setEnd}
			/>

			{worldProgressActive && (
				<div
					className="progress-container"
					aria-label="Loading data"
					aria-valuemin={0}
					aria-valuemax={100}
					aria-valuenow={worldProgress}
					role="progressbar"
				>
					<div
						className="progress-fill"
						style={{ width: `${worldProgress}%` }}
					/>
				</div>
			)}

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
					) : mobileView === "leaderboard" ? (
						<Leaderboard entries={leaderboard.slice(0, 100)} />
					) : (
						<>
							<KpiCards totals={totals} />
							<ChartsGrid
								line={lineSeries}
								stacked={stacked}
								chartMode={chartMode}
								chartScale={chartScale}
							/>
						</>
					)
				) : (
					<>
						<KpiCards totals={totals} />
						<ChartsGrid
							line={lineSeries}
							stacked={stacked}
							chartMode={chartMode}
							chartScale={chartScale}
						/>
						<MapPanel points={mapPoints} />
						<Leaderboard entries={leaderboard.slice(0, 100)} />
					</>
				)}
			</main>
		</div>
	);
}
