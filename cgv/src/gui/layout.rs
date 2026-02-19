
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Egui library
use egui;

// Local imports
use crate::{gui, player::SIDEPANEL_SAFETY_MARGINS};



//////
//
// Module-wide constants
//

/// The default width (in logical units) of the right-hand side of a [`ControlTable`] layout.
pub const DEFAULT_RHS_WIDTH: f32 = 184.;



//////
//
// Structs
//

/// A smart 2-column grid layout that uses the assumption that it will contain a list of controls, with a descriptive
/// label on the left and the control on the right, to efficiently make use of the available space while maintaining a
/// clean and tidy look.
///
/// Widgets are provided via Closures taking the active *egui* [UI object](egui::Ui) as well as an `f32` indicating the
/// ideal width of any widget added to the grid cell (see example), which users are free to ignore.
///
/// Instances of the `ControlTable` layout can not be created directly, instead a [`ControlTableLayouter`] must be used
/// to take care of the creation which will then initiate the user-defined logic for adding `ControlTable` rows via a
/// provided closure.
pub struct ControlTable<'ui>
{
	/// Reference to the *egui* UI object the control table is adding its contents to.
	ui: &'ui mut egui::Ui,

	#[doc=include_str!("_doc/ControlTable_lhsWidth.md")]
	lhsWidth: f32,

	#[doc=include_str!("_doc/ControlTable_rhsWidth.md")]
	rhsWidth: f32
}
impl<'ui> ControlTable<'ui>
{
	/// Create the control table on the given *egui* [UI object](`egui::Ui`) with the given layouting parameters.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object the contents of the `ControlTable` should appear for.
	/// * `layouter` – The control table layouter providing the required layouting parameters
	pub(in crate::gui::layout) fn new (ui: &'ui mut egui::Ui, definer: &ControlTableLayouter) -> Self {
		Self { ui,   lhsWidth: definer.lhsWidth,   rhsWidth: definer.rhsWidth }
	}

	/// Add a labeled control to the table.
	///
	/// # Arguments
	///
	/// * `label` – The descriptive text to be displayed on the left-hand side,
	/// * `control` – The actual control widget to be displayed on the right-hand side.
	///
	/// # Returns
	///
	/// The returned value of the `control` closure, typically the [`egui::Response`] of the control widget.
	pub fn add<R> (&mut self, label: impl AsRef<str>, control: impl FnOnce(&mut egui::Ui, f32)->R) -> R
	{
		self.ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
			ui.set_min_width(self.lhsWidth);
			ui.label(label.as_ref());
		});
		let response = self.ui.horizontal(|ui| control(ui, self.rhsWidth)).inner;
		self.ui.end_row();
		response
	}

	/// Add a row to the table with a custom left-hand side.
	///
	/// # Arguments
	///
	/// * `lhs` – The UI content to display on the left-hand side of this row.
	/// * `rhs` – The actual control widget to be displayed on the right-hand side.
	///
	/// # Returns
	///
	/// A tuple containing the returned values of both closures, typically the [responses](egui::Response) of the
	/// respective widgets added to the row.
	pub fn addCustom<Rlhs, Rrhs> (
		&mut self, lhs: impl FnOnce(&mut egui::Ui, f32)->Rlhs, rhs: impl FnOnce(&mut egui::Ui, f32)->Rrhs
	) -> (Rlhs, Rrhs)
	{
		let lhsResponse = self.ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
			ui.set_min_width(self.lhsWidth);
			lhs(ui, self.lhsWidth)
		}).inner;
		let rhsResponse = rhs(self.ui, self.rhsWidth);
		self.ui.end_row();
		(lhsResponse, rhsResponse)
	}
}

/// A layout controller for adding [control tables](ControlTable) to a UI region.
///
/// The layouter, once initialized for a given *egui* [UI object](egui::Ui), can then be reused to add an arbitrary
/// number of control tables with differing contents to that UI object so long as the available horizontal space doesn't
/// change. This is typically the case for vertical parent layouts, or all subregions of the same level in a hierarchy
/// spawned by vertically stacking [collapsible regions](egui::CollapsingHeader).
pub struct ControlTableLayouter {
	#[doc=include_str!("_doc/ControlTable_lhsWidth.md")]
	lhsWidth: f32,

	#[doc=include_str!("_doc/ControlTable_rhsWidth.md")]
	rhsWidth: f32
}
impl ControlTableLayouter
{
	/// Creates a `ControlTableLayouter` for layouting a [`ControlTable`] inside the currently active region of the
	/// given *egui* [UI object](`egui::Ui`).
	///
	/// The enforced width of the right-hand side typically containing the actual control widgets will be exactly
	/// [`DEFAULT_RHS_WIDTH`]. If there is not enough available space to accommodate a right-hand side of that size, the
	/// whole region containing the defined [`ControlTable`] will get a horizontal scroll bar.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object `ControlTable`s should be layouted for.
	pub fn new (ui: &egui::Ui) -> Self {
		Self::withMinRhsWidth(ui, DEFAULT_RHS_WIDTH)
	}

	/// Creates a `ControlTableLayouter` for layouting a [`ControlTable`] inside the currently active region of the
	/// given *egui* [UI object](`egui::Ui`), forcing the specified minimum width for the right-hand side typically
	/// containing the actual control widgets.
	///
	/// If there is not enough available space to accommodate a right-hand side of that size, the whole region
	/// containing the defined [`ControlTable`] will get a horizontal scroll bar.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object `ControlTable`s should be layouted for.
	/// * `minRhsWidth` – The desired minimum width that the right-hand-side of the defined
	///                   [control tables](ControlTable) will never be smaller than.
	pub fn withMinRhsWidth (ui: &egui::Ui, minRhsWidth: f32) -> Self {
		let availableWidth = ui.available_width()-SIDEPANEL_SAFETY_MARGINS.x;
		let rhsWidth = f32::max(minRhsWidth, availableWidth*1./2.);
		Self { rhsWidth,  lhsWidth: f32::max(availableWidth-rhsWidth - ui.spacing().item_spacing.x, 0.) }
	}

	/// Start layouting the contents for a [`ControlTable`] as added by the user-provided closure.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object the [control tables](ControlTable) should be added to.
	/// * `idSalt` – A name that *egui* can use as salt for a hash that uniquely identifies the GUI region occupied by
	///              the fully layouted `ControlTable`.
	/// * `addRows` – Closure that adds all desired rows containing the controls.
	///
	/// # Returns
	///
	/// The [`egui::InnerResponse`] of the UI region containing the fully layouted `ControlTable`, with the returned
	/// value of the user closure `addRows` as the inner value.
	pub fn layout<R> (
		&self, ui: &mut egui::Ui, idSalt: impl AsRef<str>, addRows: impl FnOnce(&mut ControlTable)->R
	) -> egui::InnerResponse<R>
	{
		egui::Grid::new(idSalt.as_ref()).num_columns(2).striped(true).show(ui, move |ui| {
			ui.spacing_mut().slider_width = self.rhsWidth - gui::SLIDER_TEXTBOX_WIDTH;
			let mut controlTable = ControlTable::new(ui, self);
			addRows(&mut controlTable)
		})
	}
}
