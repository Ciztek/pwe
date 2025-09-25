export type Platform = "tauri" | "capacitor" | "web";

/**
 * Lightweight runtime platform detection.
 * - Tauri: window.__TAURI__ is defined by Tauri.
 * - Capacitor: window.Capacitor is present (Capacitor sets a global).
 * - Fallback: web.
 */
export function detectPlatform(): Platform {
	try {
		// @ts-expect-error
		if (typeof (window as any).__TAURI__ !== "undefined") {
			return "tauri";
		}

		// @ts-expect-error
		if (typeof (window as any).Capacitor !== "undefined") {
			return "capacitor";
		}
	} catch (e) {
		// ignore
	}
	return "web";
}

export function isTauri(): boolean {
	// @ts-expect-error
	return typeof (window as any).__TAURI__ !== "undefined";
}

export function isCapacitor(): boolean {
	// @ts-expect-error
	return typeof (window as any).Capacitor !== "undefined";
}
