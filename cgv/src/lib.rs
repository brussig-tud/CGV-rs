
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]



//////
//
// Module definitions
//

// Enable tests to compile cleanly
#[doc(hidden)]
pub extern crate self as cgv;

// The module implementing the Player
pub mod player;
pub use player::{Player, InputEvent, EventOutcome, RenderSetup, ManagedBindGroupLayouts}; // re-export

// The module encapsulating all low-level graphics objects.
mod context;
pub use context::Context; // re-export

// The module encapsulating rendering-related higher-level managed render state (common uniform buffers etc.)
pub mod renderstate;
pub use renderstate::RenderState; // re-export

/// The parent module of all GPU abstractions.
pub mod hal;

/// The module providing implementations for various common gpu compute tasks.
pub mod gpu;

/// The module containing high-level rendering facilities.
pub mod renderer;
pub use renderer::Renderer; // re-export

/// The module containing all viewing functionality.
pub mod view;

/// The module providing various reusable UI components
pub mod gui;

/// The module providing functionality related to data handling
pub mod data;

/// Re-export cgv-shader
pub use cgv_shader as shader;

/// Make sure we can access glm functionality as such
pub extern crate nalgebra_glm as glm;

/// Re-export important 3rd party libraries/library components
pub use tracing;
pub use anyhow::{anyhow, Result, Error};
pub use rand;
pub use rand_distr;
pub mod time {
	pub use web_time::{Instant as Instant, Duration as Duration};
}
pub use eframe::{wgpu as wgpu, egui as egui};
pub use egui_extras;

/// Unit tests
#[cfg(test)]
mod tests;



//////
//
// Imports
//

// Standard library
use std::any::Any;

// Ctor library
#[cfg(not(target_arch="wasm32"))]
use ctor;

// Tracing library
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// CGV imports
pub use cgv_util as util; // re-export
pub use cgv_runenv as run; // re-export



//////
//
// Vault
//

// Populate the vault.
#[cfg(not(target_arch="wasm32"))]
#[ctor::ctor(unsafe)]
fn initTracingProxy ()
{
	#[cfg(target_os="windows")]
		let ansiTermResult = ansi_term::enable_ansi_support();
	initTracing();
	#[cfg(target_os="windows")]
		if let Err(err) = ansiTermResult {
			tracing::error!("Failed to enable ANSI terminal support: {err}");
		}
}

// Encapsulate common init tasks for the tracing crate.
fn initTracing ()
{
	// Set up logging
	let mut envFilterBuilder = EnvFilter::builder();
	#[cfg(debug_assertions)] {
		envFilterBuilder = envFilterBuilder.with_default_directive(tracing::Level::DEBUG.into());
	}
	#[cfg(not(debug_assertions))] {
		envFilterBuilder = envFilterBuilder.with_default_directive(tracing::Level::INFO.into());
	}
	let envFilter = envFilterBuilder
		.from_env_lossy()
		.add_directive("wgpu_core::device::resource=warn".parse().expect(
			"Failed to set up logging/tracing facilities!"
		));

	let subscriber = tracing_subscriber::registry().with(envFilter);
	#[cfg(target_arch="wasm32")] {
		use tracing_wasm::{WASMLayer, WASMLayerConfig};

		console_error_panic_hook::set_once();
		let wasm_layer = WASMLayer::new(WASMLayerConfig::default());

		subscriber.with(wasm_layer).init();
	}
	#[cfg(not(target_arch="wasm32"))] {
		let fmt_layer = tracing_subscriber::fmt::Layer::default();
		subscriber.with(fmt_layer).init();
	}
}



///////
//
// Enums and structs
//

/// The common color type.
///
/// **TODO**: put into to-be-done `media` module/crate.
pub type RGBA = egui::ecolor::Rgba;

/// Holds information about the eye(s) in a stereo render pass.
#[derive(Debug)]
pub struct StereoEye {
	/// The index of the eye currently being rendered.
	pub current: u32,

	/// The maximum eye index in the current stereo render.
	pub max: u32
}

/// Enumerates the kinds of global render passes over the scene.
#[derive(Debug)]
pub enum GlobalPass
{
	/// A simple, straight-to-the-target global pass.
	Simple,

	/// A stereo pass - the encapsulated value indicates which eye exactly is being rendered currently.
	Stereo(StereoEye),

	/// A custom pass, with a custom value.
	Custom(Box<dyn Any + Send>)
}
impl GlobalPass
{
	/// Construct a `GlobalPass::Stereo` value for the given eye index, with the [`StereoEye::max`] field set to a
	/// *don't care* value. Useful for interacting with various [`view::Camera`] APIs that require you to state a global
	/// pass for the operation.
	///
	/// # Arguments
	///
	/// * `eye` – The eye index the stereo pass should refer to.
	///
	/// # Returns
	///
	/// A `GlobalPass::Stereo` value that refers to the specified eye index.
	///
	/// # Example
	///
	/// **TODO: add once [`view::Camera`] is overhauled**
	pub fn stereoQuery (eye: u32) -> Self {
		Self::Stereo(StereoEye { current: eye, max: u32::default() })
	}
}

/// Configuration for a global render pass.
/// **TODO: The design needs overhauling - cameras cannot know the *index* of any of their declared render passes, as
/// that also depends on other cameras.**
pub struct GlobalPassInfo
{
	pub pass: GlobalPass,
	/// Index of the associated [`RenderState`] within [`GlobalPasses::renderStates`].<br />
	/// **TODO: remove and replace by player-managed data structure that cameras don't interact with.**
	pub index: usize,
	pub clearColor: wgpu::Color,
	pub depthClearValue: f32,
	completionCallback: std::cell::Cell<Option<Box<dyn FnMut(&Context, u32) + Send>>>,
}

/// Global passes and their associated render state, defined by a camera.
#[derive(Clone,Copy)]
pub struct GlobalPasses<'cam> {
	pub info: &'cam [GlobalPassInfo],
	pub renderStates: &'cam [RenderState],
}



///////
//
// Traits
//

/// Convenience extension trait for our default color type to convert it to [`glm::Vec4`]
pub trait AsVec4 {
	fn as_vec4 (&self) -> &glm::Vec4;
}
impl AsVec4 for cgv::RGBA {
	fn as_vec4 (&self) -> &glm::Vec4 {
		unsafe {
			// SAFETY: `RGBA` is just `[f32; 4]`, exactly like `glm::Vec4`.
			&*(self as *const Self as *const glm::Vec4)
		}
	}
}


/// Traits that cannot be used or implemented outside this crate ([sealed trait idiom](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)).
mod private
{
	/// Restricts implementation of [`Component`](crate::Component) to this crate.
	pub trait ComponentSeal {}
}

/// Base trait for different kinds of objects owned by the [`Player`].
///
/// Implemented only for instantiations of [`ComponentObject`] to emulate field inheritance like in C++, with the
/// inherited fields defined by [`ComponentBase`]. The private supertrait ensures that other crates cannot add further
/// implementations.
pub trait Component: std::any::Any + Send + private::ComponentSeal {}
impl dyn Component
{
	/// Access the base class subobject present in every implementor.
	#[inline]
	pub fn base (&self) -> &crate::ComponentBase
	{
		// SAFETY: Component can only be implemented in this crate and is only implemented for ComponentObject, which is
		// layed out such that all instantiations are pointer-convertible to ComponentBase.
		unsafe{&*(self as *const Self).cast()}
	}
	/// Mutably access the base class subobject present in every implementor.
	#[inline]
	pub fn base_mut (&mut self) -> &crate::ComponentBase
	{
		// SAFETY: See above.
		unsafe{&mut *(self as *mut Self).cast()}
	}
}

/// Helper for upcasting trait objects of a [`Component`] subtrait to `dyn Component`.
pub trait DynComponent: AsRef<dyn Component> + AsMut<dyn Component> {}

/// Prefix of every type that implements [`Component`], like a base class subobject.
#[derive(Default)]
pub struct ComponentBase {}


/// Common layout for all [`Component`] types.
///
/// Emulates field inheritance like in C++ through "base class subobjects" in form of the fields `component` and `base`.
#[repr(C, align(2))]
pub struct ComponentObject<Base, User>
{
	/// "Base class subobject" for [`Component`].
	pub component: ComponentBase,
	/// "Base class subobject" for a given kind of component.
	///
	/// Every component trait has a corresponding `Base` type, e.g. [`ApplicationBase`] for [`Application`].
	pub base: Base,
	/// Data specific to each type that implements a component trait.
	pub user: User,
}
impl<Base, User> private::ComponentSeal for ComponentObject<Base, User> {}
/// [`Component`] marks all instantiations of this type.
impl<Base, User> Component for ComponentObject<Base, User>
	where Self: ::std::any::Any + ::std::marker::Send
{}
impl<Base, User> From<User> for ComponentObject<Base, User> where Base: Default
{
	/// Create a component object with defaulted base fields.
	fn from (user: User) -> Self
	{
		Self{component: Default::default(), base: Default::default(), user}
	}
}
/// Provide convenient access to the user data. In some situations, however, `Self::user` must be written explicitely to
/// appease the borrow checker.
impl<Base, User> std::ops::Deref for ComponentObject<Base, User>
{
	type Target = User;
	fn deref (&self) -> &User {&self.user}
}
impl<Base, User> std::ops::DerefMut for ComponentObject<Base, User>
{
	fn deref_mut (&mut self) -> &mut User {&mut self.user}
}

/// Expands to `$then` when `$if` is non-empty and `$else` otherwise.
macro_rules! ifElse {
	( ($($if:tt)+) { $($then:tt)* } $({ $($else:tt)* })? ) => { $($then)* };
	( (          ) { $($then:tt)* } $({ $($else:tt)* })? ) => { $($($else)*)? };
}
use ifElse;

/// Defines the trait for a specific kind of [`Component`].
macro_rules! componentKind {(
	$(#[$attr:meta])*
	$vis:vis trait $Component:ident
	{
		/// Type of the "base class subobject" for this component kind.
		base: $Base:ty;
		/// The interface for this component kind. Must be dyn compatible.
		$(
		$(#[$fnAttr:meta])*
		fn $fn:ident ( $($arg:ident: $argT:ty),* ) $(-> $returnT:ty)? $($body:block)? $(;)?
		)+
	}
) =>
{
	$(#[$attr])*
	///
	/// The sealed [`Component`] supertrait ensures that this trait can only be implemented for instantiations of
	/// [`ComponentObject`], although it does not enforce use of the correct base type. The generic parameter exists
	/// only so other crates can implement this trait with their own `User` types and is abstracted over by its default.
	$vis trait $Component<User = ()>: $crate::Component
	{
		$($crate::ifElse!{($($body)?)
			{$(#[$fnAttr])* fn $fn ( $($arg: $argT),* ) $(-> $returnT)? $($body)?}
			{$(#[$fnAttr])* fn $fn ( $($arg: $argT),* ) $(-> $returnT)?;}
		})+
	}
	/// Abstract over the trait's generic parameter by implementing the default instantiation as a forward to any
	/// concrete implementation. Note that, while the trait can be implemented for any [`ComponentObject`], this
	/// abstraction is only provided for instantiations with the correct base type.
	impl<User> $Component for ComponentObject<$Base, User> where Self: $Component<User>
	{
		$(
		#[inline(always)]
		fn $fn ( $($arg: $argT),* ) $(-> $returnT)?
		{
			$Component::<User>::$fn( $($arg),* )
		}
		)+
	}
	/// Implementing both the concrete and default instantiation of the trait makes it ambiguous to use, even though
	/// both implementations are identical. To avoid the need for verbose disambiguation, all trait methods are also
	/// aliased as associated methods, which take precedence.
	impl<User> ComponentObject<$Base, User> where Self: $Component<User>
	{
		$(
		#[inline(always)]
		pub fn $fn ( $($arg: $argT),* ) $(-> $returnT)?
		{
			$Component::<User>::$fn( $($arg),* )
		}
		)+
	}

	/// Upcast trait objects.
	impl ::std::convert::AsRef<dyn $crate::Component> for dyn $Component
	{
		#[inline(always)]
		fn as_ref (&self) -> &dyn $crate::Component {self}
	}
	impl ::std::convert::AsMut<dyn $crate::Component> for dyn $Component
	{
		#[inline(always)]
		fn as_mut (&mut self) -> &mut dyn $crate::Component {self}
	}
	impl $crate::DynComponent for dyn $Component {}

	impl dyn $Component
	{
		/// Access the base class subobject for this component kind.
		#[inline]
		pub fn base (&self) -> &$Base
		{
			// SAFETY: While the generic $Component<T> can be implemented for any ComponentObject, the default signature
			// is only implemented for ComponentObjects with the correct Base. All such instantiations have the base
			// class subobjects as a prefix.
			unsafe{&(*(self as *const Self).cast::<ComponentObject<$Base, ()>>()).base}
		}

		/// Mutably access the base class subobject for this component kind.
		#[inline]
		pub fn base_mut (&mut self) -> &mut $Base
		{
			// SAFETY: See above.
			unsafe{&mut (*(self as *mut Self).cast::<ComponentObject<$Base, ()>>()).base}
		}
	}
}}
use componentKind;


////
// Application

componentKind!{
/// An application that can be [run](Player::run) by a [`Player`].
pub trait Application
{
	base: ApplicationBase;

	/// Report a short title for the application that can be displayed in the application tab bar of the [`Player`].
	///
	/// # Returns
	///
	/// A string slice containing a short descriptive title for the application.
	fn title (self: &Self) -> &str;

	/// Called once on creation of the application, before it's asked to create its pipelines.
	///
	/// # Arguments
	///
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	///
	/// # Returns
	///
	/// `Ok` if successful, or some descriptive error detailing the failure if not.
	fn preInit (self: &mut Self, player: &mut Player) -> Result<()>;

	/// Called when the [`Player`] changed global render state, e.g. because a new [`view::Camera`] became active. Since
	/// this could mean framebuffers with a different format and depth testing strategy, applications should (re-)create
	/// their pipelines accordingly. The [`Player`] guarantees that this will be called at least once before the
	/// application is asked to render its contribution to the scene.
	///
	/// **ToDo:** Make the framework detect compatible changes and only notify for incompatible global pass changes
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `renderSetup` – The global render setup of the *CGV-rs* [`Player`].
	/// * `globalPasses` – The list of global passes the application will need to render to.
	fn recreatePipelines (self: &mut Self, context: &Context, renderSetup: &RenderSetup, globalPasses: &GlobalPasses);

	/// Called once on creation of the application, after it was asked to create its pipelines.
	///
	/// # Arguments
	///
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	///
	/// # Returns
	///
	/// `Ok` if successful, or some descriptive error detailing the failure if not.
	fn postInit (self: &mut Self, player: &mut Player) -> Result<()>;

	/// Called when there is user input that can be processed.
	///
	/// # Arguments
	///
	/// * `event` – The input event that the application should inspect and possibly act upon.
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	/// * `this` - Provides access to `self` from outside this function, e.g. in an asynchronous callback.
	///
	/// # Returns
	///
	/// The [outcome](EventOutcome) of the event processing.
	fn input (self: &mut Self, event: &InputEvent, player: &mut Player, this: player::AppHandle) -> EventOutcome;

	/// Called when the main framebuffer was resized.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context, useful e.g. to re-create resources when they're affected by the resize.
	/// * `newSize` – The new main framebuffer size, in pixels.
	fn resize (self: &mut Self, context: &Context, newSize: glm::UVec2);

	/// Called when the [player](Player) wants to prepare a new frame for rendering.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context for rendering.
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	/// * `this` - Provides access to `self` from outside this function, e.g. in an asynchronous callback.
	///
	/// # Returns
	///
	/// `true` when the application deems a scene redraw is required, `false` otherwise.
	fn update (self: &mut Self, player: &mut Player, this: player::AppHandle) -> bool;

	/// Called when the [player](Player) is about to ask the application to render its contribution to the scene within
	/// a [global render pass](GlobalPassInfo).
	///
	/// # Arguments
	///
	/// * `context` – The graphics context for rendering.
	/// * `renderState` – The render state for the ongoing global render pass over the scene.
	/// * `globalPass` – Identifies the global render pass over the scene that spawned this call to `render`.
	///
	/// # Returns
	///
	/// `Some` array of command buffers containing any commands the application might need to perform before being able
	/// to render, or `None` if no preparation is required.
	fn prepareFrame (self: &mut Self, context: &Context, renderState: &RenderState, globalPass: &GlobalPassInfo)
		-> Option<Vec<wgpu::CommandBuffer>>;

	/// Called when the [player](Player) needs the application to render its contents.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context for rendering.
	/// * `renderState` – The render state for the ongoing global render pass over the scene.
	/// * `renderPass` – The *WGPU* render pass to the internally managed framebuffer that the application can add
	///                  draw calls to.
	/// * `globalPass` – Identifies the global pass over the scene that the render pass is for.
	fn render (
		self: &mut Self, context: &Context, renderState: &RenderState, managedRenderPass: &mut wgpu::RenderPass,
		globalPass: &GlobalPassInfo
	) -> Option<Vec<wgpu::CommandBuffer>>;

	/// Called when the [player](Player) needs the application to define its graphical main UI (which goes in the
	/// player's application panel). Independent or free-floating UI should be drawn in [`Application::freeUi`] instead.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object on which to define the application graphical UI.
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	fn ui (self: &mut Self, ui: &mut egui::Ui, ps: &mut Player);

	/// Called when the [player](Player) asks the application to define its free/independent UI (e.g. floating windows
	/// that should stay open even if the app loses player focus). Can be left unimplemented when not needed.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object on which to define the application graphical UI.
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	#[expect(unused_variables)]
	fn freeUi (self: &mut Self, ui: &mut egui::Ui, ps: &mut Player) {}
}}

/// Fields present in every [`Application`].
#[derive(Default)]
pub struct ApplicationBase {}

pub type AppObject<User> = ComponentObject<ApplicationBase, User>;



////
// ApplicationFactory

/// An object that can create instances of an applications that can be run by the [`Player`].
pub trait ApplicationFactory
{
	/// Create an instance of the target application.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `renderSetup` – The global render setup of the *CGV-rs* [`Player`].
	/// * `environment` – Runtime environment information (like the shader path) for the to-be-created application.
	///
	/// # Returns
	///
	/// A boxed instance of the application if successful, or some descriptive error detailing the failure if no
	/// instance could be created.
	fn create (
		&self, context: &Context, renderSetup: &RenderSetup, environment: run::Environment
	) -> Result<Box<dyn Application>>;
}
impl<F, User> ApplicationFactory for F
where
	F: for<'a, 'b> Fn(&'a Context, &'b RenderSetup, run::Environment)->Result<User>,
	AppObject<User>: Application
{
	fn create (&self, context: &Context, renderSetup: &RenderSetup, environment: run::Environment)
	-> Result<Box<dyn Application>> {
		Ok(Box::new(AppObject::from(self(context, renderSetup, environment)?)))
	}
}



///////
//
// Functions
//

///
#[cfg(feature="slang_runtime")]
pub fn obtainShaderCompileEnvironment () -> shader::compile::Environment<shader::slang::EnvModule>
{
	// Imports we only need here, when the feature `slang_runtime` is enabled
	use std::sync::LazyLock;
	use util::{uuid::Uuid, unique::Realm};

	// Statically keep the environment in memory
	static SHADER_LIB_ENVIRONMENT: LazyLock<shader::compile::Environment<shader::slang::EnvModule>> = LazyLock::new(||
		shader::compile::Environment::deserialize(util::sourceGeneratedBytes!("/coreshaderlib.env")).expect(
			"core shader library environment could not be deserialized"
		)
	);
	static SHADER_LIB_COUNTER: util::unique::RealmU32 = util::unique::RealmU32::one();
	let newCount = SHADER_LIB_COUNTER.newEntity();
	let newUuid = Uuid::from_u128(SHADER_LIB_ENVIRONMENT.uuid().as_u128() + newCount as u128);
	SHADER_LIB_ENVIRONMENT.cloneWithNewUuid(
		newUuid, &format!("{}_instance{newCount}", SHADER_LIB_ENVIRONMENT.label())
	)
}
