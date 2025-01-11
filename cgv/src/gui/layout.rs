
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Egui library
use egui;



//////
//
// Structs
//

////
// ControlTable

/// Holds a pair of widgets making up a row in a [`ControlTable`].
struct ControlTableRow<'captures> {
	lhs: Option<Box<dyn FnOnce(&mut egui::Ui, f32) + 'captures>>,
	rhs: Box<dyn FnOnce(&mut egui::Ui, f32) + 'captures>
}

/// A smart 2-column grid layout that uses the assumption that it will contain a list of controls, with a descriptive
/// label on the left and the control on the right, to efficiently make use of the available space while maintaining a
/// clean and tidy look.
///
/// Widgets are provided via Closures taking the active *egui* [UI object](egui::Ui) as well as an `f32` indicating the
/// ideal width of any widget added to the grid cell (see example), which users are free to ignore.
pub struct ControlTable<'captures> {
	minRhsWidth: f32,
	controls: Vec<ControlTableRow<'captures>>
}
impl<'captures> ControlTable<'captures>
{
	/// Create a `ControlTable` with the specified minimum width for the right-hand side. If there is not enough
	/// available space to accommodate a right-hand side of that size, the whole layouted region will get a horizontal
	/// scroll bar.
	///
	/// # Arguments
	///
	/// * `minRhsWidth` – The desired minimum width that the right-hand-side of the control table will never be smaller
	///                   than.
	pub fn withMinRhsWidth (minRhsWidth: f32) -> Self {
		Self{minRhsWidth,  controls: Vec::<ControlTableRow>::with_capacity(16)}
	}

	/// Add a labeled control to the table.
	///
	/// # Arguments
	///
	/// * `label` – The descriptive text to be displayed on the left-hand side,
	/// * `control` – The actual control widget to be displayed on the right-hand side.
	pub fn add (
		&mut self, label: impl AsRef<str> + 'captures,
		control: impl FnOnce(&mut egui::Ui, f32) + 'captures
	){
		self.controls.push(ControlTableRow {
			lhs: Some(Box::new(move |ui, _| {ui.label(label.as_ref());} )),
			rhs: Box::new(control)
		});
	}

	/// Add a labeled control to the table, where the control response is not being evaluated further and can thus be
	/// conveniently omitted by the caller.
	///
	/// # Arguments
	///
	/// * `label` – The descriptive text to be displayed on the left-hand side,
	/// * `control` – The actual control widget to be displayed on the right-hand side.
	pub fn addWithoutResponse (
		&mut self, label: impl AsRef<str> + 'captures,
		control: impl FnOnce(&mut egui::Ui, f32) -> egui::Response + 'captures
	){
		self.controls.push(ControlTableRow {
			lhs: Some(Box::new(move |ui, _| { ui.label(label.as_ref()); })),
			rhs: Box::new(move |ui, idealWidth| { control(ui, idealWidth); })
		});
	}

	/// Add control without a left-hand side label to the table.
	///
	/// # Arguments
	///
	/// * `control` – The actual control widget to be displayed on the right-hand side.
	pub fn addWithoutLabel (
		&mut self, control: impl FnOnce(&mut egui::Ui, f32) + 'captures
	){
		self.controls.push(ControlTableRow{ lhs: None,  rhs: Box::new(control) });
	}

	/// Add control without a left-hand side label to the table, where the control response from the right-hand side
	/// widget is not being evaluated further and can thus be conveniently omitted by the caller.
	///
	/// # Arguments
	///
	/// * `control` – The actual control widget to be displayed on the right-hand side.
	pub fn addWithoutLabelAndResponse (
		&mut self, control: impl FnOnce(&mut egui::Ui, f32)->egui::Response + 'captures
	){
		self.controls.push(ControlTableRow {
			lhs: None,
			rhs: Box::new(move |ui, idealWidth| { control(ui, idealWidth); })
		});
	}

	/// Add a control to the table, with a user-provided widget on the left-hand side.
	///
	/// # Arguments
	///
	/// * `lhs` – Contents on the left-hand side (typically a label describing the control).
	/// * `rhs` – Contents on the right-hand side (typically the actual control widget).
	pub fn addCustom (
		&mut self,
		lhs: impl FnOnce(&mut egui::Ui, f32)->egui::Response + 'captures,
		rhs: impl FnOnce(&mut egui::Ui, f32)->egui::Response + 'captures
	){
		self.controls.push(ControlTableRow{
			lhs: Some(Box::new(move |ui, idealWidth| { lhs(ui, idealWidth); })),
			rhs: Box::new(move |ui, idealWidth| { rhs(ui, idealWidth); })
		});
	}

	/// Render the layouted GUI on the provided *egui* UI object.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object the contents of the `ControlTable` should appear for.
	/// * `idSalt` – A name that *egui* can use as salt for a hash that uniquely identifies the GUI region occupied by
	///              the layout.
	pub fn show (self, ui: &mut egui::Ui, idSalt: impl AsRef<str>)
	{
		let availableWidth = ui.available_width();
		let rhsWidth = f32::max(self.minRhsWidth, availableWidth*1./2.);
		let lhsMinWidth = f32::max(availableWidth-rhsWidth - ui.spacing().item_spacing.x, 0.);
		egui::Grid::new(idSalt.as_ref()).num_columns(2).striped(true).show(ui, move |ui|
		{
			ui.spacing_mut().slider_width = rhsWidth-56.;
			for control in self.controls
			{
				if let Some(lhs) = control.lhs {
					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
						ui.set_min_width(lhsMinWidth);
						lhs(ui, lhsMinWidth);
					});
				}
				(control.rhs)(ui, rhsWidth);
				ui.end_row();
			}
		});
	}
}

impl Default for ControlTable<'_> {
	/// Create a `ControlTable` with the default minimum size of 192 units for the right-hand side containing the actual
	/// controls.
	fn default () -> Self {
		Self::withMinRhsWidth(192.)
	}
}
