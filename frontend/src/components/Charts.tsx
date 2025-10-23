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
} from "recharts";

type SeriesPoint = {
	date: string;
	value: number;
};

export function CasesLineChart({
	data,
	color = "#8884d8",
	height = "100%",
}: {
	data: SeriesPoint[];
	color?: string;
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
				/>
				<YAxis
					width={56}
					domain={[0, "dataMax"]}
					allowDecimals={false}
					tickFormatter={short}
				/>
				<Tooltip
					labelFormatter={(d: string) => format(parseISO(String(d)), "PPP")}
					formatter={(v: number) =>
						[short(Number(v)), "Confirmed"] as [string, string]
					}
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
}: {
	data: Array<{
		date: string;
		confirmed: number;
		deaths: number;
		recovered: number;
	}>;
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
				/>
				<YAxis
					width={56}
					domain={[0, "dataMax"]}
					allowDecimals={false}
					tickFormatter={short}
				/>
				<Tooltip
					labelFormatter={(d: string) => format(parseISO(String(d)), "PPP")}
					formatter={(v: number, name: string) =>
						[
							short(Number(v)),
							name.charAt(0).toUpperCase() + name.slice(1),
						] as [string, string]
					}
				/>
				<Area
					type="monotone"
					dataKey="confirmed"
					stackId="1"
					stroke="#8884d8"
					fill="#8884d8"
				/>
				<Area
					type="monotone"
					dataKey="recovered"
					stackId="1"
					stroke="#82ca9d"
					fill="#82ca9d"
				/>
				<Area
					type="monotone"
					dataKey="deaths"
					stackId="1"
					stroke="#ff6b6b"
					fill="#ff6b6b"
				/>
			</AreaChart>
		</ResponsiveContainer>
	);
}
