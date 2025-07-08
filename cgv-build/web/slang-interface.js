import Module from "./slang-wasm.js";

// The Slang compilation and reflection context used by the JavaScript bridge to serve Slang shader-related requests.
export class SlangContext {
	// Helper function for exiting context creation on error.
	static handleSlangError (slangModule) {
		const error = slangModule.getLastError();
		return {context: null, message: (error.type + " error: " + error.message)};
	}

	// Create a Slang context using the given Slang WASM module.
	static create (slangModule)
	{
		// Perform initialization
		let globalSession;
		let compileTargets = {};
		try {
			// Create global session
			globalSession = slangModule.createGlobalSession();
			if (!globalSession)
				return this.handleSlangError(slangModule);

			// Log feasible compile targets
			// - obtain compile target map from Slang
			let compileTargetMap = slangModule.getCompileTargets();
			if (!compileTargetMap)
				return this.handleSlangError(slangModule);
			// - scan list for feasible targets
			let numValidTargets = 0;
			for (let i=0; i<compileTargetMap.length; i++)
			{
				const target = compileTargetMap[i];
				if (target.name === "WGSL") {
					compileTargets["WGSL"] = target.value;
					numValidTargets++;
				}
				else if (target.name === "SPIRV") {
					compileTargets["SPIRV"] = target.value;
					numValidTargets++;
				}
			}
			if (numValidTargets < 1)
				return {context: null, message: "Slang did not report any feasible compilation targets"};
		} catch (e) {
			return {context: null, message: ''+e};
		}

		// Done, create the SlangContext object
		return {context: new SlangContext(slangModule, globalSession, compileTargets), message: "Success."};
	}

	// The actual constructor.
	constructor(slangModule, globalSession, compileTargets) {
		this.slang = slangModule;
		this.globalSession = globalSession;
		this.availableTargets = compileTargets;
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
