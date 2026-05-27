
//////
//
// Imports
//

// Egui library and framework
use egui;
use egui_tiles as etiles;

// Local imports
use crate::{*, player::*};



//////
//
// Structs
//

/// A view of the scene as composed by all active [applications](Application).
pub(crate) struct SceneView {
	#[expect(dead_code)]
	renderSetup: RenderSetup,
	prevFramebufferResolution: glm::UVec2
}

/// An [application](Application)'s [main GUI](Application::ui).
#[expect(dead_code)]
pub(crate) struct AppGui;

///
impl etiles::Behavior<SceneView> for Player
{
	fn pane_ui (&mut self, ui: &mut egui::Ui, _: etiles::TileId, pane: &mut SceneView) -> etiles::UiResponse
	{
		// Keep track of reasons to do a scene redraw
		let mut redrawScene = self.continuousRedrawRequests > 0;

		// Update framebuffer size
		let availableSpace_egui = ui.available_size();
		let pxlsPerPoint = ui.ctx().pixels_per_point();
		let fbResolution = {
			let pixelsEgui = (availableSpace_egui*pxlsPerPoint).ceil();
			glm::vec2(pixelsEgui.x as u32, pixelsEgui.y as u32)
		};
		if fbResolution != pane.prevFramebufferResolution && fbResolution.x > 0 && fbResolution.y > 0
		{
			self.camera.resize(&self.state.context, fbResolution);
			self.state.viewportCompositor.updateSource(&self.state.context, self.camera.framebuffer().color0());
			pane.prevFramebufferResolution = fbResolution;
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
			let complexEvents = self.prepareEvents(&inputState, &response, pxlsPerPoint);
			redrawScene |= self.dispatchEvents(&inputState.events, &complexEvents);
		}

		// If nobody else did, consume the global [ESC] quit shortcut
		if   (   response.contains_pointer() || self.state.menubarResponse.as_ref().unwrap().contains_pointer()
		      || self.state.sidepanelResponse.as_ref().unwrap().contains_pointer())
		  && ui.input_mut(|i| i.consume_shortcut(&self.quitShortcut))
		{
			self.exit(ui.ctx());
		}

		// Update camera interactor
		if let Some(mut ci) = self.cameraInteractors.takeMain() {
			ci.update(self, Handle(self.cameraInteractors.main));
			self.cameraInteractors.putMain(ci);
		}
		if self.camera.update() {
			redrawScene = true;
		}

		// Schedule compositing of the scene view onto the eframe center panel.
		self.pendingRedraw |= redrawScene;
		ui.painter().add(egui_wgpu::Callback::new_paint_callback(rect, StaticImpls));

		// Done!
		egui_tiles::UiResponse::None
	}

	fn tab_title_for_pane(&mut self, _: &SceneView) -> egui::WidgetText {
		format!("View - {}", self.camera.name()).into()
	}
}
