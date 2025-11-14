type Totals = { confirmed: number; deaths: number };

export default function KpiCards({ totals }: { totals: Totals }) {
	return (
		<section className="cards kpis" style={{ gridArea: "kpis" }}>
			<div className="card small confirmed">
				<h3>Confirmed</h3>
				<p className="big">{totals.confirmed.toLocaleString()}</p>
			</div>
			<div className="card small deaths">
				<h3>Deaths</h3>
				<p className="big">{totals.deaths.toLocaleString()}</p>
			</div>
		</section>
	);
}
