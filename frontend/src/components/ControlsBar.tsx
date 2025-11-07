import type { ReactNode } from "react";

type Props = {
	title?: ReactNode;
	place: string;
	places: string[];
	start: string;
	end: string;
	mobileOrder?: "charts" | "map";
	onMobileOrderChange?: (v: "charts" | "map") => void;
	mobileView?: "kpi" | "map";
	onMobileViewChange?: (v: "kpi" | "map") => void;
	onPlaceChange: (v: string) => void;
	onStartChange: (v: string) => void;
	onEndChange: (v: string) => void;
};

export default function ControlsBar({
	title = <h2>EpiCovid — Dashboard</h2>,
	place,
	places,
	start,
	end,
	mobileOrder = "charts",
	onMobileOrderChange,
	mobileView = "kpi",
	onMobileViewChange,
	onPlaceChange,
	onStartChange,
	onEndChange,
}: Props) {
	return (
		<header className="dashboard-header">
			{title}
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
				<label className="mobile-only">
					Mobile layout:
					<select
						value={mobileOrder}
						onChange={(e) =>
							onMobileOrderChange?.(e.target.value as "charts" | "map")
						}
					>
						<option value="charts">Charts first</option>
						<option value="map">Map first</option>
					</select>
				</label>
				<label className="mobile-only">
					View:
					<select
						value={mobileView}
						onChange={(e) =>
							onMobileViewChange?.(e.target.value as "kpi" | "map")
						}
					>
						<option value="kpi">KPIs & Charts</option>
						<option value="map">Map</option>
					</select>
				</label>
			</div>
		</header>
	);
}
