// Simple mock API that returns time series data for Covid cases, deaths and recovered
export type CovidDatum = {
	date: string; // ISO date
	place: string;
	confirmed: number;
	deaths: number;
	recovered: number;
};

// Generate synthetic data for a few locations over a timeframe
export function fetchMockCovidSeries(): Promise<CovidDatum[]> {
	const places = ["World", "France", "USA", "Brazil", "India"];
	const start = new Date();
	start.setDate(start.getDate() - 29); // 30 days

	const rows: CovidDatum[] = [];
	for (let i = 0; i < 30; i++) {
		const d = new Date(start);
		d.setDate(start.getDate() + i);
		const iso = d.toISOString().slice(0, 10);
		places.forEach((p, idx) => {
			// base values vary by place
			const base = 1000 * (idx + 1);
			const growth = Math.round(
				base * (1 + i * 0.03) + Math.sin(i / 3 + idx) * 200,
			);
			const deaths = Math.max(
				0,
				Math.round(growth * (0.01 + idx * 0.002 + Math.random() * 0.01)),
			);
			const recovered = Math.max(
				0,
				Math.round(growth * (0.6 - idx * 0.03 + Math.random() * 0.15)),
			);
			rows.push({ date: iso, place: p, confirmed: growth, deaths, recovered });
		});
	}

	// simulate network latency
	return new Promise((resolve) => setTimeout(() => resolve(rows), 350));
}
