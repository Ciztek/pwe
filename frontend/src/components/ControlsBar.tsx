import type { ReactNode } from "react";

type Props = {
	title?: ReactNode;
	place: string;
	places: string[];
	start: string;
	end: string;
	chartMode?: "cumulative" | "daily";
	onChartModeChange?: (v: "cumulative" | "daily") => void;
	chartScale?: "linear" | "log";
	onChartScaleChange?: (v: "linear" | "log") => void;
	onRefresh?: () => void;
	mobileOrder?: "charts" | "map";
	onMobileOrderChange?: (v: "charts" | "map") => void;
	mobileView?: "kpi" | "map" | "leaderboard";
	onMobileViewChange?: (v: "kpi" | "map" | "leaderboard") => void;
	onPlaceChange: (v: string) => void;
	onStartChange: (v: string) => void;
	onEndChange: (v: string) => void;
};

export default function ControlsBar({
	title,
	place,
	places,
	start,
	end,
	chartMode = "cumulative",
	onChartModeChange,
	chartScale = "linear",
	onChartScaleChange,
	onRefresh,
	mobileView = "kpi",
	onMobileViewChange,
	onPlaceChange,
	onStartChange,
	onEndChange,
}: Props) {
	return (
		<header className="dashboard-header">
			<h2
				className="dashboard-title"
				role="button"
				tabIndex={0}
				onClick={() => onRefresh?.()}
				onKeyDown={(e) => {
					if (e.key === "Enter" || e.key === " ") {
						e.preventDefault();
						onRefresh?.();
					}
				}}
				title="Click to refresh data"
			>
				{title || "EpiCovid — Dashboard"}{" "}
				<span style={{ fontSize: "0.85rem", opacity: 0.75 }}>&#x21bb;</span>
			</h2>
			<div className="controls">
				<label>
					Place:
					<select value={place} onChange={(e) => onPlaceChange(e.target.value)}>
						{places.map((p) => (
							<option key={p} value={p}>
								{p}
							</option>
						))}
					</select>
				</label>
				<label>
					Start:
					<input
						type="date"
						value={start}
						onChange={(e) => onStartChange(e.target.value)}
					/>
				</label>
				<label>
					End:
					<input
						type="date"
						value={end}
						onChange={(e) => onEndChange(e.target.value)}
					/>
				</label>
				<label>
					Mode:
					<select
						value={chartMode}
						onChange={(e) =>
							onChartModeChange?.(e.target.value as "cumulative" | "daily")
						}
					>
						<option value="cumulative">Cumulative</option>
						<option value="daily">Daily New</option>
					</select>
				</label>
				<label>
					Scale:
					<select
						value={chartScale}
						onChange={(e) =>
							onChartScaleChange?.(e.target.value as "linear" | "log")
						}
					>
						<option value="linear">Linear</option>
						<option value="log">Log</option>
					</select>
				</label>
				<label className="mobile-only">
					View:
					<select
						value={mobileView}
						onChange={(e) =>
							onMobileViewChange?.(
								e.target.value as "kpi" | "map" | "leaderboard",
							)
						}
					>
						<option value="kpi">KPIs & Charts</option>
						<option value="map">Map</option>
						<option value="leaderboard">Leaderboard</option>
					</select>
				</label>
			</div>
		</header>
	);
}
