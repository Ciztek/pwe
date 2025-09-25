import type { Platform } from "../hooks/usePlatform";
import { isCapacitor, isTauri } from "../hooks/usePlatform";

export function getPlatform(): Platform {
	if (isTauri()) return "tauri";
	if (isCapacitor()) return "capacitor";
	return "web";
}

export async function openExternal(url: string) {
	if (isTauri()) {
		// dynamic import so web bundlers don't include Tauri API if not used
		const { shell } = await import("@tauri-apps/api/shell");
		await shell.open(url);
		return;
	}

	if (isCapacitor()) {
		const capBrowser = (await import("@capacitor/browser")) as unknown as {
			Browser?: { open: (opts: { url: string }) => Promise<void> };
		};
		if (capBrowser.Browser && typeof capBrowser.Browser.open === "function") {
			await capBrowser.Browser.open({ url });
		}
		return;
	}

	window.open(url, "_blank", "noopener");
}

export async function notify(title: string, body?: string) {
	if (isTauri()) {
		const { Notification } = await import("@tauri-apps/api/notification");
		new Notification(title, { body }).show();
		return;
	}

	if (isCapacitor()) {
		// Capacitor local notifications require plugin; we fallback to web Notification
		if (window.Notification && Notification.permission === "granted") {
			new Notification(title, { body });
		} else if (window.Notification && Notification.permission !== "denied") {
			await Notification.requestPermission().then(
				(p) => p === "granted" && new Notification(title, { body }),
			);
		}
		return;
	}

	if (window.Notification && Notification.permission === "granted") {
		new Notification(title, { body });
	} else if (window.Notification && Notification.permission !== "denied") {
		await Notification.requestPermission().then(
			(p) => p === "granted" && new Notification(title, { body }),
		);
	}
}
