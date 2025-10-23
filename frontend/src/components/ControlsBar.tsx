import type { ReactNode } from "react";

type Props = {
	title?: ReactNode;
	place: string;
	places: string[];
	start: string;
	end: string;
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
				<label style={{ marginLeft: 12 }}>
					Start:
					<input
						type="date"
						value={start}
						onChange={(e) => onStartChange(e.target.value)}
					/>
				</label>
				<label style={{ marginLeft: 12 }}>
					End:
					<input
						type="date"
						value={end}
						onChange={(e) => onEndChange(e.target.value)}
					/>
				</label>
			</div>
		</header>
	);
}
