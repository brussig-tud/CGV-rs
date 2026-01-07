import Module from "./slang-wasm.js";

// A container storing a `Slang` *composite component* made from several modules and/or entry points.
export class SlangComposite
{
	constructor(slangContext, compositeHandle, compositeObject)
	{
		this.object = compositeObject;

		// Internal API
		this.cleanup = () => {
			this.object = null;
		}
	}
}

// A container storing a `Slang` component that represents an entry point of a Slang module.
export class SlangEntryPoint
{
	constructor(slangContext, epHandle, componentObject)
	{
		this.object = componentObject;

		// Internal API
		this.cleanup = () => {
			this.object = null;
		}

		// API methods
		this.name = () => this.object.getName();
	}
}

// The module container storing the assigned module handle and its owned module components (the entry points)
export class SlangModule
{
	constructor(slangContext, moduleHandle, moduleObject)
	{
		this.object = moduleObject;
		this.entryPoints = {};
		[...Array(this.object.getDefinedEntryPointCount()).keys()].map(
			(i) => {
				const handle = SlangContext.getUniqueNumber();
				const newEP = new SlangEntryPoint(
					slangContext, handle, this.object.getDefinedEntryPoint(i)
				);
				slangContext.entryPoints.set(handle, newEP);
				this.entryPoints[newEP.name()] = handle;
			}
		);

		// Internal API
		this.cleanup = () => {
			// Entry points
			Object.entries(this.entryPoints).forEach(([_, handle]) => {
				slangContext.getEntryPoint(handle).cleanup();
				slangContext.entryPoints.delete(handle);
			});
			this.entryPoints = null;
			this.object = null;
		}

		// API methods
		this.getEntryPoints = () => {
			const wireHandles = Object.entries(this.entryPoints).map(
				([_, handle]) => BigInt(handle)
			).toArray();
			return wireHandles;
		};
	}
}

// The session container storing the assigned session handle and its owned modules
export class SlangSession
{
	constructor(slangContext, globalSession, sessionHandle, sessionObject)
	{
		this.object = sessionObject;
		this.globalSession = globalSession;
		this.modules = new Set();
		this.composites = new Set();

		// Internal API
		this.cleanup = () => {
			// Modules
			this.modules.forEach((handle) => {
				slangContext.getModule(handle).cleanup();
				slangContext.modules.delete(handle);
			});
			this.modules = null;
			// Composites
			this.composites.forEach((handle) => {
				slangContext.getComposite(handle).cleanup();
				slangContext.composites.delete(handle);
			});
			this.composites = null;
			this.object = null;
			this.globalSession = null;
		}

		// Public API
		this.loadModuleFromSource = (moduleName, modulePath, moduleSourceCode) => {
			const newModule = this.object.loadModuleFromSource(moduleSourceCode, moduleName, modulePath);
			if (!newModule)
				return slangContext.handleContextError();
			const handle = SlangContext.getUniqueNumber();
			slangContext.modules.set(handle, new SlangModule(slangContext, handle, newModule));
			this.modules.add(handle);
			console.info("Session #"+sessionHandle+": loaded new Slang module #."+handle+":");
			console.info(newModule);
			console.info("Session #"+sessionHandle+" modules now:");
			console.info(this.modules);
			return BigInt(handle);
		}
		this.createComposite = (handles) => {
			/*const newComposite = this.object.createComposite(handles);
			if (!newComposite)
				return slangContext.handleContextError();
			const handle = SlangContext.getUniqueNumber();
			this.composites.set(handle, new SlangComposite(slangContext, handle, newComposite));
			console.info("Session #"+sessionHandle+": created new composite #."+handle+":");
			console.info(newComposite);
			console.info("Session #"+sessionHandle+" composites now:");
			console.info(this.composites);*/
		}
	}
}

// The global session container storing the assigned global session handle and its owned sessions
export class SlangGlobalSession
{
	constructor(slangContext, globalSessionHandle, globalSessionObject)
	{
		this.object = globalSessionObject;
		this.handle = globalSessionHandle;
		this.sessions = new Set();

		// Internal API
		this.cleanup = () => {
			// Sessions
			if (this.sessions.size > 0)
				console.warn(
					"Destroying global session #"+this.handle+" which still owns "+this.sessions.size+" sessions"
				);
			const ownedSessionHandles = structuredClone(this.sessions);
			ownedSessionHandles.forEach((handle) => slangContext.dropSession(handle));
			this.sessions = null;
			this.object = null;
			this.handle = null;
		}

		// API methods
		this.createSession = () => {
			const compilationTarget = "WGSL";
			const target = slangContext.availableTargets[compilationTarget];
			console.info(
				  "Global session #"+this.handle+": New session requested for target: "
				+ compilationTarget+"("+target+")");
			const newSession = this.object.createSession(target);
			if (!newSession)
				return slangContext.handleContextError();
			let handle = SlangContext.getUniqueNumber();
			slangContext.sessions.set(handle, new SlangSession(slangContext, this, handle, newSession));
			this.sessions.add(handle);
			console.info("Global session #"+this.handle+": created new session #"+handle+":");
			console.info(newSession);
			console.info("Global session #"+this.handle+": sessions now:");
			console.info(this.sessions);
			console.info("All sessions now:");
			console.info(slangContext.sessions);
			return BigInt(handle);
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
		this.sessions = new Map();
		this.modules = new Map();
		this.entryPoints = new Map();
		this.composites = new Map();

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
			globalSession.cleanup();
			this.globalSessions.delete(handle_bg);
			console.info("Dropped Slang global session #"+handle_bg);
			console.info("Slang global sessions now:");
			console.info(this.globalSessions);
		}
		this.getSession = (handle) => {
			const handle_bg = Number(handle);
			const session = this.sessions.get(handle_bg);
			if (session === undefined)
				console.error("Invalid Slang session handle: "+handle_bg);
			return session;
		};
		this.dropSession = (handle) => {
			const handle_bg = Number(handle);
			let session = this.getSession(handle_bg);
			if (session === undefined) {
				console.error("Attempted to drop non-existent session handle: "+handle_bg);
				return;
			}
			let globalSession = session.globalSession;
			session.cleanup();
			if (!globalSession.sessions.delete(handle_bg))
				console.error("INTERNAL STATE CORRUPTION: session #"+handle_bg+" was orphaned from its parent global session");
			this.sessions.delete(handle_bg);
			session = null;
			console.info("Global session #"+globalSession.handle+": dropped session #"+handle_bg);
			console.info("Global session #"+globalSession.handle+": sessions now:");
			console.info(globalSession.sessions);
			console.info("All sessions now:");
			console.info(this.sessions);
		}
		this.getModule = (handle) => {
			const handle_bg = Number(handle);
			const module = this.modules.get(handle_bg);
			if (module === undefined)
				console.error("Invalid Slang module handle: "+handle_bg);
			return module;
		};
		/* this.dropModule = (handle) */ // <- not implemented (Slang does not allow removing modules from a session)
		this.getEntryPoint = (handle) => {
			const handle_bg = Number(handle);
			const entryPoint = this.entryPoints.get(handle_bg);
			if (entryPoint === undefined)
				console.error("Invalid Slang entry point handle: "+handle_bg);
			return entryPoint;
		};
		/* this.dropEntryPoint = (handle) */ // <- not implemented (Entry points are intrinsically linked to modules)
		this.getComposite = (handle) => {
			const handle_bg = Number(handle);
			const composite = this.entryPoints.get(handle_bg);
			if (composite === undefined)
				console.error("Invalid Composite handle: "+handle_bg);
			return composite;
		};
		this.dropComposite = (handle) => {
			const handle_bg = Number(handle);
			let composite = this.getComposite(handle_bg);
			if (composite === undefined) {
				console.error("Attempted to drop non-existent composite handle: "+handle_bg);
				return;
			}
			/*let globalSession = session.globalSession;
			session.cleanup();
			if (!globalSession.sessions.delete(handle_bg))
				console.error("INTERNAL STATE CORRUPTION: composite #"+handle_bg+" was orphaned from its parent session");
			this.sessions.delete(handle_bg);
			session = null;
			console.info("Global session #"+globalSession.handle+": dropped session #"+handle_bg);
			console.info("Global session #"+globalSession.handle+": sessions now:");
			console.info(globalSession.sessions);
			console.info("All sessions now:");
			console.info(this.sessions);*/
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
	targetObj.slangjs_dropSession = function (sessionHandle) {
		targetObj.slangCtx.dropSession(sessionHandle);
	};
	targetObj.slangjs_session_loadModuleFromSource = function (sessionHandle, moduleName, modulePath, moduleSourceCode) {
		let session = targetObj.slangCtx.getSession(sessionHandle);
		return session.loadModuleFromSource(moduleName, modulePath, moduleSourceCode);
	};
	targetObj.slangjs_module_getEntryPoints = function (sessionHandle, moduleHandle) {
		// ToDo: flatten module storage inside context, and only store list of handles in global session container
		let session = targetObj.slangCtx.getSession(sessionHandle);
		let module = session.getModule(moduleHandle);
		return module.getEntryPoints();
	};
}
