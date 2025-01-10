
//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// Egui library
use egui;

// Local imports
use crate::util;



//////
//
// Structs
//

/// Holds a pair of widgets making up a row in a [`ControlTable`].
struct ControlTableRow {
	lhs: Box<dyn FnOnce(&mut egui::Ui, f32) -> egui::Response>,
	rhs: Box<dyn FnOnce(&mut egui::Ui, f32) -> egui::Response>
}

/// A smart 2-column grid layout that uses the assumption that it will contain a list of controls, with a descriptive
/// label on the left and the control on the right, to efficiently make use of the available space while maintaining a
/// clean and tidy look.
pub struct ControlTable<'ui, 'id_salt>
{
	ui: &'ui egui::Ui,
	id_salt: &'id_salt str,

	rhsWidth: f32,
	lhsMinWidth: f32,
	controls: Vec<ControlTableRow>
}
impl<'ui, 'id_salt> ControlTable<'ui, 'id_salt>
{
	/// Create a `ControlTable` that will operate on the given *egui* UI object, specifying a minimum width that the
	/// right-hand side will never be smaller than. If there is not enough available space to accommodate a right-hand
	/// side of that size, then the whole layouted region will get a horizontal scroll bar.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object the contents of the `ControlTable` should appear on.
	/// * `id_salt` – A name that *egui* can use as salt for a hash that uniquely identifies the GUI region occupied by
	///               the layout.
	/// * `minRhsWidth` – The desired minimum width that the right-hand-side of the control table will never be smaller
	///                   than.
	pub fn withUiAndMinRhsWidth (ui: &'ui mut egui::Ui, id_salt: &'id_salt str, minRhsWidth: f32) -> Self {
		let availableWidth = ui.available_width();
		let rhsWidth = f32::max(minRhsWidth, availableWidth*1./2.);
		let controls = Vec::<ControlTableRow>::with_capacity(16);
		Self {
			ui, id_salt, rhsWidth, controls,
			lhsMinWidth: f32::max(availableWidth-rhsWidth - ui.spacing().item_spacing.x, 0.)
		}
	}

	/// Create a `ControlTable` that will operate on the given *egui* UI object. The minimum size of the control column
	/// will be 192 units.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object the contents of the `ControlTable` should appear on.
	/// * `id_salt` – A name that *egui* can use as salt for a hash that uniquely identifies the GUI region occupied by
	///               the layout.
	pub fn withUi (ui: &'ui mut egui::Ui, id_salt: &'id_salt str) -> Self {
		Self::withUiAndMinRhsWidth(ui, id_salt, 192f32)
	}

	/// Add a control to the Table.
	///
	/// # Arguments
	///
	/// * `lhs` – Contents on the left-hand side (typically a label describing the control).
	/// * `rhs` – Contents on the right-hand side (typically the actual control widget).
	pub fn add (
		&mut self,
		lhs: impl FnOnce(&mut egui::Ui, f32)->egui::Response + 'static,
		rhs: impl FnOnce(&mut egui::Ui, f32)->egui::Response + 'static
	){
		self.controls.push(ControlTableRow { lhs: Box::new(lhs), rhs: Box::new(rhs) });
	}

	/// Render the layouted GUI.
	pub fn show (self)
	{
		egui::Grid::new(self.id_salt).num_columns(2).striped(true).show(util::mutify(self.ui), |ui|
		{
			for control in self.controls
			{
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(self.lhsMinWidth);
					(control.lhs)(ui, self.lhsMinWidth);
				});
				(control.rhs)(ui, self.rhsWidth);
				ui.end_row();
			}
		});
	}
}
