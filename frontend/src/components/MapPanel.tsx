import WorldMap from "./WorldMap";

export default function MapPanel({
	points,
}: {
	points: Array<{ lat: number; lon: number; value: number; place?: string }>;
}) {
	return (
		<div className="map-card fill" style={{ gridArea: "map" }}>
			<WorldMap points={points} height="100%" />
		</div>
	);
}
