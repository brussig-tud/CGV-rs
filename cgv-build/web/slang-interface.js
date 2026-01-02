import Module from "./slang-wasm.js";

// The session container storing the assigned session handle and its owned modules
export class SlangSession
{
	constructor(slangContext, sessionHandle, sessionObject)
	{
		this.object = sessionObject;
		this.modules = new Map();

		// API methods
		this.loadModuleFromSource = (moduleName, modulePath, moduleSourceCode) => {
			const module = this.object.loadModuleFromSource(moduleSourceCode, moduleName, modulePath);
			if (!module)
				return slangContext.handleContextError();
			const handle = SlangContext.getUniqueNumber();
			this.modules.set(handle, module);
			console.info("Session #"+sessionHandle+": loaded new Slang module #."+handle+":");
			console.info(module);
			console.info("Session #"+sessionHandle+" modules now:");
			console.info(this.modules);
			return BigInt(handle);
		}
	}
}

// The global session container storing the assigned global session handle and its owned sessions
export class SlangGlobalSession
{
	constructor(slangContext, globalSessionHandle, globalSessionObject)
	{
		this.object = globalSessionObject;
		this.sessions = new Map();

		// API methods
		this.getSession = (handle) => {
			const handle_bg = Number(handle);
			const session = this.sessions.get(handle_bg);
			if (session === undefined)
				console.error("Invalid Slang session handle: "+handle_bg);
			return session;
		};
		this.createSession = () => {
			const compilationTarget = "WGSL";
			const target = slangContext.availableTargets[compilationTarget];
			console.info(
				  "Global session #"+globalSessionHandle+": New session requested for target: "
				+ compilationTarget+"("+target+")");
			const newSession = this.object.createSession(target);
			if (!newSession)
				return slangContext.handleContextError();
			let handle = SlangContext.getUniqueNumber();
			this.sessions.set(handle, new SlangSession(slangContext, handle, newSession));
			console.info("Global session #"+globalSessionHandle+": created new session #"+handle+":");
			console.info(newSession);
			console.info("Global session #"+globalSessionHandle+": sessions now:");
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
			console.info("Global session #"+globalSessionHandle+": dropped session #"+handle_bg);
			console.info("Global session #"+globalSessionHandle+": sessions now:");
			console.info(this.sessions);
		}
	}
}

// The Slang runtime context used by the JavaScript bridge to serve Slang shader-related requests.
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

	// Create a Slang context with the desired compilation targets using the given Slang WASM module.
	static create (slangModule)
	{
		// Perform initialization
		let compileTargets = {};
		try {
			// Log feasible compile targets
			const compileTargetArray = slangModule.getCompileTargets();
			if (!compileTargetArray)
				return this.handleContextCreationError(slangModule);
			compileTargetArray.forEach((compileTarget, index) => {
				compileTargets[compileTarget.name] = compileTarget.value;
			});
		} catch (e) {
			return {context: null, message: ''+e};
		}

		// Done, create the SlangContext object
		return {context: new SlangContext(slangModule, compileTargets), message: "Success."};
	}

	// The actual constructor.
	constructor(slangModule, compileTargets)
	{
		// Fields
		this.availableTargets = compileTargets;
		this.globalSessions = new Map();

		// Helper methods
		this.handleContextError = () => {
			const error = slangModule.getLastError();
			console.error(error.type + " error: " + error.message);
			return BigInt(-1);
		}
		this.getGlobalSession = (handle) => {
			const handle_bg = Number(handle);
			const globalSession = this.globalSessions.get(handle_bg);
			if (globalSession === undefined)
				console.error("Invalid Slang global session handle: "+handle_bg);
			return globalSession;
		};

		// API methods
		this.createGlobalSession = () => {
			// Create global session
			console.info("New Slang global session requested");
			const newGlobalSession = slangModule.createGlobalSession();
			if (!newGlobalSession)
				return this.handleContextError();
			let handle = SlangContext.getUniqueNumber();
			this.globalSessions.set(handle, new SlangGlobalSession(this, handle, newGlobalSession));
			console.info("Created new Slang global session #"+handle+":");
			console.info(newGlobalSession);
			console.info("Slang global sessions now:");
			console.info(this.globalSessions);
			return BigInt(handle);
		}
		this.dropGlobalSession = (handle) => {
			const handle_bg = Number(handle);
			let globalSession = this.getGlobalSession(handle_bg);
			if (globalSession === undefined) {
				console.error("Attempted to drop non-existent global session handle: "+handle_bg);
				return;
			}
			globalSession.object = null;
			this.globalSessions.delete(handle_bg);
			console.info("Dropped Slang global session #"+handle_bg);
			console.info("Slang global sessions now:");
			console.info(this.globalSessions);
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
	targetObj.slangjs_createGlobalSession = function () {
		let ctx = targetObj.slangCtx;
		return ctx.createGlobalSession();
	};
	targetObj.slangjs_dropGlobalSession = function (handle) {
		let ctx = targetObj.slangCtx;
		ctx.dropGlobalSession(handle);
	};
	targetObj.slangjs_createSession = function (globalSessionHandle) {
		let globalSession = targetObj.slangCtx.getGlobalSession(globalSessionHandle);
		return globalSession.createSession();
	};
	targetObj.slangjs_dropSession = function (globalSessionHandle, sessionHandle) {
		let globalSession = targetObj.slangCtx.getGlobalSession(globalSessionHandle);
		globalSession.dropSession(sessionHandle);
	};
	targetObj.slangjs_loadModuleFromSource = function (globalSessionHandle, sessionHandle, moduleName, modulePath, moduleSourceCode) {
		// ToDo: flatten sessions list inside context, and only store handles per global session container
		let globalSession = targetObj.slangCtx.getGlobalSession(globalSessionHandle);
		let session = globalSession.getSession(sessionHandle);
		return session.loadModuleFromSource(moduleName, modulePath, moduleSourceCode);
	};
}
