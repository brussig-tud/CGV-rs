import Module from "./slang-wasm.js";

// The Slang compilation and reflection context used by the JavaScript bridge to serve Slang shader-related requests.
export class SlangContext {
	// Create a Slang context using the given Slang WASM module.
	static create (slangModule)
	{
		// Perform initialization
		let globalSession;
		let compileTargetMap;
		try {
			globalSession = slangModule.createGlobalSession();
			compileTargetMap = slangModule.getCompileTargets();
			if (!globalSession || !compileTargetMap) {
				const error = slangModule.getLastError();
				return {context: null, message: (error.type + " error: " + error.message)};
			}
		} catch (e) {
			return {context: null, message: ''+e};
		}

		// Done, create the SlangContext object
		return {context: new SlangContext(slangModule, globalSession, compileTargetMap), message: "Success."};
	}
	constructor(slangModule, globalSession, compileTargetMap) {
		this.slang = slangModule;
		this.globalSession = globalSession;
		this.availableTargets = compileTargetMap;
	}
}

// Initialize Slang WASM module and add the JavaScript bridge functions to the provided Object.
export default async function slang_setupAndAddInterface (targetObj)
{
	// Load and link the Slang WASM Module
	let result = SlangContext.create(await Module());
	if (!result.context) {
		console.error(result.message);
		return;
	}
	targetObj.slangCtx = result.context;

	// Set up bridging interface
	targetObj.slangjs_interopTest = function (moduleSourceCode) {
		// We assume that moduleSourceCode is a String. Convert it into Uint8Array
		console.info("slangjs_interopTest(): Using Slang Context:");
		console.info(targetObj.slangCtx);
		console.info('slangjs_interopTest(): Echoing bytes of received string "'+moduleSourceCode+'"');
		const encoder = new TextEncoder();
		return encoder.encode(moduleSourceCode);
	};
}
