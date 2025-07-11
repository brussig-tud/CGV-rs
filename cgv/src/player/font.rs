
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Egui library and framework
use egui;

// Local imports
use crate::*;



//////
//
// Functions
//

/// Replace the Egui default font with something that looks better on low-DPI
pub(crate) fn replaceDefaults (eguiContext: &egui::Context)
{
	// Start with the default fonts (we will be adding to them rather than replacing them).
	let mut fonts = egui::FontDefinitions::default();

	// Install CGV-rs selected default fonts
	let regularName = "NotoSans";
	let regularItalicName = "NotoSansItalic";
	let monoName = "JetBrainsMono";
	/*match eguiContext.theme()
	{
		egui::Theme::Dark => {
			fonts.font_data.insert(
				regularName.to_owned(), std::sync::Arc::new(egui::FontData::from_static(util::sourceBytes!(
					"/res/font/NotoSans/NotoSans-Light.ttf"
				)))
			);
			fonts.font_data.insert(
				regularItalicName.to_owned(), std::sync::Arc::new(egui::FontData::from_static(util::sourceBytes!(
					"/res/font/NotoSans/NotoSans-LightItalic.ttf"
				))),
			);
		},

		egui::Theme::Light => {*/
			fonts.font_data.insert(
				regularName.to_owned(), std::sync::Arc::new(egui::FontData::from_static(util::sourceBytes!(
					"/res/font/NotoSans/NotoSans.ttf"
				)))
			);
			fonts.font_data.insert(
				regularItalicName.to_owned(), std::sync::Arc::new(egui::FontData::from_static(util::sourceBytes!(
					"/res/font/NotoSans/NotoSans-Italic.ttf"
				))),
			);
		/*}
	};*/
	fonts.font_data.insert(
		monoName.to_owned(), std::sync::Arc::new(egui::FontData::from_static(util::sourceBytes!(
			"/res/font/JetBrainsMono/JetBrainsMono.ttf"
		))),
	);

	// Set highest priority for each font in its respective family, effectively making them the default
	fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, regularName.to_owned());
	fonts.families.entry(egui::FontFamily::Name("Italics".into())).or_default()
		.insert(0, regularItalicName.to_owned());
	fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, monoName.to_owned());

	// Tell egui to use these fonts
	eguiContext.set_fonts(fonts);
}
