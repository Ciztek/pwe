import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { detectPlatform, isCapacitor } from "./hooks/usePlatform";

function initPlatform() {
	try {
		const platform = detectPlatform();
		document.body.classList.add(`platform-${platform}`);

		if (isCapacitor()) {
			// defer dynamic import until runtime; hide splashscreen if available
			// eslint-disable-next-line @typescript-eslint/ban-ts-comment
			// @ts-expect-error
			import("@capacitor/core")
				.then((cap) => {
					try {
						// Capacitor splash screen plugin may be available
						// eslint-disable-next-line @typescript-eslint/ban-ts-comment
						// @ts-expect-error
						if (
							cap &&
							cap.SplashScreen &&
							typeof cap.SplashScreen.hide === "function"
						) {
							// eslint-disable-next-line @typescript-eslint/ban-ts-comment
							// @ts-expect-error
							cap.SplashScreen.hide();
						}
					} catch (e) {
						// ignore
					}
				})
				.catch(() => {});
		}
	} catch (e) {
		// ignore
	}
}

const rootElement = document.getElementById("root");

initPlatform();

if (rootElement) {
	createRoot(rootElement).render(
		<StrictMode>
			<App />
		</StrictMode>,
	);
}
