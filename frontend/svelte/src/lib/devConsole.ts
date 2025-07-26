// Development-only console logging utility
// This helps remove console logs from production builds

const isDev = import.meta.env.DEV;

export const devConsole = {
	log: (...args: unknown[]) => {
		if (isDev) {
			console.log(...args);
		}
	},
	error: (...args: unknown[]) => {
		if (isDev) {
			console.error(...args);
		}
	},
	warn: (...args: unknown[]) => {
		if (isDev) {
			console.warn(...args);
		}
	}
};
