// Minimal ambient module declarations for Tauri JS API packages
// These allow TypeScript to compile when the @tauri-apps/api package types are not available.
declare module "@tauri-apps/api/shell" {
	export function open(url: string): Promise<void>;
	export const shell: {
		open: (url: string) => Promise<void>;
	};
}

declare module "@tauri-apps/api/notification" {
	export class Notification {
		constructor(title: string, options?: { body?: string });
		show(): void;
	}
	const NotificationConstructor: typeof Notification;
	export { NotificationConstructor as Notification };
}
