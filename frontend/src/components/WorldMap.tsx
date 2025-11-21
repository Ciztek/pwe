/* eslint-disable @typescript-eslint/no-explicit-any */
import { useEffect, useRef } from "react";

// Lazy-load Leaflet via CDN so we don't require it as a build dependency.
async function ensureLeaflet() {
	if ((window as any).L) return (window as any).L;
	// Insert CSS
	const cssHref = "https://unpkg.com/leaflet@1.9.4/dist/leaflet.css";
	if (!document.querySelector(`link[href="${cssHref}"]`)) {
		const l = document.createElement("link");
		l.rel = "stylesheet";
		l.href = cssHref;
		document.head.appendChild(l);
	}
	// Insert script
	await new Promise<void>((resolve, reject) => {
		if ((window as any).L) return resolve();
		const s = document.createElement("script");
		s.src = "https://unpkg.com/leaflet@1.9.4/dist/leaflet.js";
		s.async = true;
		s.onload = () => resolve();
		s.onerror = () => reject(new Error("Failed to load Leaflet from CDN"));
		document.body.appendChild(s);
	});
	return (window as any).L;
}

export default function WorldMap({
	points,
	height = "100%",
}: {
	points: Array<{
		lat: number;
		lon: number;
		value: number;
		deaths?: number;
		place?: string;
	}>;
	height?: number | string;
}) {
	const rootRef = useRef<HTMLDivElement | null>(null);
	const mapRef = useRef<any>(null);
	const layerGroupRef = useRef<any>(null);

	useEffect(() => {
		let mounted = true;
		(async () => {
			let L: any;
			try {
				L = await ensureLeaflet();
			} catch (e) {
				console.warn("Leaflet failed to load:", e);
				return;
			}
			if (!mounted || !rootRef.current) return;
			if (!mapRef.current) {
				mapRef.current = L.map(rootRef.current, {
					worldCopyJump: false,
					zoomControl: true,
					minZoom: 2,
					maxZoom: 10,
					maxBounds: L.latLngBounds([
						[-85, -180],
						[85, 180],
					]),
					maxBoundsViscosity: 1.0,
				}).setView([20, 0], 2);
				// Ensure only dark tile layer present. Remove any pre-existing tile layers (e.g., light fallback from prior sessions)
				mapRef.current.eachLayer((layer: any) => {
					if (layer instanceof L.TileLayer) {
						mapRef.current.removeLayer(layer);
					}
				});
				// Candidate dark tile URLs with English/international labels prioritized.
				// Note: Wikimedia 'osm-intl' emphasizes international (English) names when available.
				// If a provider fails (404 / network), we cascade to next.
				const darkCandidates: Array<{ url: string; options: any }> = [
					{
						url: "https://maps.wikimedia.org/osm-intl/{z}/{x}/{y}.png",
						options: {
							attribution: "&copy; OpenStreetMap contributors | Wikimedia maps",
							maxZoom: 19,
							noWrap: true,
							detectRetina: false,
						},
					},
					{
						url: "https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png",
						options: {
							attribution: "&copy; OpenStreetMap contributors &copy; CARTO",
							subdomains: "abcd",
							maxZoom: 19,
							noWrap: true,
							continuousWorld: false,
							detectRetina: false,
						},
					},
					{
						url: "https://tiles.stadiamaps.com/tiles/alidade_dark/{z}/{x}/{y}.png",
						options: {
							attribution:
								"&copy; OpenStreetMap contributors &copy; Stadia Maps &copy; OpenMapTiles",
							maxZoom: 20,
							noWrap: true,
							detectRetina: false,
						},
					},
					{
						url: "https://tiles.stadiamaps.com/tiles/alidade_smooth_dark/{z}/{x}/{y}.png",
						options: {
							attribution:
								"&copy; OpenStreetMap contributors &copy; Stadia Maps &copy; OpenMapTiles",
							maxZoom: 20,
							noWrap: true,
							detectRetina: false,
						},
					},
				];
				let tileIndex = 0;
				let loadedTiles = 0;
				let fallbackTimeout: number | null = null;
				let errorTiles = 0;
				let earlyErrorCheck: number | null = null;
				function applyCssFallback() {
					const fallback = L.tileLayer(
						"https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
						{
							attribution: "&copy; OpenStreetMap contributors",
							maxZoom: 19,
							noWrap: true,
						},
					);
					fallback.addTo(mapRef.current);
					if (!document.getElementById("worldmap-dark-fallback-style")) {
						const style = document.createElement("style");
						style.id = "worldmap-dark-fallback-style";
						style.textContent = `
.world-map-canvas { position: relative; }
.world-map-canvas .leaflet-tile { filter: grayscale(0.25) brightness(0.72) contrast(1.08) saturate(0.6); }
.world-map-canvas::after { content:''; position:absolute; inset:0; background:rgba(13,17,23,0.55); pointer-events:none; }
`;
						document.head.appendChild(style);
					}
					console.warn(
						"[worldmap] Falling back to filtered OSM with overlay; dark tiles unavailable.",
					);
				}
				function addNextTile() {
					if (tileIndex >= darkCandidates.length) {
						// Final fallback after exhausting candidates
						applyCssFallback();
						return;
					}
					const candidate = darkCandidates[tileIndex];
					const layer = L.tileLayer(candidate.url, candidate.options);
					errorTiles = 0;
					loadedTiles = 0;
					layer.on("tileerror", () => {
						errorTiles++;
						// If too many errors accumulate quickly and nothing loaded yet, bail early.
						if (errorTiles >= 6 && loadedTiles === 0) {
							console.warn(
								`[worldmap] excessive tile 404s on candidate ${tileIndex}; switching early.`,
							);
							mapRef.current.removeLayer(layer);
							if (earlyErrorCheck) window.clearTimeout(earlyErrorCheck);
							tileIndex++;
							addNextTile();
						}
					});
					layer.on("load", () => {
						console.info(
							`[worldmap] dark tile candidate ${tileIndex} loaded successfully`,
						);
						// Remove any CSS fallback darkening filter if present
						const fallbackStyle = document.getElementById(
							"worldmap-dark-fallback-style",
						);
						if (fallbackStyle) fallbackStyle.remove();
						// Ensure stable tone class present
						if (
							rootRef.current &&
							!rootRef.current.classList.contains("dark-tone")
						) {
							rootRef.current.classList.add("dark-tone");
						}
					});
					layer.on("tileload", () => {
						loadedTiles++;
					});
					layer.addTo(mapRef.current);
					// Early error-rate check (1s). If no tiles loaded and errors piling up, switch.
					if (earlyErrorCheck) window.clearTimeout(earlyErrorCheck);
					earlyErrorCheck = window.setTimeout(() => {
						if (loadedTiles === 0 && errorTiles >= 4) {
							console.warn(
								`[worldmap] early error check triggered on candidate ${tileIndex}; moving to next.`,
							);
							mapRef.current.removeLayer(layer);
							tileIndex++;
							addNextTile();
						}
					}, 1000);
					// Start / reset timeout to verify tile visibility
					if (fallbackTimeout) window.clearTimeout(fallbackTimeout);
					fallbackTimeout = window.setTimeout(() => {
						if (loadedTiles === 0) {
							console.warn(
								"[worldmap] No tiles loaded in time; applying fallback overlay.",
							);
							// remove this failed candidate before fallback
							mapRef.current.removeLayer(layer);
							applyCssFallback();
						}
					}, 2500);
				}
				addNextTile();
				// Add a scale control
				L.control
					.scale({ metric: true, imperial: false })
					.addTo(mapRef.current);
			}

			// clear existing point layer group (keep base tile layer)
			if (layerGroupRef.current) {
				mapRef.current.removeLayer(layerGroupRef.current);
			}
			const g = L.layerGroup();
			layerGroupRef.current = g;

			// compute max to scale marker radii; use sqrt scaling for better perception
			const max = Math.max(1, ...points.map((p) => p.value || 0));
			const maxRadius = 42;
			const minRadius = 6;
			for (const p of points) {
				// Clamp coords to stay within map maxBounds and wrap longitude to [-180,180]
				const lat = Math.max(-85, Math.min(85, p.lat));
				let lon = p.lon;
				if (lon < -180 || lon > 180) {
					lon = ((((lon + 180) % 360) + 360) % 360) - 180;
				}
				const norm = Math.sqrt((p.value || 0) / max);
				const radius = Math.max(minRadius, norm * maxRadius);
				const c = L.circleMarker([lat, lon], {
					radius,
					fillColor: "#ff6b6b",
					color: "#a60000",
					weight: 1,
					fillOpacity: 0.7,
				});
				const short = (n: number) => {
					const abs = Math.abs(n);
					if (abs >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(2)}B`;
					if (abs >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
					if (abs >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
					return `${Math.round(n)}`;
				};
				const label =
					`${p.place || ""}<br/>Cases: ${p.value.toLocaleString()} (${short(p.value)})` +
					(typeof p.deaths === "number"
						? `<br/>Deaths: ${p.deaths.toLocaleString()} (${short(p.deaths)})`
						: "");
				c.bindPopup(label, { closeButton: false, offset: [0, -2] });
				c.addTo(g);
				c.on("mouseover", () => c.openPopup());
				c.on("mouseout", () => c.closePopup());
			}
			g.addTo(mapRef.current);

			// Ensure map sizes correctly after being mounted in responsive containers
			setTimeout(() => {
				try {
					if (
						mapRef.current &&
						typeof mapRef.current.invalidateSize === "function"
					) {
						mapRef.current.invalidateSize({ animate: false });
					}
				} catch {
					// ignore
				}
			}, 0);

			// Fit map view only on initial render or when first points appear
			const prevCount = (mapRef.current as any)._lastPointCount || 0;
			if (prevCount === 0) {
				if (points.length >= 2) {
					const bounds = L.latLngBounds(
						points.map((p) => [p.lat, p.lon] as [number, number]),
					);
					mapRef.current.fitBounds(bounds, { padding: [20, 20], maxZoom: 5 });
					if (mapRef.current.getZoom() < 2) mapRef.current.setZoom(2);
				} else if (points.length === 1) {
					mapRef.current.setView([points[0].lat, points[0].lon], 4);
				} else {
					mapRef.current.setView([20, 0], 2);
				}
			}
			(mapRef.current as any)._lastPointCount = points.length;
		})();
		return () => {
			mounted = false;
		};
	}, [points]);

	return (
		<div
			ref={rootRef}
			className="world-map-canvas"
			style={{ width: "100%", height, borderRadius: 8, overflow: "hidden" }}
		/>
	);
}
