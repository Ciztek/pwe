// Minimal ambient module declarations for Capacitor used by the app.
declare module "@capacitor/core" {
	export const Capacitor: unknown;
	export const SplashScreen: {
		hide?: () => void;
	};
	const def: unknown;
	export default def;
}

declare module "@capacitor/browser" {
	export const Browser: {
		open: (opts: { url: string }) => Promise<void>;
	};
}
