import { CasesLineChart, StackedAreaChart } from "./Charts";

export type LinePoint = { date: string; value: number };
export type StackedPoint = {
	date: string;
	confirmed: number;
	deaths: number;
	recovered: number;
};

export default function ChartsGrid({
	line,
	stacked,
}: {
	line: LinePoint[];
	stacked: StackedPoint[];
}) {
	return (
		<>
			<div className="chart-card fill" style={{ gridArea: "line" }}>
				<CasesLineChart data={line} color="#2962ff" height="100%" />
			</div>
			<div className="chart-card fill" style={{ gridArea: "stacked" }}>
				<StackedAreaChart data={stacked} height="100%" />
			</div>
		</>
	);
}
