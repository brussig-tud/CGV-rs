import Module from "./slang-wasm.js";

// The Slang compilation and reflection context used by the JavaScript bridge to serve Slang shader-related requests.
class SlangContext {
	constructor(slangModule) {
		this.slangModule = slangModule;
		this.globalSession = this.slangModule.createGlobalSession();
	}
}

// Initialize Slang WASM module and add the JavaScript bridge functions to the provided Object.
export default async function slang_setupAndAddInterface (targetObj)
{
	// Load and link the Slang WASM Module
	targetObj.slangCtx = new SlangContext(await Module());

	// Set up bridging interface
	targetObj.slangjs_interopTest = function (moduleSourceCode) {
		// We assume that moduleSourceCode is a String. Convert it into an Uint8Array
		console.info("slangjs_interopTest(): Using Slang Context:");
		console.info(targetObj.slangCtx);
		console.info('slangjs_interopTest(): Echoing bytes of received string "'+moduleSourceCode+'"');
		const encoder = new TextEncoder();
		return encoder.encode(moduleSourceCode);
	};
}
