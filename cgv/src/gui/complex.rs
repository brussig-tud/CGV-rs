
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Egui library
use egui;

// GLM library
use glm;



//////
//
// Functions
//

pub fn vec3 (ui: &mut egui::Ui, vec: &mut glm::Vec3) -> bool
{
	let speed = f32::max(0.03125*vec.norm(), 0.03125) as f64;
	let mut changed = false;
	changed |= ui.add(egui::DragValue::new(&mut vec.x).speed(speed)).changed();
	changed |= ui.add(egui::DragValue::new(&mut vec.y).speed(speed)).changed();
	changed |= ui.add(egui::DragValue::new(&mut vec.z).speed(speed)).changed();
	changed
}

pub fn vec3_sized (ui: &mut egui::Ui, vec: &mut glm::Vec3, width: f32) -> bool
{
	let itemSpacing = ui.spacing().item_spacing.x;
	let boxWidth = (width-itemSpacing-itemSpacing)/3.;
	let boxSize = egui::vec2(boxWidth, ui.style().spacing.interact_size.y);
	ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
	let speed = f32::max(0.03125*vec.norm(), 0.03125) as f64;
	let mut changed = false;
	changed |= ui.add_sized(boxSize, egui::DragValue::new(&mut vec.x).speed(speed)).changed();
	changed |= ui.add_sized(boxSize, egui::DragValue::new(&mut vec.y).speed(speed)).changed();
	changed |= ui.add_sized(boxSize, egui::DragValue::new(&mut vec.z).speed(speed)).changed();
	changed
}
