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
				// Candidate dark tile URLs (will try primary then fallbacks on error)
				const darkCandidates: Array<{ url: string; options: any }> = [
					// Standard OSM (will be dimmed by overlay if selected)
					{
						url: "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
						options: {
							attribution: "&copy; OpenStreetMap contributors",
							subdomains: ["a", "b", "c"],
							maxZoom: 19,
							noWrap: false,
						},
					},
					// HOT OSM variant
					{
						url: "https://{s}.tile.openstreetmap.fr/hot/{z}/{x}/{y}.png",
						options: {
							attribution:
								"&copy; OpenStreetMap contributors, Tiles style by HOT",
							subdomains: ["a", "b", "c"],
							maxZoom: 19,
							noWrap: false,
						},
					},
					{
						url: "https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png",
						options: {
							attribution: "&copy; OpenStreetMap contributors &copy; CARTO",
							subdomains: "abcd",
							maxZoom: 19,
							noWrap: true,
							continuousWorld: false,
						},
					},
					{
						url: "https://tiles.stadiamaps.com/tiles/alidade_dark/{z}/{x}/{y}{r}.png",
						options: {
							attribution:
								"&copy; OpenStreetMap contributors &copy; Stadia Maps &copy; OpenMapTiles",
							maxZoom: 20,
							noWrap: true,
						},
					},
					{
						url: "https://tiles.stadiamaps.com/tiles/alidade_smooth_dark/{z}/{x}/{y}{r}.png",
						options: {
							attribution:
								"&copy; OpenStreetMap contributors &copy; Stadia Maps &copy; OpenMapTiles",
							maxZoom: 20,
							noWrap: true,
						},
					},
				];
				let tileIndex = 0;
				function addNextTile() {
					if (tileIndex >= darkCandidates.length) {
						// Final fallback: use standard OSM tiles with a CSS darkening filter
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
							style.textContent =
								".world-map-canvas .leaflet-tile { filter: brightness(0.55) saturate(0.3) contrast(1.1); }";
							document.head.appendChild(style);
						}
						console.warn(
							"[worldmap] All dark providers failed. Using filtered OSM fallback.",
						);
						return;
					}
					const candidate = darkCandidates[tileIndex];
					const layer = L.tileLayer(candidate.url, candidate.options);
					layer.on("tileerror", () => {
						console.warn(
							`[worldmap] dark tiles failed for candidate ${tileIndex}. Trying next...`,
						);
						mapRef.current.removeLayer(layer);
						tileIndex++;
						addNextTile();
					});
					layer.on("load", () => {
						console.info(
							`[worldmap] dark tile candidate ${tileIndex} loaded successfully`,
						);
					});
					layer.addTo(mapRef.current);
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
