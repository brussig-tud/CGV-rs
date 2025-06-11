
//////
//
// Imports
//

// Egui library and framework
use egui;

// Local imports
use crate::*;
use crate::view::CameraParameters;



///////
//
// Constants
//

// For consistent labeling of UI theme-related stuff
const LIGHT_ICON: &str = "☀"; // ToDo: consider 💡
const DARK_ICON: &str = "🌙";
const SYSTEM_ICON: &str = "💻";



//////
//
// Functions
//

/// Draw (and act upon) the [`crate::Player`] menu bar at the top of the main window.
pub(crate) fn menuBar (player: &mut Player, eguiContext: &egui::Context)
{
	egui::TopBottomPanel::top("menu_bar").show(eguiContext, |ui|
		egui::ScrollArea::horizontal().show(ui, |ui|
		{
			egui::menu::bar(ui, |ui|
			{
				// The global [ESC] quit shortcut
				let quit_shortcut = egui::KeyboardShortcut::new(
					egui::Modifiers::NONE, egui::Key::Escape
				);
				if ui.input_mut(|i| i.consume_shortcut(&quit_shortcut)) {
					player.exit(ui.ctx());
				}

				// Menu bar
				ui.menu_button("File", |ui| {
					ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
					#[cfg(not(target_arch="wasm32"))]
					if ui.add(
						egui::Button::new("Quit").shortcut_text(ui.ctx().format_shortcut(&quit_shortcut))
					).clicked() {
						player.exit(ui.ctx());
					}
					#[cfg(target_arch="wasm32")]
						ui.label("<nothing here>");
				});
				ui.separator();

				/* Dark/Light mode toggle */ {
					let menuIcon = if ui.ctx().style().visuals.dark_mode {DARK_ICON} else {LIGHT_ICON};
					ui.menu_button(menuIcon, |ui| {
						ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
						if ui.add(egui::Button::new(format!("{LIGHT_ICON} Light"))).clicked() {
							ui.ctx().set_theme(egui::ThemePreference::Light);
							ui.close_menu();
						}
						else if ui.add(egui::Button::new(format!("{DARK_ICON} Dark"))).clicked() {
							ui.ctx().set_theme(egui::ThemePreference::Dark);
							ui.close_menu();
						}
						else if ui.add(egui::Button::new(format!("{SYSTEM_ICON} System"))).clicked() {
							ui.ctx().set_theme(egui::ThemePreference::System);
							ui.close_menu();
						}
					});
				}
				ui.separator();

				// App switcher
				// - player settings
				if ui.selectable_label(player.activeSidePanel==0, "Player").clicked() {
					player.activeSidePanel = 0;
				}
				// - view settings
				if ui.selectable_label(player.activeSidePanel==1, "View").clicked() {
					player.activeSidePanel = 1;
				}
				// - applications
				/*for (idx, app) in player.applications.iter().enumerate()
				{
					if ui.selectable_label(player.activeSidePanel==0, "Camera").clicked() {
						player.activeSidePanel = 0;
					}
				}*/
				if ui.selectable_label(
					player.activeSidePanel==2, player.activeApplication.as_ref().unwrap().title()
				).clicked() {
					player.activeSidePanel = 2;
				}
			});
		})
	);
}

/// Draw (and act upon) the [`crate::Player`] side panel GUI.
pub(crate) fn sidepanel (player: &mut Player, eguiContext: &egui::Context)
{
	egui::SidePanel::right("CGV__sidePanel")
		.resizable(true)
		.default_width(320.)
		.show(eguiContext, |ui|
		{
			egui::ScrollArea::both().show(ui, |ui|
			{
				ui.horizontal(|ui|
				{
					ui.vertical(|ui|
					{
						match player.activeSidePanel
						{
							0 => self::player(player, ui),
							1 => self::view(player, ui),
							2 => {
								// Application UI
								ui.centered_and_justified(|ui| ui.heading(
									player.activeApplication.as_ref().unwrap().title()
								));
								ui.separator();
								let this = util::statify(player);
								player.activeApplication.as_mut().unwrap().ui(ui, this);
							},

							_ => {
								// We can only get here if there is a logic bug somewhere
								macro_rules! MSG {() => {"INTERNAL LOGIC ERROR: UI state corrupted!"};}
								tracing::error!(MSG!());
								unreachable!(MSG!());
							}
						}
					});
				});
				ui.allocate_space(ui.available_size());
			});
		});
}

/// Draw (and act upon) the side panel GUI for configuring and controlling [`crate::Player`] behavior.
pub(crate) fn player (player: &mut Player, ui: &mut egui::Ui)
{
	// Side panel header
	ui.centered_and_justified(|ui| ui.heading("▶ Player"));
	ui.separator();

	// Player control flow configuration
	egui::CollapsingHeader::new("Control flow")
		.id_salt("CGV__player_control")
		.default_open(true)
		.show(ui, |ui| gui::layout::ControlTableLayouter::new(ui).layout(
			ui, "CGV__player_control_layout", |ui|
			{
				ui.add("Instant redraw",|ui, _idealSize|
					ui.label(format!("{} requests", player.continousRedrawRequests))
				);
				ui.add("force:", |ui, _|
					if ui.add(gui::widget::toggle(&mut player.userInstantRedraw)).clicked() {
						if player.userInstantRedraw {
							player.pushContinuousRedrawRequest();
						}
						else {
							player.dropContinuousRedrawRequest();
						}
					}
				)
			}
		)
	);
}

/// Draw (and act upon) the side panel GUI for configuring and controlling the scene view, namely by manipulating.
/// [`view::Camera`]s and [`view::CameraInteractor`]s.
pub(crate) fn view (player: &mut Player, ui: &mut egui::Ui)
{
	// Side panel header
	ui.centered_and_justified(|ui| ui.heading("📷 View"));
	ui.separator();

	// Active camera and interactor selection
	gui::layout::ControlTableLayouter::new(ui).layout(ui, "CGV__CameraUi", |cameraUi|
	{
		// activeCameraInteractor
		cameraUi.add("Interactor", |ui, idealSize|
			egui::ComboBox::from_id_salt("CGV_view_inter")
				.selected_text(
					player.cameraInteractors[player.activeCameraInteractor].title()
				)
				.width(idealSize)
				.show_ui(ui, |ui|
					for (i, ci) in player.cameraInteractors.iter().enumerate() {
						ui.selectable_value(&mut player.activeCameraInteractor, i, ci.title());
					}
				)
		);

		// activeCamera
		let mut sel: usize = 0; // dummy, remove once camera management is done
		cameraUi.add("Active Camera", |ui, idealSize|
			egui::ComboBox::from_id_salt("CGV_view_act")
				.selected_text(player.camera.name())
				.width(idealSize)
				.show_ui(ui, |ui| ui.selectable_value(
					&mut sel, 0, player.camera.name()
				))
		);
	});

	// Settings from active camera and interactor
	ui.add_space(6.);
	egui::CollapsingHeader::new("Interactor settings")
		.id_salt("CGV_view_inter_s")
		.show(ui, |ui| player.cameraInteractors[player.activeCameraInteractor].ui(
			player.camera.as_mut(), ui
		));
	egui::CollapsingHeader::new("Active camera settings")
		.id_salt("CGV_view_act_s")
		.show(ui, |ui| CameraParameters::ui(player.camera.as_mut(), ui));
}
