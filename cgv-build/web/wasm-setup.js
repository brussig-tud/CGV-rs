// Hand over control to CGV-rs WASM module
console.info("Delegating control to CGV-rs application '@PACKAGE_NAME@'");
init().catch((error) => {
	// Prevent the necessary control flow exception from showing up in the console
	if (!error.message.startsWith(
		"Using exceptions for control flow, don't mind me. This isn't actually an error!"
	)) {
		throw error; // Legit error, rethrow
	}
});
