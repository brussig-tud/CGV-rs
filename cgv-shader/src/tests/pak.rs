
//////
//
// Imports
//

// Standard library
#[allow(unused_imports)] // BTreeSet is only required in case of some enabled features
use std::{path::Path, collections::{BTreeMap, BTreeSet}, fmt::Display};

// Local imports
use crate::*;



//////
//
// Tests
//

#[cfg(all(feature="compilation",feature="slang_runtime"))]
#[test]
fn test_shaderPackaging_fromSourceFile ()
{
	// Build shader package from test source file
	let slangCtx = tests::slang::createContext(slang::obtainGlobalSession());
	let feasibleSourceTypes = feasibleSourceTypes();
	let package = Package::fromSourceFileMultipleTypes(
		feasibleSourceTypes, &slangCtx, util::pathInsideCrate!("/shader/tests/multiple_entrypoints.slang"),
		None
	).expect("failed to create shader package");

	// Serialize
	let blob = package.serialize();

	// Deserialize and check
	let package = Package::deserialize(&blob).expect("failed to deserialize test shader package");
	// - can we retrieve each source type we packaged, and does each contain specializations for all expected entry
	//   points?
	for &sourceType in feasibleSourceTypes
	{
		// Was the instance packaged?
		let instance = package.instance(sourceType);
		assert!(
			instance.is_some(), "shader package does not contain an instance for expected source type {sourceType}"
		);
		let instance = instance.unwrap();

		// Does the instance contain the generic program that includes all entry points?
		assert!(
			instance.code(None).is_some(),
			"shader package instance for source type `{sourceType}` does not contain the generic program"
		);

		// Does the instance contain specializations for all expected entry points?
		for entryPoint in ["vertexMain", "fragmentMain", "computeMain1", "computeMain2"] {
			assert!(
				instance.code(Some(entryPoint)).is_some(),
				"shader package instance for source type `{sourceType}` does not contain a specialization for expected\
				 entry point '{entryPoint}'"
			);
		}
	}
}
