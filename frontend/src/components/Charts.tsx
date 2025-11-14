import { format, parseISO } from "date-fns";
import {
	Area,
	AreaChart,
	CartesianGrid,
	Line,
	LineChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
	Legend,
} from "recharts";

type SeriesPoint = {
	date: string;
	value: number;
};

export function CasesLineChart({
	data,
	color = "#8884d8",
	height = "100%",
	scale = "linear",
}: {
	data: SeriesPoint[];
	color?: string;
	height?: number | string;
	scale?: "linear" | "log";
}) {
	const short = (n: number) => {
		const abs = Math.abs(n);
		if (abs >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(2)}B`;
		if (abs >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
		if (abs >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
		return `${Math.round(n)}`;
	};

	return (
		<ResponsiveContainer width="100%" height={height}>
			<LineChart data={data} margin={{ top: 6, right: 12, left: 4, bottom: 0 }}>
				<CartesianGrid strokeDasharray="3 3" />
				<XAxis
					dataKey="date"
					tickFormatter={(d: string | number) =>
						format(parseISO(String(d)), "MM/dd")
					}
					minTickGap={24}
					tickCount={Math.min(6, Math.max(2, data.length))}
					tick={{ fill: "#e2e6ec", fontSize: 11 }}
				/>
				<YAxis
					width={56}
					domain={scale === "log" ? [1, "dataMax"] : [0, "dataMax"]}
					allowDecimals={false}
					tickFormatter={short}
					tick={{ fill: "#e2e6ec", fontSize: 11 }}
					scale={scale}
				/>
				<Tooltip
					labelFormatter={(d: string) => format(parseISO(String(d)), "PPP")}
					formatter={(v: number) =>
						[short(Number(v)), "Confirmed"] as [string, string]
					}
					contentStyle={{
						background: "#1e2530",
						border: "1px solid #3a4450",
						borderRadius: 6,
						color: "#e2e6ec",
						boxShadow: "0 4px 12px rgba(0,0,0,0.6)",
						padding: "6px 8px",
					}}
					itemStyle={{ color: "#e2e6ec", padding: 0 }}
					cursor={{ stroke: "#3a4450", strokeWidth: 1 }}
				/>
				<Line
					type="monotone"
					dataKey="value"
					stroke={color}
					dot={data.length <= 2}
					strokeWidth={2}
				/>
			</LineChart>
		</ResponsiveContainer>
	);
}

export function StackedAreaChart({
	data,
	height = "100%",
	scale = "linear",
}: {
	data: Array<{
		date: string;
		confirmed: number;
		deaths: number;
	}>;
	height?: number | string;
	scale?: "linear" | "log";
}) {
	const short = (n: number) => {
		const abs = Math.abs(n);
		if (abs >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(2)}B`;
		if (abs >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
		if (abs >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
		return `${Math.round(n)}`;
	};
	return (
		<ResponsiveContainer width="100%" height={height}>
			<AreaChart data={data} margin={{ top: 6, right: 12, left: 4, bottom: 0 }}>
				<CartesianGrid strokeDasharray="3 3" />
				<XAxis
					dataKey="date"
					tickFormatter={(d: string | number) =>
						format(parseISO(String(d)), "MM/dd")
					}
					minTickGap={24}
					tickCount={Math.min(6, Math.max(2, data.length))}
					tick={{ fill: "#e2e6ec", fontSize: 11 }}
				/>
				<YAxis
					width={56}
					domain={scale === "log" ? [1, "dataMax"] : [0, "dataMax"]}
					allowDecimals={false}
					tickFormatter={short}
					tick={{ fill: "#e2e6ec", fontSize: 11 }}
					scale={scale}
				/>
				<Tooltip
					labelFormatter={(d: string) => format(parseISO(String(d)), "PPP")}
					formatter={(v: number, name: string) =>
						[
							short(Number(v)),
							name.charAt(0).toUpperCase() + name.slice(1),
						] as [string, string]
					}
					contentStyle={{
						background: "#1e2530",
						border: "1px solid #3a4450",
						borderRadius: 6,
						color: "#e2e6ec",
						boxShadow: "0 4px 12px rgba(0,0,0,0.6)",
						padding: "6px 8px",
					}}
					itemStyle={{ color: "#e2e6ec", padding: 0 }}
					cursor={{ stroke: "#3a4450", strokeWidth: 1 }}
				/>
				<Legend
					verticalAlign="top"
					align="right"
					wrapperStyle={{ fontSize: "0.7rem", paddingBottom: 4 }}
				/>
				<Area
					type="monotone"
					dataKey="confirmed"
					stackId="1"
					stroke="#2962ff"
					fill="#2962ff"
					fillOpacity={0.35}
				/>
				<Area
					type="monotone"
					dataKey="deaths"
					stackId="1"
					stroke="#ff6b6b"
					fill="#ff6b6b"
					fillOpacity={0.4}
				/>
			</AreaChart>
		</ResponsiveContainer>
	);
}

// Daily new cases & deaths with rolling 7-day average overlay for confirmed
export function DailyNewCasesChart({
	data,
	height = "100%",
	scale = "linear",
}: {
	data: Array<{ date: string; confirmed: number; deaths: number }>;
	height?: number | string;
	scale?: "linear" | "log";
}) {
	// Transform cumulative series to daily increments
	const daily = data.map((d, i) => {
		const prev = i > 0 ? data[i - 1] : undefined;
		return {
			date: d.date,
			confirmed: prev ? Math.max(0, d.confirmed - prev.confirmed) : d.confirmed,
			deaths: prev ? Math.max(0, d.deaths - prev.deaths) : d.deaths,
		};
	});
	// Rolling 7-day average of confirmed daily increments
	const avg7 = daily.map((_, i) => {
		const slice = daily.slice(Math.max(0, i - 6), i + 1);
		const sum = slice.reduce((acc, v) => acc + v.confirmed, 0);
		return { date: daily[i].date, avg: sum / slice.length };
	});
	const short = (n: number) => {
		const abs = Math.abs(n);
		if (abs >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(2)}B`;
		if (abs >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
		if (abs >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
		return `${Math.round(n)}`;
	};
	return (
		<ResponsiveContainer width="100%" height={height}>
			<LineChart
				data={daily}
				margin={{ top: 6, right: 12, left: 4, bottom: 0 }}
			>
				<CartesianGrid strokeDasharray="3 3" />
				<XAxis
					dataKey="date"
					tickFormatter={(d: string | number) =>
						format(parseISO(String(d)), "MM/dd")
					}
					minTickGap={24}
					tickCount={Math.min(6, Math.max(2, daily.length))}
					tick={{ fill: "#e2e6ec", fontSize: 11 }}
				/>
				<YAxis
					width={56}
					domain={scale === "log" ? [1, "dataMax"] : [0, "dataMax"]}
					allowDecimals={false}
					tickFormatter={short}
					tick={{ fill: "#e2e6ec", fontSize: 11 }}
					scale={scale}
				/>
				<Tooltip
					labelFormatter={(d: string) => format(parseISO(String(d)), "PPP")}
					formatter={(v: number, name: string) => {
						if (name === "avg7")
							return [short(Number(v)), "7d Avg"] as [string, string];
						return [
							short(Number(v)),
							name.charAt(0).toUpperCase() + name.slice(1),
						] as [string, string];
					}}
					contentStyle={{
						background: "#1e2530",
						border: "1px solid #3a4450",
						borderRadius: 6,
						color: "#e2e6ec",
						boxShadow: "0 4px 12px rgba(0,0,0,0.6)",
						padding: "6px 8px",
					}}
					itemStyle={{ color: "#e2e6ec", padding: 0 }}
					cursor={{ stroke: "#3a4450", strokeWidth: 1 }}
				/>
				<Legend
					verticalAlign="top"
					align="right"
					wrapperStyle={{ fontSize: "0.7rem", paddingBottom: 4 }}
				/>
				<Line
					dataKey="confirmed"
					name="Confirmed"
					stroke="#2962ff"
					strokeWidth={2}
					activeDot={{ r: 5 }}
				/>
				<Line
					dataKey="deaths"
					name="Deaths"
					stroke="#ff6b6b"
					strokeWidth={2}
					activeDot={{ r: 5 }}
				/>
				<Line
					data={avg7}
					dataKey="avg"
					name="avg7"
					stroke="#ffd166"
					dot={false}
					strokeDasharray="4 3"
					strokeWidth={2}
				/>
			</LineChart>
		</ResponsiveContainer>
	);
}
