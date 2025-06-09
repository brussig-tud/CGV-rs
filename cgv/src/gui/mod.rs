
//////
//
// Module definitions
//

/// Submodule providing assorted custom widgets.
pub mod widget;

/// Submodule providing pseudo-atomic controls for somewhat more complex datatypes.
pub mod complex;

/// Submodule providing custom "smart" layouts.
pub mod layout;



//////
//
// Module-wide constants
//

/// The empirically determined width (in logical units) of the text box behind *egui* sliders including padding.
pub const SLIDER_TEXTBOX_WIDTH: f32 = 56.;
