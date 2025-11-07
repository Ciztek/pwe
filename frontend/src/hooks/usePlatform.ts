export type Platform = "tauri" | "capacitor" | "web";

/**
 * Lightweight runtime platform detection.
 * - Tauri: window.__TAURI__ is defined by Tauri.
 * - Capacitor: window.Capacitor is present (Capacitor sets a global).
 * - Fallback: web.
 */
export function detectPlatform(): Platform {
	try {
		const w = window as unknown as Record<string, unknown>;
		if (typeof w.__TAURI__ !== "undefined") {
			return "tauri";
		}

		if (typeof w.Capacitor !== "undefined") {
			return "capacitor";
		}
	} catch {
		// ignore
	}
	return "web";
}

export function isTauri(): boolean {
	const w = window as unknown as Record<string, unknown>;
	return typeof w.__TAURI__ !== "undefined";
}

export function isCapacitor(): boolean {
	const w = window as unknown as Record<string, unknown>;
	return typeof w.Capacitor !== "undefined";
}
