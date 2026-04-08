
//////
//
// Language config
//

// Allow debugging the build script
#![allow(internal_features)]
#![feature(core_intrinsics)]



//////
//
// Functions
//

/// Attach VS Code debugger to the build process, optionally halting execution right after
pub fn debugWithVsCode (halt: bool) -> Result<()>
{
	// Build launch config URL
	let url = format!(
		"vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}", std::process::id()
	);

	// Start VS Code
	match std::process::Command::new("code").arg("--open-url").arg(url).output()
	{
		Ok(output) => {
			if output.status.success() {
				std::thread::sleep(std::time::Duration::from_secs(3)); // <- give debugger time to attach
				if halt {
					std::intrinsics::breakpoint();
				}
				Ok(())
			} else {
				Err(anyhow!("Could not attach debugger to build process: {}", String::from_utf8_lossy(&output.stderr)))
			}
		},

		Err(err) => Err(anyhow!("Could not attach debugger to build process: {}", err))
	}
}
