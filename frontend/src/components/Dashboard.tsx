import { useEffect, useMemo, useState } from "react";
import type { CovidDatum } from "../services/mockApi";
import { fetchMockCovidSeries } from "../services/mockApi";
import { CasesLineChart, StackedAreaChart } from "./Charts";

type Place = string;

export default function Dashboard() {
	const [data, setData] = useState<CovidDatum[]>([]);
	const [place, setPlace] = useState<Place>("World");
	const [loading, setLoading] = useState(true);

	useEffect(() => {
		setLoading(true);
		fetchMockCovidSeries().then((rows) => {
			setData(rows);
			setLoading(false);
		});
	}, []);

	const places = useMemo(
		() => Array.from(new Set(data.map((d) => d.place))),
		[data],
	);

	const series = useMemo(() => {
		const filtered = data.filter((d) => d.place === place);
		return filtered.map((d) => ({ date: d.date, value: d.confirmed }));
	}, [data, place]);

	const stacked = useMemo(() => {
		// produce one item per date with aggregated fields
		const map = new Map<
			string,
			{ date: string; confirmed: number; deaths: number; recovered: number }
		>();
		data
			.filter((d) => d.place === place)
			.forEach((d) => {
				map.set(d.date, {
					date: d.date,
					confirmed: d.confirmed,
					deaths: d.deaths,
					recovered: d.recovered,
				});
			});
		return Array.from(map.values()).sort((a, b) =>
			a.date.localeCompare(b.date),
		);
	}, [data, place]);

	return (
		<div className="dashboard-root">
			<header className="dashboard-header">
				<h2>EpiCovid — Dashboard</h2>
				<div className="controls">
					<label>
						Place:
						<select value={place} onChange={(e) => setPlace(e.target.value)}>
							{places.map((p) => (
								<option key={p} value={p}>
									{p}
								</option>
							))}
						</select>
					</label>
				</div>
			</header>

			<main>
				{loading ? (
					<p>Loading mock data…</p>
				) : (
					<>
						<section className="cards">
							<div className="card small">
								<h3>Confirmed</h3>
								<p className="big">
									{stacked.length
										? stacked[stacked.length - 1].confirmed.toLocaleString()
										: "—"}
								</p>
							</div>
							<div className="card small">
								<h3>Recovered</h3>
								<p className="big">
									{stacked.length
										? stacked[stacked.length - 1].recovered.toLocaleString()
										: "—"}
								</p>
							</div>
							<div className="card small">
								<h3>Deaths</h3>
								<p className="big">
									{stacked.length
										? stacked[stacked.length - 1].deaths.toLocaleString()
										: "—"}
								</p>
							</div>
						</section>

						<section className="charts">
							<div className="chart-card">
								<h4>Confirmed cases (last 30 days)</h4>
								<CasesLineChart data={series} color="#8884d8" />
							</div>

							<div className="chart-card">
								<h4>Breakdown (Confirmed / Recovered / Deaths)</h4>
								<StackedAreaChart data={stacked} />
							</div>
						</section>
					</>
				)}
			</main>
		</div>
	);
}
