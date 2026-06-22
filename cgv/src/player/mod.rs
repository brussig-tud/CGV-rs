
//////
//
// Module definitions
//

/// Private submodule holding the code for setting up our custom fonts
mod font;

/// Private submodule holding the factored-out built-in GUI definitions
mod ui;
pub use ui::SIDEPANEL_SAFETY_MARGINS; // re-export

/// Submodule providing the [`RenderSetup`].
mod rendersetup;
pub use rendersetup::RenderSetup; // - re-export

/// Submodule providing the [`ViewportCompositor`]
mod viewportcompositor;
use viewportcompositor::*;



//////
//
// Imports
//

// Standard library
use std::{any::Any, sync::Arc};
#[cfg(not(target_arch="wasm32"))]
use std::fs;

// Winit library
#[cfg(all(not(target_arch="wasm32"),not(target_os="windows"),not(target_os="macos"),feature="wayland"))]
#[allow(unused_imports)]
use winit::platform::wayland::EventLoopBuilderExtWayland;
#[cfg(all(not(target_arch="wasm32"),not(target_os="windows"),not(target_os="macos"),feature="x11"))]
#[allow(unused_imports)]
use winit::platform::x11::EventLoopBuilderExtX11;

// WGPU API
use wgpu;

// Egui library and framework
use egui;
use eframe::egui_wgpu;
use eframe::epaint;

// Local imports
use crate::*;
use crate::view::{Camera, CameraInteractor};



//////
//
// Macros
//

/// Internal helper macro companion to [`Player::activeCameras`] that avoids borrowing the entire player by only
/// referring to `$player.camera` .
///
/// **TODO: What about other active cameras?**
macro_rules! activeCameras {
	($player:expr) => { std::slice::from_ref(&$player.camera) };
}



///////
//
// Enums and structs
//

/// Struct containing information about a key event. Essentially replicates [`egui::Event::Key`].
#[derive(Debug)]
pub struct KeyInfo
{
	/// The key code of the key the event relates to. See [`egui::Event::Key`] for details.
	pub key: egui::Key,

	/// Whether this is a *press* event (`true`) or *release* (`false`). See [`egui::Event::Key`] for details.
	pub pressed: bool,

	/// When [`Self::pressed`] is `true`, indicates whether this is a generated *repeat* event due to the user holding the key
	/// down. See [`egui::Event::Key`] for details.
	pub repeat: bool,

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: egui::Modifiers
}

/// Struct containing information about a click.
#[derive(Debug)]
pub struct ClickInfo {
	/// The pointer button the click originated from.
	pub button: egui::PointerButton,

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: egui::Modifiers,

	/// The pointer coordinates within the main viewport at the time of the click, in pixels.
	pub position: glm::UVec2
}

/// Struct containing information about a mouse wheel event
#[derive(Debug)]
pub struct MouseWheelInfo {
	/// The amount of scrolling in logical screen points along each axis that the wheel movement(s) are equivalent to.
	pub amount: glm::Vec2,

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: egui::Modifiers
}

/// Struct containing information about a drag event.
#[derive(Debug)]
pub struct DragInfo {
	/// Which pointer buttons are down. Should be queried using the [`egui::PointerButton`] enum.
	pub buttons: [bool; 5],

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: egui::Modifiers,

	/// The direction of the drag, using logical screen points as unit.
	pub direction: glm::Vec2
}
impl DragInfo {
	/// Convenience method for querying the [`buttons`](DragInfo::buttons) field.
	#[inline(always)]
	pub fn button (&self, button: egui::PointerButton) -> bool {
		self.buttons[button as usize]
	}
}

/// Enumeration of input events.
#[derive(Debug)]
pub enum InputEvent
{
	/// An event related to keyboard state.
	Key(KeyInfo),

	/// A simple click or tap.
	Click(ClickInfo),

	/// A double click or tap.
	DoubleClick(ClickInfo),

	/// A triple click or tap.
	TripleClick(ClickInfo),

	/// A mouse wheel / scroll event.
	MouseWheel(MouseWheelInfo),

	/// A pre-processed drag motion (including touch screen swipes).
	Dragged(DragInfo)
}

/// Enumeration of possible event handling outcomes.
#[derive(Debug)]
pub enum EventOutcome
{
	/// The event was handled and should be closed. The wrapped `bool` indicates whether a redraw
	/// needs to happen as a result of the processing that was done.
	HandledExclusively(bool),

	/// The event was handled but should not be closed (i.e. subsequent handlers will also receive
	/// it). The wrapped `bool` indicates whether a redraw needs to happen as a result of the
	/// processing that was done.
	HandledDontClose(bool),

	/// The event was not handled.
	NotHandled
}

/// Collects all bind group layouts available for interfacing with the managed [render passes](GlobalPassInfo) over the
/// scene as set up by the *CGV-rs* [`Player`].
pub struct ManagedBindGroupLayouts {
	/// The layout of the bind group for the [viewing](renderstate::ViewingStruct) uniforms.
	pub viewing: wgpu::BindGroupLayout
}



/// Provides safe access to a global [`Player`] object.
mod instance {

use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::{AtomicUsize, Ordering}};
use super::Player;

/// Stores the global [`Player`] instance and tracks its state in a threadsafe manner, accounting for both
/// initialization and synchronization.
///
/// Similar to a `Mutex<Option<Player>>` without poisoning or waiting.
struct Lock
{
	data: UnsafeCell<MaybeUninit<Player>>,
	state: AtomicUsize,
}
// SAFETY: `Lock` acts as a mutex, see https://doc.rust-lang.org/std/sync/struct.Mutex.html#impl-Sync-for-Mutex%3CT%3E.
unsafe impl Sync for Lock where Player: Send {}
mod state
{
	pub const UNINIT:    usize = 0;
	pub const AVAILABLE: usize = 1;
	pub const LOCKED:    usize = 2;
}
macro_rules! msg
{
	(ACQ_LOCKED) => {"Attempted to acquire multiple simultaneous references to the CGV player"};
	(BAD_STATE)  => {"The CGV player lock is in an invalid state"};
}

static INSTANCE: Lock = Lock{
	data: UnsafeCell::new(MaybeUninit::uninit()),
	state: AtomicUsize::new(state::UNINIT),
};

/// Store a new [`Player`] in the global instance, dropping any previous value.
///
/// **Panics** if the player is currently locked.
#[inline]
pub fn set (player: Player) -> LockGuard
{
	match INSTANCE.state.swap(state::LOCKED, Ordering::Acquire) {
		state::UNINIT => {},
		state::AVAILABLE => unsafe{(*INSTANCE.data.get()).assume_init_drop()},
		state::LOCKED => panic!(msg!(ACQ_LOCKED)),
		_ => panic!(msg!(BAD_STATE))
	}
	unsafe{&mut*INSTANCE.data.get()}.write(player);
	return LockGuard(std::marker::PhantomData);
}

/// Acquire the global [`Player`] instance for exclusive access.
///
/// **Panics** if the player is uninitialized or locked already.
#[inline]
pub fn lock () -> LockGuard
{
	match INSTANCE.state.compare_exchange(
		state::AVAILABLE,
		state::LOCKED,
		Ordering::Acquire,
		Ordering::Relaxed,
	) {
		Ok(_) => LockGuard(std::marker::PhantomData),
		Err(state::LOCKED) => panic!(msg!(ACQ_LOCKED)),
		Err(state::UNINIT) => panic!("Attempted to acquire the CGV player while it is not running"),
		_ => panic!(msg!(BAD_STATE))
	}
}

/// Mark the global [`Player`] instance as available for referencing.
///
/// **Safety**: The instance data must be initialized and unreferenced.
unsafe fn unlock ()
{
	INSTANCE.state.store(state::AVAILABLE, Ordering::Release);
}

/// Drop the global [`Player`] instance if it exists.
///
/// **Panics** if the players is currently locked.
pub fn reset ()
{
	match INSTANCE.state.swap(state::LOCKED, Ordering::Acquire) {
		state::UNINIT => return,
		state::AVAILABLE => {
			unsafe{(&mut*INSTANCE.data.get()).assume_init_drop()};
			INSTANCE.state.store(state::UNINIT, Ordering::Release);
		}
		state::LOCKED => panic!(msg!(ACQ_LOCKED)),
		_ => panic!(msg!(BAD_STATE))
	}
}


/// Threadsafe exclusive access to the global [`Player`] instance.
///
/// Obtained by [locking](lock) the player, unlocks it when dropped.
pub struct LockGuard (std::marker::PhantomData<Player>);
impl std::ops::Drop for LockGuard
{
	fn drop(&mut self) {unsafe{unlock()}}
}
impl std::ops::Deref for LockGuard
{
	type Target = Player;

	fn deref(&self) -> &Player
	{
		unsafe{(&mut *INSTANCE.data.get()).assume_init_ref()}
	}
}
impl std::ops::DerefMut for LockGuard
{
	fn deref_mut(&mut self) -> &mut Player
	{
		unsafe{(&mut *INSTANCE.data.get()).assume_init_mut()}
	}
}

} // mod instance

pub use instance::{lock, LockGuard};


//////
//
// Classes
//

/// Smart pointer with unique ownership used to store [`Component`] trait objects owned by the player.
///
/// The tag indicates whether the component is active (0) or not (1).
type CompPtr<T> = cgv_util::Tagged<Box<T>, 1>;

/// Container storing a specific kind of [`Component`] as boxed trait objects.
///
/// Entries can be individually marked as inactive, and independently one may be selected as "main". The meaning of
/// these designations, if any, depends on the kind of component.
pub struct Components<Comp: DynComponent + ?Sized>
{
	slots: Vec<Option<CompPtr<Comp>>>,
	pub(self) main: usize,
}
impl<Comp: DynComponent + ?Sized> Components<Comp>
{
	const EMPTY: Self = Self{slots: Vec::new(), main: 0};

	const MSG_BAD_HANDLE: &'static str = "Invalid handle: The requested object no longer exists.";
	const MSG_MISSING_OBJ: &'static str
		= "Invalid handle: The requested object no longer exists or is already borrowed.";
	const MSG_WRONG_TYPE: &'static str = "Invalid handle: The requested object is not of the expected type.";

	/// Borrow the [`Component`] identified by the given handle and downcast to `T`.
	///
	/// **Panics** if the requested object no longer exists, is borrowed already, or not of type `T`.
	pub fn get<T: Component> (&self, handle: Handle<Comp>) -> &T
	{
		<dyn Any>::downcast_ref::<T>(
			self.slots.get(handle.index()).expect(Self::MSG_BAD_HANDLE)
			.as_deref().expect(Self::MSG_MISSING_OBJ)
			.as_ref()
		).expect(Self::MSG_WRONG_TYPE)
	}

	/// Mutably borrow the [`Component`] identified by the given handle and downcast to `T`.
	///
	/// **Panics** if the requested object no longer exists, is borrowed already, or not of type `T`.
	pub fn get_mut<T: Component> (&mut self, handle: Handle<Comp>) -> &mut T
	{
		<dyn Any>::downcast_mut::<T>(
			self.slots.get_mut(handle.index()).expect(Self::MSG_BAD_HANDLE)
			.as_deref_mut().expect(Self::MSG_MISSING_OBJ)
			.as_mut()
		).expect(Self::MSG_WRONG_TYPE)
	}

	/// Borrow the current main component if there is one.
	fn main (&self) -> Option<&Comp>
	{
		self.slots.get(self.main)?.as_deref()
	}

	/// Mutably borrow the current main component if there is one.
	fn main_mut (&mut self) -> Option<&mut Comp>
	{
		self.slots.get_mut(self.main)?.as_deref_mut()
	}

	/// If there is a main component, move it out of the container.
	///
	/// This allows calling a method of the component with a reference to the player, since they no longer alias.
	/// Make sure to reinsert the component afterwards using [`Self::putMain`].
	pub(self) fn takeMain (&mut self) -> Option<CompPtr<Comp>>
	{
		self.slots.get_mut(self.main)?.take()
	}

	/// Store the given component in the slot selected as main, dropping any previous value.
	///
	/// Should generally be used only to undo [`Self::takeMain`]. For seleting a different main component, set
	/// [`Self::main`] instead.
	pub(self) fn putMain (&mut self, new_actor: CompPtr<Comp>)
	{
		self.slots[self.main] = Some(new_actor);
	}
}

/// Identifies a [`Component`] of type `Comp` stored by the [`Player`].
///
/// Provides access to that component via [`Components::get`] and [`Components::get_mut`]. Handles remain valid as long
/// as the player owns the referenced component; in particular, they can be used in asynchronous callbacks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Handle<Comp: ?Sized>
{
	idx: u16,
	marker: std::marker::PhantomData<fn() -> Comp>,
}
impl<Comp: ?Sized> Handle<Comp>
{
	fn new (index: usize) -> Self
	{
		debug_assert!(index <= u16::MAX as usize);
		Self{idx: index as u16, marker: std::marker::PhantomData}
	}

	pub(crate) fn index(self) -> usize {self.idx as usize}
}
pub type AppHandle = Handle<dyn Application>;
pub type CameraHandle = Handle<dyn Camera>;
pub type CamIntHandle = Handle<dyn CameraInteractor>;

////
// Player

/// The central application host class.
pub struct Player
{
	pub camera: Box<dyn Camera>,
	pub applications: Components<dyn Application>,
	pub cameraInteractors: Components<dyn CameraInteractor>,
	pub state: State,
}

/// Global event loop and rendering resources.
pub struct State
{
	pub egui: egui::Context,
	pub context: Context,

	pub renderSetup: RenderSetup,
	pub(crate) defaultClearColor: egui::Color32,
	prevFramebufferResolution: glm::UVec2,

	quitShortcut: egui::KeyboardShortcut,
	activeSidePanel: usize,

	viewportCompositor: ViewportCompositor,

	pendingRedraw: bool,
	continuousRedrawRequests: u32,
	userInstantRedraw: bool,

	startInstant: time::Instant,
	prevFrameElapsed: time::Duration,
	prevFrameDuration: time::Duration
}

/// Implicitly access the [`state`](Self::state) subobject for convenience.
///
/// Note that this borrows the entire player, in case of aliasing issues you must instead use the field explicitly.
impl std::ops::Deref for Player
{
	type Target = State;
	fn deref (&self) -> &State {&self.state}
}
/// Implicitly access the [`state`](Self::state) subobject for convenience.
///
/// Note that this borrows the entire player, in case of aliasing issues you must instead use the field explicitly.
impl std::ops::DerefMut for Player
{
	fn deref_mut (&mut self) -> &mut State {&mut self.state}
}

impl Player
{
	pub fn new (
		applicationFactory: Box<dyn ApplicationFactory>,
		cc: &eframe::CreationContext,
		environment: run::Environment
	) -> Result<Self>
	{
		// Log player initialization start
		tracing::info!("Initializing Player...");

		// Get necessary context handles from eframe
		if cc.wgpu_render_state.is_none() {
			return Err(anyhow!("eframe is not configured to use the WGPU backend"));
		}
		let eguiRs = cc.wgpu_render_state.as_ref().unwrap();

		// Adjust GUI styling to our own CGV-rs defaults
		font::replaceDefaults(&cc.egui_ctx);
		cc.egui_ctx.all_styles_mut(|style|
		{
			// Slightly smaller font overall
			for (_, fontId) in style.text_styles.iter_mut() {
				fontId.size *= 0.984375; // ToDo: consider 0.96875
			}

			// Significantly smaller window headers
			let em = style.text_styles[&egui::TextStyle::Body].size;
			style.text_styles.get_mut(&egui::TextStyle::Heading).map(|font| font.size = 1.25*em);
		});

		// On WASM, increase the double click distance to make it easier to use double-taps on mobile devices
		#[cfg(target_arch="wasm32")]
		cc.egui_ctx.options_mut(|options| options.input_options.max_click_dist *= 5.);

		// Create context
		let context = Context::new(eguiRs);

		// Log render setup
		let defaultClearColor = egui::Rgba::from_rgb(0.0707, 0.217, 0.457);
		let renderSetup = RenderSetup::new(
			&context, eguiRs.target_format, eguiRs.target_format, hal::DepthStencilFormat::D32,
			wgpu::Color{
				r: defaultClearColor.r() as f64, g: defaultClearColor.g() as f64,
				b: defaultClearColor.b() as f64, a: defaultClearColor.a() as f64
			},
			1., wgpu::CompareFunction::Less
		);
		let defaultClearColor = defaultClearColor.into();

		// Create stateful rendering components
		let camera = Box::new(view::CameraObject::from(view::MonoCamera::new(
			&context, &renderSetup, glm::vec2(2, 2), renderSetup.defaultColorFormat(),
			renderSetup.defaultDepthStencilFormat().into(), Some("MonoCamera0")
		)));
		let viewportCompositor = ViewportCompositor::new(
			&context, &renderSetup, camera.framebuffer().color0(), Some("CGV__MainViewportCompositor")
		)?;

		// Now construct
		let mut player = Self {
			camera,
			cameraInteractors: Components {
				slots: vec![
					Some(CompPtr::fromSafe(Box::new(view::CamIntObject::from(view::OrbitInteractor::new())), 1)),
					Some(CompPtr::fromSafe(Box::new(view::CamIntObject::from(view::WASDInteractor::new())), 1)),
				],
				main: 0,
			},
			applications: Components::EMPTY,
			state: State {
				quitShortcut: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape),
				egui: cc.egui_ctx.clone(),
				activeSidePanel: 0,

				context,
				renderSetup,
				defaultClearColor,

				prevFramebufferResolution: glm::vec2(0u32, 0u32),

				viewportCompositor,

				pendingRedraw: false,
				continuousRedrawRequests: 0,
				userInstantRedraw: false,

				startInstant: time::Instant::now(),
				prevFrameElapsed: time::Duration::from_secs(0),
				prevFrameDuration: time::Duration::from_secs(0),
			}
		};

		// Init application(s)
		let mut mainApplication = applicationFactory.create(
			&player.state.context, &player.state.renderSetup, environment
		)?;
		mainApplication.preInit(&mut player)?;
		mainApplication.recreatePipelines(
			&player.context, &player.renderSetup, &Player::globalPassesFromCameras(player.activeCameras())
		);
		mainApplication.postInit(&mut player)?;
		player.applications.slots.push(Some(CompPtr::fromSafe(mainApplication, 0)));
		player.activeSidePanel = 2;

		// Done!
		tracing::info!("Startup complete.");
		Ok(player)
	}

	#[cfg(not(target_arch="wasm32"))]
	pub fn run (applicationFactory: Box<dyn ApplicationFactory>) -> Result<()>
	{
		// Log that we have begun the startup process
		tracing::info!("Starting up...");
		tracing::info!("Platform: {}", util::meta::platformTargetTriple());

		// Set up environment
		let environment = {
			let path = util::meta::currentExeDir().join("ENVIRONMENT.yaml");
			match fs::read(path)
			{
				Ok(bytes) => run::Environment::deserialize(&bytes).unwrap_or_else(|e| {
					tracing::warn!("Failed to read ENVIRONMENT.yaml file: {}", e);
					run::Environment::default()
				}),

				Err(e) => {
					if e.kind() != std::io::ErrorKind::NotFound {
						tracing::warn!("Failed to read ENVIRONMENT.yaml file: {}", e);
					}
					run::Environment::default()
				}
			}
		};

		// Prepare default player icon
		let icon = image::load_from_memory(util::sourceBytes!("/res/ico/defaultIcon.png"))?;

		// Setup WGPU native
		let options = eframe::NativeOptions {
			viewport: egui::ViewportBuilder::default()
				.with_inner_size([1216., 800.])
				.with_icon(egui::viewport::IconData {
					rgba: icon.as_bytes().to_owned(), width: icon.width(), height: icon.height()
				}),
			vsync: false,
			multisampling: 0,
			//depth_buffer: 0,
			//stencil_buffer: 0,
			hardware_acceleration: eframe::HardwareAcceleration::Required,
			renderer: eframe::Renderer::Wgpu,
			//run_and_return: false,
			#[allow(unused_variables)] // in Windows builds, we're not using `elBuilder` in the next line
			event_loop_builder: Some(Box::new(|elBuilder| {
				// Conditional code for the two supported display protocols on *nix. Wayland takes precedence in case
				// both protocols are enabled.
				#[cfg(all(not(target_os="windows"),not(target_os="macos")))] {
					// - Wayland (either just Wayland or both)
					#[cfg(all(feature="wayland"))]
						elBuilder.with_wayland();
					// - just X11
					#[cfg(all(feature="x11",not(feature="wayland")))]
						elBuilder.with_x11();
					// - neither - invalid configuration!
					#[cfg(all(not(feature="wayland"),not(feature="x11")))]
						compile_error!("Must enable one of `x11` or `wayland` for Unix builds!");
				}
			})),
			//centered: false,
			wgpu_options: egui_wgpu::WgpuConfiguration {
				//present_mode: Default::default(),
				//desired_maximum_frame_latency: None,
				//native_adapter_selector: None,
				//trace_path: None,
				wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(egui_wgpu::WgpuSetupCreateNew {
					instance_descriptor: wgpu::InstanceDescriptor {
						#[cfg(all(not(target_os="windows"),not(target_os="macos")))]
							backends: wgpu::Backends::VULKAN,
						#[cfg(target_os="windows")]
							backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN,
						#[cfg(target_os="macos")]
							backends: wgpu::Backends::METAL,
						#[cfg(debug_assertions)]
							flags: wgpu::InstanceFlags::debugging(),
						#[cfg(not(debug_assertions))]
							flags: wgpu::InstanceFlags::empty(),
						backend_options: wgpu::BackendOptions::from_env_or_default(),
						memory_budget_thresholds: Default::default(),
						display: None
					},
					native_adapter_selector: None,
					power_preference: wgpu::PowerPreference::HighPerformance,
					device_descriptor: Arc::new(|_| wgpu::DeviceDescriptor {
						label: Some("CGV__WgpuDevice"),
						experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
						  required_features: wgpu::Features::INDIRECT_FIRST_INSTANCE
						| wgpu::Features::PASSTHROUGH_SHADERS,
						//required_limits: Default::default(),
						//memory_hints: Default::default(),
						..Default::default()
					}),
					display_handle: None
				}),
				..Default::default()
			},
			//persist_window: false,
			//persistence_path: None,
			dithering: true,
			..Default::default()
		};

		// Run and report result
		match eframe::run_native(
			"CGV-rs Player", options, Box::new(move |cc| {
				instance::set(Player::new(applicationFactory, cc, environment)?);
				Ok(Box::new(StaticImpls))
			})
		){
			Ok(_) => {
				tracing::info!("Shutdown complete.");
				Ok(())
			},
			Err(error) => Err(anyhow::anyhow!("{:?}", error))
		}
	}

	#[cfg(target_arch="wasm32")]
	pub fn run (applicationFactory: Box<dyn ApplicationFactory>) -> Result<()>
	{
		// In case of WASM, make sure the JavaScript console is set up for receiving log messages first thing (for non-
		// WASM targets, tracing/logging is already being set up at module loading time)
		initTracing();

		// Log that we have begun the startup process
		tracing::info!("Starting up...");
		tracing::info!("Platform: {}", util::meta::platformTargetTriple().full());

		// Setup WebGPU
		let webOptions = eframe::WebOptions {
			//depth_buffer: 0,
			wgpu_options: egui_wgpu::WgpuConfiguration {
				//present_mode: Default::default(),
				//desired_maximum_frame_latency: None,
				wgpu_setup: egui_wgpu::WgpuSetup::CreateNew( egui_wgpu::WgpuSetupCreateNew {
					instance_descriptor: wgpu::InstanceDescriptor {
						backends: wgpu::Backends::BROWSER_WEBGPU,
						#[cfg(debug_assertions)]
							flags: wgpu::InstanceFlags::debugging(),
						#[cfg(not(debug_assertions))]
							flags: wgpu::InstanceFlags::empty(),
						backend_options: wgpu::BackendOptions::from_env_or_default(),
						memory_budget_thresholds: Default::default(),
						display: None
					},
					native_adapter_selector: None,
					power_preference: wgpu::PowerPreference::HighPerformance,
					device_descriptor: Arc::new(|_| wgpu::DeviceDescriptor {
						label: Some("CGV__WgpuDevice"),
						//required_features: Default::default(),
						//required_limits: Default::default(),
						//memory_hints: Default::default(),
						..Default::default()
					}),
					display_handle: None
				}),
				..Default::default()
			},
			dithering: true,
			..Default::default()
		};

		// Dispatch the main loop
		use eframe::wasm_bindgen::JsCast as _;
		use eframe::web_sys;
		wasm_bindgen_futures::spawn_local(async move {
			let document = web_sys::window()
				.expect("No window")
				.document()
				.expect("No document");

			let canvas = document
				.get_element_by_id("cgvRsCanvas")
				.expect("Failed to find target canvas with id=`cgvRsCanvas`!")
				.dyn_into::<web_sys::HtmlCanvasElement>()
				.expect("Element with id=`cgvRsCanvas` was not a HtmlCanvasElement!");

			let startResult = eframe::WebRunner::new()
				.start(
					canvas,
					webOptions,
					Box::new(|cc| {
						instance::set(Player::new(applicationFactory, cc, run::Environment::default())?);
						Ok(Box::new(StaticImpls))
					})
				)
				.await;

			// Remove the loading text and spinner:
			match startResult {
				Ok(_) => if let Some(loadingIndicator) =
					document.get_element_by_id("cgvLoadingIndicator") { loadingIndicator.remove(); },
				Err(error) => {
					let msgDetail = if let Some(errorDesc) = error.as_string()
					{ errorDesc }
					else
					{ format!("{:?}", error) };
					if let Some(loadingIndicator) =
						document.get_element_by_id("cgvLoadingIndicator") {
						let msg = format!(
							"<p>The CGV-rs Player has crashed.<br/>Reason: {:?}</p><p>See the developer console for details. </p>", msgDetail
						);
						loadingIndicator.set_inner_html(msg.as_str());
					}
					panic!("FATAL: failed to start CGV-rs Player:\n{msgDetail}");
				}
			};
		});

		// Done
		Ok(())
	}

	fn prepareEvents (
		&self, inputState: &egui::InputState, viewportResponse: &egui::Response, menubarResponse: &egui::Response,
		sidepanelResponse: &egui::Response, highDpiScaleFactor: f32
	) -> Vec<InputEvent>
	{
		// Pre-allocate event list
		let mut preparedEvents = Vec::with_capacity(4); // <-- heuristically chosen

		// Mouse wheel
		let (didntPinch, zoom, amount)
		 = if     viewportResponse.contains_pointer()
		      && (inputState.smooth_scroll_delta.x != 0. || inputState.smooth_scroll_delta.y != 0.)
		{
			(true, true, glm::vec2(inputState.smooth_scroll_delta.x, inputState.smooth_scroll_delta.y))
		}
		else
		{
			// Try pinch zoom next
			let zoomDelta = inputState.zoom_delta_2d();
			let forwardPinch =    viewportResponse.contains_pointer() || menubarResponse.contains_pointer()
			                        || sidepanelResponse.contains_pointer();
			if forwardPinch && (zoomDelta.x != 1. || zoomDelta.y != 1.) {
				const PINCH_SENSITIVITY: f32 = 20./3.;
				(false, true, glm::vec2(
					PINCH_SENSITIVITY*if zoomDelta.x > 1. { zoomDelta.x } else { -1./zoomDelta.x },
					PINCH_SENSITIVITY*if zoomDelta.y > 1. { zoomDelta.y } else { -1./zoomDelta.y }
				))
			}
			else {
				// No zooming action at all this frame
				(true, false, glm::Vec2::zeros())
			}
		};
		if zoom {
			preparedEvents.push(InputEvent::MouseWheel(MouseWheelInfo { amount, modifiers: inputState.modifiers }));
		}

		// Dragging action
		if didntPinch && viewportResponse.dragged()
		{
			let dm = viewportResponse.drag_motion();
			if dm.length_sq() > 0.
			{
				preparedEvents.push(InputEvent::Dragged(DragInfo {
					buttons: {
						let p = &inputState.pointer; [
							p.primary_down(), p.secondary_down(), p.middle_down(),
							p.button_down(egui::PointerButton::Extra1), p.button_down(egui::PointerButton::Extra2)
						]
					},
					modifiers: inputState.modifiers,
					direction: glm::vec2(dm.x, dm.y)
				}));
			}
		}

		// Clicks
		let pointerPos = viewportResponse.interact_pointer_pos().map(|pos_egui| {
			let pos_egui = (pos_egui-viewportResponse.rect.min) * highDpiScaleFactor;
			glm::vec2(pos_egui.x as u32, pos_egui.y as u32)
		});
		if let Some(pointerPos) = pointerPos
		{
			for button in [
				egui::PointerButton::Primary, egui::PointerButton::Secondary, egui::PointerButton::Middle,
				egui::PointerButton::Extra1, egui::PointerButton::Extra2
			]{
				let clickInfo = ClickInfo {
					button, modifiers: inputState.modifiers, position: pointerPos
				};
				if viewportResponse.triple_clicked_by(button) {
					preparedEvents.push(InputEvent::TripleClick(clickInfo));
				}
				else if viewportResponse.double_clicked_by(button) {
					preparedEvents.push(InputEvent::DoubleClick(clickInfo));
				}
				else if viewportResponse.clicked_by(button) {
					preparedEvents.push(InputEvent::Click(clickInfo));
				}
			}
		}

		// Report result
		preparedEvents
	}

	fn dispatchTranslatedEvent (&mut self, event: &InputEvent) -> bool
	{
		// Keep track of whether a full scene redraw is needed
		let mut redraw = false;

		// Applications get first dibs
		// - the main (foreground) application
		if let Some(mut app) = self.applications.takeMain() {
			let outcome = app.input(&event, self, AppHandle::new(self.applications.main));
			self.applications.putMain(app);

			match outcome {
				// Event was closed by the receiver
				EventOutcome::HandledExclusively(redrawRequested) => return redrawRequested,

				// Event was acted upon but others may react to it too
				EventOutcome::HandledDontClose(redrawRequested) => redraw |= redrawRequested,

				// Event was ignored
				EventOutcome::NotHandled => {}
			}
		}

		// - now all active background applications in some undefined order.
		for idx in 0..self.applications.slots.len() {
			if idx == self.applications.main {continue};
			let Some(mut app) = self.applications.slots[idx].take_if(|ptr| ptr.tag() == 0) else {continue};
			let outcome = app.input(&event, self, AppHandle::new(idx));
			self.applications.slots[idx] = Some(app);

			match outcome {
				// Event was closed by the receiver
				EventOutcome::HandledExclusively(redrawRequested) => return redrawRequested,

				// Event was acted upon but others may react to it too
				EventOutcome::HandledDontClose(redrawRequested) => redraw |= redrawRequested,

				// Event was ignored
				EventOutcome::NotHandled => {}
			}
		}

		// Finally, the main camera interactor
		if let Some(mut ci) = self.cameraInteractors.takeMain() {
			let outcome = ci.input(&event, self, CamIntHandle::new(self.cameraInteractors.main));
			self.cameraInteractors.putMain(ci);

			match outcome {
				// Event was handled
				  EventOutcome::HandledExclusively(redrawRequested)
				| EventOutcome::HandledDontClose(redrawRequested) => redraw |= redrawRequested,

				// Event was ignored
				EventOutcome::NotHandled => {}
			}
		}

		redraw
	}

	fn dispatchEvents (&mut self, events: &[egui::Event], complexEvents: &[InputEvent]) -> bool
	{
		// Gather key events
		let translatedEvents =  events.iter().filter_map(|event| {
			match event
			{
				&egui::Event::Key { key, /*physical_key, */pressed, repeat, modifiers, .. }
				=> Some(InputEvent::Key(KeyInfo { key, pressed, repeat, modifiers })),

				_ => None
			}
		});

		// Gather mouse events
		/* t.b.d. */

		// Dispatch events
		let mut redraw = false;
		translatedEvents.for_each(|ref event| redraw |= self.dispatchTranslatedEvent(event));
		complexEvents.iter().for_each(|event| redraw |= self.dispatchTranslatedEvent(event));
		redraw
	}

	/// Performs the logic for preparing applications for rendering the scene.
	fn prepare (
		&mut self, _: &wgpu::Device, _: &wgpu::Queue, _: &mut wgpu::CommandEncoder
	) -> Vec<wgpu::CommandBuffer>
	{
		// Make all global passes needed by the main camera
		let mut cmdBuffers = Vec::with_capacity(8);
		let cameraName = self.camera.name();
		let globalPasses = Self::globalPassesFromCameras(activeCameras!(self));
		for passNr in 0..globalPasses.info.len()
		{
			// Get actual pass information
			let passInfo = &globalPasses.info[passNr];
			let renderState = &globalPasses.renderStates[passInfo.index];
			tracing::debug!("Camera[{cameraName:?}]: Preparing global pass #{passNr} ({:?})", passInfo.pass);

			// Update managed render state
			// Uniforms
			// - viewing
			renderState.viewingUniforms.upload(&self.context);

			// Prepare the main application (if any)
			if let Some(application) = self.applications.main_mut() {
				if let Some(newCommands) = application.prepareFrame(
					&self.state.context, renderState, &passInfo
				){
					cmdBuffers.extend(newCommands);
				}
			}

			// Prepare other active applications
			self.applications.slots.iter_mut().filter_map(|slot| slot.as_mut().take_if(|ptr| ptr.tag() == 0)).fold(
				&mut cmdBuffers, |commands, app| {
					if let Some(newCommands) = app.prepareFrame(
						&self.state.context, renderState, &passInfo
					){
						commands.extend(newCommands);
					}
					commands
				}
			);
		}

		// Done!
		cmdBuffers
	}

	/// Performs the logic for letting applications render their contribution to the scene.
	fn redraw (
		&mut self, _: &wgpu::Device, _: &wgpu::Queue, _: &mut wgpu::CommandEncoder
	) -> Vec<wgpu::CommandBuffer>
	{
		// Make all global passes needed by the main camera
		let mut cmdBuffers = Vec::with_capacity(8);
		let mut cmdEncoder = self.context.device().create_command_encoder(&Default::default());
		let cameraName = self.camera.name();
		let globalPasses = Self::globalPassesFromCameras(activeCameras!(self));
		for passNr in 0..globalPasses.info.len()
		{
			// Get actual pass information
			let passInfo = &globalPasses.info[passNr];
			let renderState = &globalPasses.renderStates[passInfo.index];

			// Update managed render state
			// Uniforms
			// - viewing
			renderState.viewingUniforms.upload(&self.context);

			// Create the managed render pass for this global pass
			let desc = wgpu::RenderPassDescriptor {
				label: Some("CGV__ManagedSceneRenderPass"),
				color_attachments: &[
					renderState.getMainColorAttachment(Some(&passInfo.clearColor)),
				],
				depth_stencil_attachment: renderState.getMainDepthStencilAttachment(Some(passInfo.depthClearValue)),
				occlusion_query_set: None,
				timestamp_writes: None,
				multiview_mask: None
			};
			let mut renderPass = cmdEncoder.begin_render_pass(&desc);

			// Render the main application (if any)
			if let Some(application) = self.applications.main_mut() {
				application.render(&self.state.context, renderState, &mut renderPass, &passInfo);
			}

			// Render other active applications
			for idx in 0..self.applications.slots.len() {
				if idx == self.applications.main {continue};
				let Some(mut app) = self.applications.slots[idx].take_if(|ptr| ptr.tag() == 0) else {continue};
				app.render(&self.context, renderState, &mut renderPass, &passInfo);
				self.applications.slots[idx] = Some(app);
			}

			if let Some(mut callback) = passInfo.completionCallback.take() {
				callback(&self.context, passNr as u32);
				passInfo.completionCallback.set(Some(callback));
			}

			// Finish the pass
			tracing::debug!("Camera[{:?}]: Global pass #{passNr} ({:?}) done", cameraName, passInfo.pass);
		}

		// Done!
		cmdBuffers.push(cmdEncoder.finish());
		cmdBuffers
	}

	/// Obtain a list of all currently active cameras (i.e. those that will contribute one or more global passes).
	pub fn activeCameras (&self) -> &[Box<dyn Camera>] {
		activeCameras!(self)
	}

	/// Obtain information about all global render passes that the player will currently dispatch.
	pub fn activeGlobalPasses (&self) -> GlobalPasses<'_> {
		Self::globalPassesFromCameras(activeCameras!(self))
	}

	/// Internal helper to help the borrow checker disentangle disjunct borrows into `self` from borrows of the cameras.
	fn globalPassesFromCameras (activeCameras: &[Box<dyn Camera>]) -> GlobalPasses<'_> {
		// TODO: What about other active cameras?
		activeCameras[0].globalPasses()
	}

	pub fn postRecreatePipelines (&mut self) {
		for i in 0..self.applications.slots.len() {
			let Some(mut app) = self.applications.slots[i].take_if(|ptr| ptr.tag() == 0 || i == self.applications.main)
				else {continue};
			app.recreatePipelines(&self.context, &self.renderSetup, &self.activeGlobalPasses());
			self.applications.slots[i] = Some(app);
		}
	}

	pub fn getDepthAtSurfacePixel_async<Closure: FnOnce(Option<f32>) + wgpu::WasmNotSend + 'static> (
		&self, pixelCoords: glm::UVec2, callback: Closure
	){
		if let Some(dispatcher) = self.camera.getDepthReadbackDispatcher(pixelCoords) {
			dispatcher.getDepthValue_async(&self.context, |depth| {
				callback(Some(depth));
			})
		}
		else {
			callback(None)
		}
	}

	pub fn unprojectPointAtSurfacePixelH_async<Closure: FnOnce(Option<&glm::Vec4>) + wgpu::WasmNotSend + 'static> (
		&self, pixelCoords: glm::UVec2, callback: Closure
	){
		if let Some(dispatcher) = self.camera.getDepthReadbackDispatcher(pixelCoords) {
			dispatcher.unprojectPointH_async(&self.context, |point| {
				callback(point);
			})
		}
		else {
			callback(None)
		}
	}

	pub fn unprojectPointAtSurfacePixel_async<Closure: FnOnce(Option<&glm::Vec3>) + wgpu::WasmNotSend + 'static> (
		&self, pixelCoords: glm::UVec2, callback: Closure
	){
		if let Some(dispatcher) = self.camera.getDepthReadbackDispatcher(pixelCoords) {
			dispatcher.unprojectPoint_async(&self.context, |point| {
				callback(point);
			})
		}
		else {
			callback(None)
		}
	}
}

impl State
{
	pub fn pushContinuousRedrawRequest (&mut self)
	{
		if self.continuousRedrawRequests < 1
		{
			self.prevFrameElapsed = self.startInstant.elapsed();
			self.prevFrameDuration = time::Duration::from_secs(0);

			tracing::info!("Starting continuous redrawing");
			self.egui.request_repaint();
		}
		self.continuousRedrawRequests += 1;
	}

	pub fn dropContinuousRedrawRequest (&mut self)
	{
		if self.continuousRedrawRequests < 1 {
			panic!("logic error - more continuous redraw requests dropped than were pushed");
		}
		self.continuousRedrawRequests -= 1;
		if self.continuousRedrawRequests < 1
		{
			self.prevFrameDuration = time::Duration::from_secs(0);
			tracing::info!("Stopping continuous redrawing");
		}
	}

	/// **TODO:** Highlight differences between this, [`Self::postFullRedraw`] and [`Self::postGuiRedraw`].
	pub fn requireSceneRedraw (&mut self) {
		self.pendingRedraw = true;
	}

	/// **TODO:** Highlight differences between this, [`Self::requireSceneRedraw`] and [`Self::postFullRedraw`].
	pub fn postGuiRedraw (&mut self) {
		if self.continuousRedrawRequests < 1 {
			// Tell Egui to start drawing immediately
			self.egui.request_repaint();
		}
	}

	/// **TODO:** Highlight differences between this, [`Self::requireSceneRedraw`] and [`Self::postGuiRedraw`].
	pub fn postFullRedraw (&mut self)
	{
		if self.continuousRedrawRequests < 1 {
			// Make sure the cameras are redrawn also (otherwise just the GUI might get redrawn)
			self.requireSceneRedraw();

			// Tell Egui to start drawing immediately
			self.egui.request_repaint();
		}
	}

	///
	pub fn lastFrameTime (&self) -> f32 {
		self.prevFrameDuration.as_secs_f32()
	}

	///
	pub fn exit (&self, eguiContext: &egui::Context) {
		tracing::info!("Exiting...");
		eguiContext.send_viewport_cmd(egui::ViewportCommand::Close);
	}
}


/// Implements traits for the global player instance using `'static` functions.
struct StaticImpls;

impl eframe::App for StaticImpls
{
	fn on_exit(&mut self) {instance::reset()}

	fn ui (&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame)
	{
		let mut lock = lock();
		let player = &mut*lock;

		////
		// Main GUI

		// Draw the main menu bar
		let menubarResponse = ui::menuBar(player, ui);

		// Draw the side panel
		let sidepanelResponse = ui::sidepanel(player, ui);

		// Draw any free-floating UIs
		if let Some(mut app) = player.applications.takeMain() {
			app.freeUi(ui, player);
			player.applications.putMain(app);
		}


		////
		// 3D viewport

		// Update viewport frame style
		let mut frame = egui::Frame::central_panel(&ui.ctx().global_style());
		frame.inner_margin = egui::Margin::ZERO;

		// Draw actual viewport panel
		egui::CentralPanel::default().frame(frame).show_inside(ui, |ui|
		{
			// Keep track of reasons to do a scene redraw
			let mut redrawScene = player.continuousRedrawRequests > 0;

			// Update framebuffer size
			let availableSpace_egui = ui.available_size();
			let pxlsPerPoint = ui.ctx().pixels_per_point();
			let fbResolution = {
				let pixelsEgui = (availableSpace_egui*pxlsPerPoint).ceil();
				glm::vec2(pixelsEgui.x as u32, pixelsEgui.y as u32)
			};
			if fbResolution != player.prevFramebufferResolution
				&& fbResolution.x > 0
				&& fbResolution.y > 0
			{
				player.camera.resize(&player.state.context, fbResolution);
				player.state.viewportCompositor.updateSource(
					&player.state.context, player.camera.framebuffer().color0());
				player.prevFramebufferResolution = fbResolution;
				tracing::info!("Main framebuffer resized to {:?}", fbResolution);
				redrawScene = true; // we'll need to redraw the scene in addition to the UI
			}
			let (rect, response) =
				ui.allocate_exact_size(availableSpace_egui, egui::Sense::click_and_drag());

			/* Route input events */ {
				// TODO: Clone may be expensive, but we have use the state outside the callback to avoid deadlocking on
				// the egui state.
				// Egui's documentation recommends calling `input` for every event you want to query, locking and
				// unlocking the context every time, which is probably faster?
				// The third alternative would be to copy only some parts of the input state into a custom type.
				let inputState = ui.input(|state| state.clone());
				let complexEvents = player.prepareEvents(
					&inputState, &response, &menubarResponse, &sidepanelResponse, pxlsPerPoint
				);
				redrawScene |= player.dispatchEvents(&inputState.events, &complexEvents);
			}

			// If nobody else did, consume the global [ESC] quit shortcut
			if   (   response.contains_pointer() || menubarResponse.contains_pointer()
			      || sidepanelResponse.contains_pointer())
			   && ui.input_mut(|i| i.consume_shortcut(&player.quitShortcut))
			{
				player.exit(ui.ctx());
			}

			// Update camera interactor
			if let Some(mut ci) = player.cameraInteractors.takeMain() {
				ci.update(player, CamIntHandle::new(player.cameraInteractors.main));
				player.cameraInteractors.putMain(ci);
			}
			if player.camera.update() {
				redrawScene = true;
			}

			// Schedule compositing of the scene view onto the eframe center panel.
			player.pendingRedraw |= redrawScene;
			ui.painter().add(egui_wgpu::Callback::new_paint_callback(rect, StaticImpls));
		});
	}
}

impl egui_wgpu::CallbackTrait for StaticImpls
{
	fn prepare (
		&self, device: &wgpu::Device, queue: &wgpu::Queue, _: &egui_wgpu::ScreenDescriptor,
		eguiEncoder: &mut wgpu::CommandEncoder, _: &mut egui_wgpu::CallbackResources
	) -> Vec<wgpu::CommandBuffer>
	{
		let mut player = lock();

		// Only prepare the scene if requested
		if !player.pendingRedraw {return Vec::new()}

		tracing::debug!("Redrawing");
		player.prepare(device, queue, eguiEncoder)
	}

	fn finish_prepare (
		&self, device: &wgpu::Device, queue: &wgpu::Queue, eguiEncoder: &mut wgpu::CommandEncoder,
		_: &mut egui_wgpu::CallbackResources
	) -> Vec<wgpu::CommandBuffer>
	{
		let mut player = lock();

		// Only redraw the scene if requested
		if !player.pendingRedraw {return Vec::new()}

		// Actually redraw the scene
		player.pendingRedraw = false;
		player.redraw(device, queue, eguiEncoder)
	}

	fn paint (
		&self, _: epaint::PaintCallbackInfo, eguiRenderPass: &mut wgpu::RenderPass<'static>,
		_: &egui_wgpu::CallbackResources
	) {
		let mut player = lock();

		// Composite current view of the scene onto egui viewport
		player.viewportCompositor.composit(eguiRenderPass);

		// Update frame stats
		if player.continuousRedrawRequests == 0 {return}

		let elapsed = player.startInstant.elapsed();
		player.prevFrameDuration = elapsed - player.prevFrameElapsed;
		player.prevFrameElapsed = elapsed;
		player.egui.request_repaint();
	}
}
