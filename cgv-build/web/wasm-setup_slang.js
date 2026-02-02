// Import Slang JavaScript bridge
import slang_setupAndAddInterface from "./slang-interface.js"

// Make JavaScript bridge accessible to CGV-rs WASM module
console.info("Loading and linking Slang WASM module...");
const startTime = performance.now();
slang_setupAndAddInterface(window).then(
	() => {
		/* Log Slang JavaScript bridge initializaion result */ {
			const duration = performance.now()-startTime;
			const initMsg = "...took "+duration+"ms. Slang-WASM JavaScript bridge";
			if (window.hasOwnProperty('slangCtx') && window.slangCtx) {
				console.info(initMsg+" ready.");
			}
			else {
				console.error(initMsg+" failed to initialize!");
			}
		}

		@CODE__WASM_SETUP@
	}
);
