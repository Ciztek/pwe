import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.scss";
import App from "./App.tsx";
import { detectPlatform, isCapacitor } from "./hooks/usePlatform";

function initPlatform() {
	try {
		const platform = detectPlatform();
		document.body.classList.add(`platform-${platform}`);

		if (isCapacitor()) {
			// defer dynamic import until runtime; hide splashscreen if available
			import("@capacitor/core")
				.then((capModule) => {
					// Narrow the expected shape so we avoid blanket 'any' casts
					type CapacitorLike = {
						SplashScreen?: {
							hide?: () => void;
						};
					};
					const cap = capModule as unknown as CapacitorLike;
					try {
						// Capacitor splash screen plugin may be available
						if (
							cap &&
							cap.SplashScreen &&
							typeof cap.SplashScreen.hide === "function"
						) {
							cap.SplashScreen.hide();
						}
					} catch {
						// ignore runtime errors
					}
				})
				.catch(() => {});
		}
	} catch {
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
