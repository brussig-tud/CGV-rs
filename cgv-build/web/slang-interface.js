import Module from "./slang-wasm.js";


// A container storing a `Slang` *composite component* made from several modules and/or entry points.
export class SlangComposite
{
	constructor(slangContext, session, compositeHandle, compositeObject)
	{
		// Common init
		this.object = compositeObject;
		this.session = session;

		// Setup reflection
		const layout = this.object.getLayout(0);
		if (layout)
		{
			// We have a linked component
			this.layout = layout;
			const layoutJson = layout.toJsonObject();
			var entryPoints = [];
			for (const epEntry in layoutJson.entryPoints)
				if (Object.prototype.hasOwnProperty.call(layoutJson.entryPoints, epEntry)) {
					const epInfo = layoutJson.entryPoints[epEntry];
					const ep = layout.findEntryPointByName(epInfo.name)
					entryPoints.push([ep, epInfo]);
				}
			this.entryPoints = entryPoints;
		}

		// Internal API
		this.cleanup = () => {
			this.object.delete();
			this.object = null;
		}

		// API methods
		this.orderedEntryPointNames = () => {
			let names_wire = [];
			this.entryPoints.forEach(
				([_, epInfo]) => names_wire.push(epInfo.name)
			);
			return names_wire;
		}
		this.link = () => {
			const linkedProg = this.object.link();
			if (!linkedProg)
				return slangContext.handleContextError();
			const handle = SlangContext.getUniqueNumber();
			slangContext.composites.set(handle, new SlangComposite(slangContext, this.session, handle, linkedProg));
			this.session.composites.add(handle);
			console.debug("Session #"+this.session.handle+": linked to new composite #."+handle+":");
			console.debug(linkedProg);
			console.debug("Session #"+this.session.handle+" composites now:");
			console.debug(this.session.composites);
			return BigInt(handle);
		}
		this.targetCode = (targetIdx) => {
			const code = this.object.getTargetCodeBlob(targetIdx);
			if (!code)
				return slangContext.handleCodeTranslationError();
			console.debug("Session #"+this.session.handle+", composite #."+compositeHandle+":");
			console.debug("Translated to target ("+targetIdx+"):\n");
			console.debug(code);
			return code;
		}
		this.entryPointCode = (entryPointIdx, targetIdx) => {
			const code = this.object.getEntryPointCodeBlob(entryPointIdx, targetIdx);
			if (!code)
				return slangContext.handleCodeTranslationError();
			console.debug("Session #"+this.session.handle+", composite #."+compositeHandle+":");
			console.debug("Translated to target ("+targetIdx+"):\n");
			console.debug(code);
			return code;
		}
	}
}


// A container storing a `Slang` *entry point* component that represents a possible entry to program execution.
export class SlangEntryPoint
{
	constructor(slangContext, epHandle, entryPointObject)
	{
		this.object = entryPointObject;

		// Internal API
		this.cleanup = () => {
			this.object.delete();
			this.object = null;
		}

		// API methods
		this.name = () => this.object.getName();
	}
}


// A container storing a `Slang` *module* and references to its *entry point* components.
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
			this.object.delete();
			this.object = null;
		}

		// API methods
		this.getEntryPoints = () => {
			let handles_wire = [];
			Object.entries(this.entryPoints).forEach(
				([_, handle]) => handles_wire.push(BigInt(handle))
			);
			return handles_wire;
		};
	}
}


// The session container storing the assigned session handle and its owned modules
export class SlangSession
{
	constructor(slangContext, globalSession, sessionHandle, sessionObject)
	{
		this.object = sessionObject;
		this.handle = sessionHandle;
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
			if (this.composites.size > 0)
				console.warn(
					"Destroying session #"+this.handle+" which still owns "+this.composites.size+" composites"
				);
			const ownedCompositeHandles = structuredClone(this.composites);
			ownedCompositeHandles.forEach((handle) => slangContext.dropComposite(handle));
			this.composites = null;
			this.object.delete();
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
			console.debug("Session #"+this.handle+": loaded new Slang module #."+handle+":");
			console.debug(newModule);
			console.debug("Session #"+this.handle+" modules now:");
			console.debug(this.modules);
			return BigInt(handle);
		}
		this.createComposite = (componentList) => {
			let newComposite = this.object.createCompositeComponentType(componentList);
			if (!newComposite)
				return slangContext.handleContextError();
			const handle = SlangContext.getUniqueNumber();
			slangContext.composites.set(handle, new SlangComposite(slangContext, this, handle, newComposite));
			this.composites.add(handle);
			console.debug("Session #"+this.handle+": created new composite #."+handle+":");
			console.debug(newComposite);
			console.debug("Session #"+this.handle+" composites now:");
			console.debug(this.composites);
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
			this.object.delete();
			this.handle = null;
		}

		// API methods
		this.createSession = () => {
			const compilationTarget = "WGSL";
			const target = slangContext.availableTargets[compilationTarget];
			console.debug(
				"Global session #"+this.handle+": New session requested for target: "
				+ compilationTarget+"("+target+")");
			const newSession = this.object.createSession(target);
			if (!newSession)
				return slangContext.handleContextError();
			let handle = SlangContext.getUniqueNumber();
			slangContext.sessions.set(handle, new SlangSession(slangContext, this, handle, newSession));
			this.sessions.add(handle);
			console.debug("Global session #"+this.handle+": created new session #"+handle+":");
			console.debug(newSession);
			console.debug("Global session #"+this.handle+": sessions now:");
			console.debug(this.sessions);
			console.debug("All sessions now:");
			console.debug(slangContext.sessions);
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
		this.componentLists = new Map();
		this.sessions = new Map();
		this.modules = new Map();
		this.entryPoints = new Map();
		this.composites = new Map();

		// Private API
		this.handleContextError = () => {
			const error = slangModule.getLastError();
			console.error(error.type + " error: " + error.message);
			return BigInt(-1);
		}
		this.handleCodeTranslationError = () => {
			const error = slangModule.getLastError();
			console.error(error.type + " error: " + error.message);
			return Uint8Array.fromHex("ff");
		}
		this.getGlobalSession = (handle) => {
			const handle_bg = Number(handle);
			const globalSession = this.globalSessions.get(handle_bg);
			if (globalSession === undefined)
				console.error("Invalid Slang global session handle: "+handle_bg);
			return globalSession;
		}
		this.getComponentList = (handle) => {
			const handle_bg = Number(handle);
			const componentList = this.componentLists.get(handle_bg);
			if (componentList === undefined)
				console.error("Invalid component list handle: "+handle_bg);
			return componentList;
		}
		this.getSession = (handle) => {
			const handle_bg = Number(handle);
			const session = this.sessions.get(handle_bg);
			if (session === undefined)
				console.error("Invalid Slang session handle: "+handle_bg);
			return session;
		}
		this.getModule = (handle) => {
			const handle_bg = Number(handle);
			const module = this.modules.get(handle_bg);
			if (module === undefined)
				console.error("Invalid Slang module handle: "+handle_bg);
			return module;
		}
		this.getEntryPoint = (handle) => {
			const handle_bg = Number(handle);
			const entryPoint = this.entryPoints.get(handle_bg);
			if (entryPoint === undefined)
				console.error("Invalid Slang entry point handle: "+handle_bg);
			return entryPoint;
		}
		this.getComposite = (handle) => {
			const handle_bg = Number(handle);
			const composite = this.composites.get(handle_bg);
			if (composite === undefined)
				console.error("Invalid Composite handle: "+handle_bg);
			return composite;
		}

		// API methods
		this.createGlobalSession = () => {
			// Create global session
			console.debug("New Slang global session requested");
			const newGlobalSession = slangModule.createGlobalSession();
			if (!newGlobalSession)
				return this.handleContextError();
			let handle = SlangContext.getUniqueNumber();
			this.globalSessions.set(handle, new SlangGlobalSession(this, handle, newGlobalSession));
			console.debug("Created new Slang global session #"+handle+":");
			console.debug(newGlobalSession);
			console.debug("Slang global sessions now:");
			console.debug(this.globalSessions);
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
			console.debug("Dropped Slang global session #"+handle_bg);
			console.debug("Slang global sessions now:");
			console.debug(this.globalSessions);
		}
		this.createComponentList = () => {
			console.debug("New component list requested");
			let handle = SlangContext.getUniqueNumber();
			const newComponentList = [];
			this.componentLists.set(handle, newComponentList);
			console.debug("Created new component list #"+handle+":");
			console.debug(newComponentList);
			console.debug("Component lists now:");
			console.debug(this.componentLists);
			return BigInt(handle);
		}
		this.dropComponentList = (handle) => {
			const handle_bg = Number(handle);
			let componentList = this.getComponentList(handle_bg);
			if (componentList === undefined) {
				console.error("Attempted to drop non-existent component list handle: "+handle_bg);
				return;
			}
			this.componentLists.delete(handle_bg);
			console.debug("Dropped component list #"+handle_bg);
			console.debug("Component lists now:");
			console.debug(this.componentLists);
		}
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
			console.debug("Global session #"+globalSession.handle+": dropped session #"+handle_bg);
			console.debug("Global session #"+globalSession.handle+": sessions now:");
			console.debug(globalSession.sessions);
			console.debug("All sessions now:");
			console.debug(this.sessions);
		}
		/* this.dropModule = (handle) */ // <- not implemented (Slang does not allow removing modules from a session)
		/* this.dropEntryPoint = (handle) */ // <- not implemented (Entry points are intrinsically linked to modules)
		this.dropComposite = (handle) => {
			const handle_bg = Number(handle);
			let composite = this.getComposite(handle_bg);
			if (composite === undefined) {
				console.error("Attempted to drop non-existent composite handle: "+handle_bg);
				return;
			}
			let session = composite.session;
			composite.cleanup();
			if (!session.composites.delete(handle_bg))
				console.error("INTERNAL STATE CORRUPTION: composite #"+handle_bg+" was orphaned from its parent session");
			this.composites.delete(handle_bg);
			composite = null;
			console.debug("Session #"+session.handle+": dropped composite #"+handle_bg);
			console.debug("Session #"+session.handle+": composites now:");
			console.debug(session.composites);
			console.debug("All composites now:");
			console.debug(this.composites);
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
		return targetObj.slangCtx.createGlobalSession();
	};
	targetObj.slangjs_dropGlobalSession = function (handle) {
		targetObj.slangCtx.dropGlobalSession(handle);
	};
	targetObj.slangjs_GlobalSession_createSession = function (globalSessionHandle) {
		let globalSession = targetObj.slangCtx.getGlobalSession(globalSessionHandle);
		return globalSession.createSession();
	};
	targetObj.slangjs_GlobalSession_dropSession = function (handle) {
		targetObj.slangCtx.dropSession(handle);
	};
	targetObj.slangjs_createComponentList = function () {
		return targetObj.slangCtx.createComponentList();
	};
	targetObj.slangjs_dropComponentList = function (handle) {
		targetObj.slangCtx.dropComponentList(handle);
	};
	targetObj.slangjs_ComponentList_addModule = function (componentListHandle, handle) {
		let ctx = targetObj.slangCtx;
		const componentList = ctx.getComponentList(componentListHandle);
		const module = ctx.getModule(handle)
		componentList.push(module.object);
	};
	targetObj.slangjs_ComponentList_addEntryPoint = function (componentListHandle, handle) {
		let ctx = targetObj.slangCtx;
		const componentList = ctx.getComponentList(componentListHandle);
		const entryPoint = ctx.getEntryPoint(handle)
		componentList.push(entryPoint.object);
	};
	targetObj.slangjs_ComponentList_addComposite = function (componentListHandle, handle) {
		let ctx = targetObj.slangCtx;
		const componentList = ctx.getComponentList(componentListHandle);
		const composite = ctx.getComposite(handle)
		componentList.push(composite.object);
	};
	targetObj.slangjs_Session_loadModuleFromSource = function (sessionHandle, moduleName, modulePath, moduleSourceCode) {
		let session = targetObj.slangCtx.getSession(sessionHandle);
		return session.loadModuleFromSource(moduleName, modulePath, moduleSourceCode);
	};
	targetObj.slangjs_Session_createComposite = function (sessionHandle, componentListHandle) {
		let ctx = targetObj.slangCtx;
		let session = ctx.getSession(sessionHandle);
		const componentList = ctx.getComponentList(componentListHandle);
		return session.createComposite(componentList);
	};
	targetObj.slangjs_Session_dropComposite = function (handle) {
		targetObj.slangCtx.dropComposite(handle);
	};
	targetObj.slangjs_Module_getEntryPoints = function (moduleHandle) {
		let module = targetObj.slangCtx.getModule(moduleHandle);
		return module.getEntryPoints();
	};
	targetObj.slangjs_EntryPoint_name = function (entryPointHandle) {
		let entryPoint = targetObj.slangCtx.getEntryPoint(entryPointHandle);
		return entryPoint.name();
	};
	targetObj.slangjs_Composite_orderedEntryPointNames = function (handle) {
		return targetObj.slangCtx.getComposite(handle).orderedEntryPointNames();
	};
	targetObj.slangjs_Composite_link = function (handle) {
		return targetObj.slangCtx.getComposite(handle).link();
	};
	targetObj.slangjs_Composite_targetCode = function (handle, targetIdx) {
		return targetObj.slangCtx.getComposite(handle).targetCode(targetIdx);
	};
	targetObj.slangjs_Composite_entryPointCode = function (handle, entryPointIdx, targetIdx) {
		return targetObj.slangCtx.getComposite(handle).entryPointCode(entryPointIdx, targetIdx);
	};
}
