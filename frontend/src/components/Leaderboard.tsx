/* Leaderboard component: scrollable list of countries ranked by confirmed cases */

export type LeaderboardEntry = {
	place: string;
	confirmed: number;
	deaths: number;
};

export default function Leaderboard({
	entries,
	height = "100%",
}: {
	entries: LeaderboardEntry[];
	height?: number | string;
}) {
	const short = (n: number) => {
		const abs = Math.abs(n);
		if (abs >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(2)}B`;
		if (abs >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
		if (abs >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
		return `${Math.round(n)}`;
	};
	return (
		<div
			className="leaderboard-card"
			style={{ gridArea: "leaderboard", height }}
		>
			<header className="leaderboard-header">
				<h4>Leaderboard</h4>
				<small>Top countries (confirmed)</small>
			</header>
			<ol className="leaderboard-list">
				{entries.map((e, idx) => (
					<li key={e.place} className="leaderboard-row">
						<span className="rank">{idx + 1}</span>
						<span className="country" title={e.place}>
							{e.place}
						</span>
						<span
							className="confirmed"
							aria-label="confirmed cases"
							title={e.confirmed.toLocaleString()}
						>
							{short(e.confirmed)}
						</span>
						<span
							className="deaths"
							aria-label="deaths"
							title={e.deaths.toLocaleString()}
						>
							{short(e.deaths)}
						</span>
					</li>
				))}
			</ol>
		</div>
	);
}
