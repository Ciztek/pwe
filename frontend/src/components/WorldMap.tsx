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
	points: Array<{ lat: number; lon: number; value: number; place?: string }>;
	height?: number | string;
}) {
	const rootRef = useRef<HTMLDivElement | null>(null);
	const mapRef = useRef<any>(null);
	const layerGroupRef = useRef<any>(null);

	useEffect(() => {
		let mounted = true;
		(async () => {
			const L = await ensureLeaflet();
			if (!mounted || !rootRef.current) return;
			if (!mapRef.current) {
				mapRef.current = L.map(rootRef.current, {
					worldCopyJump: true,
					zoomControl: true,
				}).setView([20, 0], 2);
				// Cleaner light basemap (Carto Positron)
				L.tileLayer(
					"https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png",
					{
						attribution: "&copy; OpenStreetMap contributors &copy; CARTO",
						subdomains: "abcd",
						maxZoom: 19,
					},
				).addTo(mapRef.current);
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
				const norm = Math.sqrt((p.value || 0) / max);
				const radius = Math.max(minRadius, norm * maxRadius);
				const c = L.circleMarker([p.lat, p.lon], {
					radius,
					fillColor: "#ff6b6b",
					color: "#a60000",
					weight: 1,
					fillOpacity: 0.7,
				});
				const label = `${p.place || ""}<br/>${p.value.toLocaleString()} cases`;
				c.bindPopup(label, { closeButton: false, offset: [0, -2] });
				c.addTo(g);
				c.on("mouseover", () => c.openPopup());
				c.on("mouseout", () => c.closePopup());
			}
			g.addTo(mapRef.current);

			// Fit map view to markers gracefully
			if (points.length >= 2) {
				const bounds = L.latLngBounds(
					points.map((p) => [p.lat, p.lon] as [number, number]),
				);
				mapRef.current.fitBounds(bounds, { padding: [20, 20] });
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
			style={{ width: "100%", height, borderRadius: 8, overflow: "hidden" }}
		/>
	);
}
