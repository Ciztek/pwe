import { format, parseISO } from "date-fns";
import {
	Area,
	AreaChart,
	CartesianGrid,
	Legend,
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
}: {
	data: SeriesPoint[];
	color?: string;
}) {
	return (
		<ResponsiveContainer width="100%" height={240}>
			<LineChart data={data} margin={{ top: 8, right: 24, left: 0, bottom: 0 }}>
				<CartesianGrid strokeDasharray="3 3" />
				<XAxis
					dataKey="date"
					tickFormatter={(d: string | number) =>
						format(parseISO(String(d)), "MM/dd")
					}
				/>
				<YAxis />
				<Tooltip
					labelFormatter={(d: any) => format(parseISO(String(d)), "PPP")}
				/>
				<Legend />
				<Line type="monotone" dataKey="value" stroke={color} dot={false} />
			</LineChart>
		</ResponsiveContainer>
	);
}

export function StackedAreaChart({
	data,
}: {
	data: Array<{
		date: string;
		confirmed: number;
		deaths: number;
		recovered: number;
	}>;
}) {
	return (
		<ResponsiveContainer width="100%" height={260}>
			<AreaChart data={data} margin={{ top: 8, right: 24, left: 0, bottom: 0 }}>
				<CartesianGrid strokeDasharray="3 3" />
				<XAxis
					dataKey="date"
					tickFormatter={(d: string | number) =>
						format(parseISO(String(d)), "MM/dd")
					}
				/>
				<YAxis />
				<Tooltip
					labelFormatter={(d: any) => format(parseISO(String(d)), "PPP")}
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
