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
				// Primary dark tile layer
				const primaryTiles = L.tileLayer(
					"https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png",
					{
						attribution: "&copy; OpenStreetMap contributors &copy; CARTO",
						subdomains: "abcd",
						maxZoom: 19,
						noWrap: true,
						continuousWorld: false,
					},
				);
				primaryTiles.addTo(mapRef.current);
				// Fallback tile layer (standard OSM) loaded only if primary errors repeatedly
				let fallbackAdded = false;
				let errorCount = 0;
				primaryTiles.on("tileerror", () => {
					errorCount++;
					if (errorCount >= 4 && !fallbackAdded) {
						fallbackAdded = true;
						L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
							attribution: "&copy; OpenStreetMap contributors",
							maxZoom: 19,
						}).addTo(mapRef.current);
						console.warn(
							"[worldmap] primary tiles failing; fallback OSM layer added",
						);
					}
				});
				// Add a scale control
				L.control
					.scale({ metric: true, imperial: false })
					.addTo(mapRef.current);
			}

			// clear existing layer group
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

			// Fit map view to markers gracefully
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
