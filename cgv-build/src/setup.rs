
pub struct Setup {
	additionalLinkerFlags: Option<String>
}

impl Setup
{
	pub fn new() -> Self { Self {
		additionalLinkerFlags: None
	}}

	pub fn fromFile (path: impl AsRef<std::path::Path>) -> anyhow::Result<Self>
	{
		// The resulting setup
		let mut setup = Self::new();

		// Parse build setup file
		let contents = std::fs::read_to_string(path)?;
		contents.split('\n').for_each(|line|
		{
			if let Some((key, value)) = line.split_once('=')
			{
				let key = key.trim().to_owned();
				let value = value.trim().to_owned();
				match key.as_str()
				{
					"ADDITIONAL_LINKER_ARGS" => setup.addLinkerFlag(&value),

					_ => {
						// Warn of unrecognized key and ignore
						println!("cargo:warning=cgv_build::Setup::fromFile(): Unrecognized key: {line}");
					}
				}
			} else {
				println!("cargo:warning=cgv_build::Setup::fromFile(): Cannot interpret line: {line}");
			}
		});
		Ok(setup)
	}

	pub fn addLinkerFlag (&mut self, flag: impl AsRef<str>)
	{
		if let Some(flags) = self.additionalLinkerFlags.as_mut() {
			flags.push_str(flag.as_ref());
		}
		else {
			self.additionalLinkerFlags = Some(flag.as_ref().into());
		}
	}

	pub fn apply (&self) {
		if let Some(flags) = self.additionalLinkerFlags.as_ref() {
			println!("cargo:rustc-link-arg={}", flags);
		}
	}
}
