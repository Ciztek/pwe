import { CasesLineChart, StackedAreaChart, DailyNewCasesChart } from "./Charts";

export type LinePoint = { date: string; value: number };
export type StackedPoint = {
	date: string;
	confirmed: number;
	deaths: number;
};

export default function ChartsGrid({
	line,
	stacked,
	chartMode = "cumulative",
	chartScale = "linear",
}: {
	line: LinePoint[];
	stacked: StackedPoint[];
	chartMode?: "cumulative" | "daily";
	chartScale?: "linear" | "log";
}) {
	return (
		<>
			{chartMode === "daily" ? (
				<div className="chart-card fill" style={{ gridArea: "line" }}>
					<DailyNewCasesChart data={stacked} height="100%" scale={chartScale} />
				</div>
			) : (
				<div className="chart-card fill" style={{ gridArea: "line" }}>
					<CasesLineChart
						data={line}
						color="#2962ff"
						height="100%"
						scale={chartScale}
					/>
				</div>
			)}
			<div className="chart-card fill" style={{ gridArea: "stacked" }}>
				<StackedAreaChart data={stacked} height="100%" scale={chartScale} />
			</div>
		</>
	);
}
