import Module from "./slang-wasm.js";

// The session container storing the assigned session handle and its owned modules
export class SlangSession {
	constructor(sessionObject) {
		this.object = sessionObject;
		this.modules = new Map();
	}
}

// The Slang compilation and reflection context used by the JavaScript bridge to serve Slang shader-related requests.
export class SlangContext
{
	// Helper function for obtaining a guaranteed unique number (e.g. for use as a handle)
	static getUniqueNumber () {
		return Math.floor(Date.now() * Math.random());
	}

	// Helper function for exiting context creation on error.
	static handleContextCreationError (slangModule) {
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
				return this.handleContextCreationError(slangModule);

			// Log feasible compile targets
			// - obtain compile target map from Slang
			let compileTargetMap = slangModule.getCompileTargets();
			if (!compileTargetMap)
				return this.handleContextCreationError(slangModule);
			// - scan list for feasible targets
			let numValidTargets = 0;
			for (let i=0; i<compileTargetMap.length; i++)
			{
				const target = compileTargetMap[i];
				if (target.name === "WGSL") {
					compileTargets["WGSL"] = target.value;
					numValidTargets++;
				}
				/*else if (target.name === "SPIRV") {
					compileTargets["SPIRV"] = target.value;
					numValidTargets++;
				}*/
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
	constructor(slangModule, globalSession, compileTargets)
	{
		// Fields
		this.slang = slangModule;
		this.globalSession = globalSession;
		this.availableTargets = compileTargets;
		this.sessions = new Map();

		// Helper methods
		this.getSession = (handle) => {
			const handle_bg = Number(handle);
			const session = this.sessions.get(handle_bg);
			if (session === undefined)
				console.error("Invalid Slang session handle: "+handle_bg);
			return session;
		};

		// API methods
		this.createSession = () => {
			const target = this.availableTargets["WGSL"];
			console.info("New Slang session requested for target: "+target);
			const newSession = this.globalSession.createSession(target);
			if (!newSession) {
				const error = this.slang.getLastError();
				console.error(error.type + " error: " + error.message);
				return BigInt(-1);
			}
			let handle = SlangContext.getUniqueNumber();
			this.sessions.set(handle, new SlangSession(newSession));
			console.info("Created new Slang session #"+handle+":");
			console.info(newSession);
			console.info("Slang sessions now:");
			console.info(this.sessions);
			return BigInt(handle);
		}
		this.dropSession = (handle) => {
			const handle_bg = Number(handle);
			let session = this.getSession(handle_bg);
			if (session === undefined) {
				console.error("Attempted to drop non-existent session handle: "+handle_bg);
				return;
			}
			session.object = null;
			this.sessions.delete(handle_bg);
			console.info("Dropped slang session #"+handle_bg);
			console.info("Slang sessions now:");
			console.info(this.sessions);
		}
		this.loadModuleFromSource = (sessionHandle, moduleName, modulePath, moduleSourceCode) => {
			const target = this.availableTargets["WGSL"];
			const session = this.getSession(sessionHandle);
			if (session === undefined)
				return BigInt(-1);
			const module = session.object.loadModuleFromSource(moduleSourceCode, moduleName, modulePath);
			if (!module) {
				const error = this.slang.getLastError();
				console.error(error.type + " error: " + error.message);
				return BigInt(-1);
			}
			const handle = SlangContext.getUniqueNumber();
			session.modules.set(handle, module);
			console.info("Session #"+sessionHandle+": loaded new Slang module #."+handle+":");
			console.info(module);
			console.info("Session #"+sessionHandle+" modules now:");
			console.info(session.modules);
			return BigInt(handle);
		}
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
	targetObj.slangjs_createSession = function () {
		let ctx = targetObj.slangCtx;
		return ctx.createSession();
	};
	targetObj.slangjs_dropSession = function (handle) {
		let ctx = targetObj.slangCtx;
		ctx.dropSession(handle);
	};
	targetObj.slangjs_loadModuleFromSource = function (sessionHandle, moduleName, modulePath, moduleSourceCode) {
		let ctx = targetObj.slangCtx;
		return ctx.loadModuleFromSource(sessionHandle, moduleName, modulePath, moduleSourceCode);
	};
}
