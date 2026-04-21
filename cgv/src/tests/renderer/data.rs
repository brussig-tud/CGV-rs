//////
//
// Imports
//

// Standard library
use std::{marker::PhantomData, panic::catch_unwind};

// Local imports
use crate::{self as cgv, *, renderer::data::*};



//////
//
// Structs
//

/// Mock data for the tests.
#[derive(Clone)]
struct MockData {
	positions: Vec<glm::Vec3>,
	indices: Vec<u32>,
	normals: Vec<glm::Vec3>,
	tangents: Vec<glm::Vec3>,
	radii: Vec<f32>,
	radiusDerivs: Vec<f32>,
	orientations: Vec<glm::Quat>,
	scalings: Vec<glm::Vec3>,
	colors: Vec<cgv::RGBA>,
	hasNormals: bool,
	hasTangents: bool,
	hasRadii: bool,
	hasRadiusDerivs: bool,
	hasOrientations: bool,
	hasScalings: bool,
	hasColors: bool
}
impl MockData
{
	fn withFlags (
		hasNormals: bool, hasTangents: bool, hasRadii: bool, hasRadiusDerivs: bool, hasOrientations: bool,
		hasScalings: bool, hasColors: bool
	) -> Self {
		Self {
			positions: vec![
				glm::vec3(0., 0., 0.),
				glm::vec3(1., 2., 3.)
			],
			indices: vec![0, 1, 0],
			normals: vec![
				glm::vec3(0., 0., 1.),
				glm::vec3(0., 1., 0.)
			],
			tangents: vec![
				glm::vec3(1., 0., 0.),
				glm::vec3(0., 1., 1.)
			],
			radii: vec![0.5, 1.5],
			radiusDerivs: vec![0.25, 0.75],
			orientations: vec![
				glm::quat_identity(),
				glm::quat_angle_axis(0.5, &glm::vec3(0., 1., 0.))
			],
			scalings: vec![
				glm::vec3(1., 1., 1.),
				glm::vec3(2., 3., 4.)
			],
			colors: vec![
				cgv::RGBA::from_rgba_unmultiplied(1., 0., 0., 1.),
				cgv::RGBA::from_rgba_unmultiplied(0., 1., 0., 1.)
			],
			hasNormals,
			hasTangents,
			hasRadii,
			hasRadiusDerivs,
			hasOrientations,
			hasScalings,
			hasColors
		}
	}

	fn fullyPopulated () -> Self {
		Self::withFlags(true, true, true, true, true, true, true)
	}
}

impl Data for MockData
{
	type PosIterator = std::vec::IntoIter<glm::Vec3>;

	fn num (&self) -> u32 { self.positions.len() as u32 }
	fn positions (&self) -> Self::PosIterator { self.positions.clone().into_iter() }
	fn pos (&self, index: u32) -> &glm::Vec3 { &self.positions[index as usize] }
}
impl Indexed for MockData
{
	type IndexIterator = std::vec::IntoIter<u32>;

	fn numIndices (&self) -> u32 { self.indices.len() as u32 }
	fn indices (&self) -> Self::IndexIterator { self.indices.clone().into_iter() }
	fn index (&self, index: u32) -> u32 { self.indices[index as usize] }
}
impl Interleaved for MockData {}

impl CanHaveNormals for MockData
{
	type NormalIterator = std::vec::IntoIter<glm::Vec3>;

	fn hasNormals (&self) -> bool { self.hasNormals }
	fn normals (&self) -> Self::NormalIterator {
		assert!(self.hasNormals());
		self.normals.clone().into_iter()
	}
	fn normal (&self, index: u32) -> &glm::Vec3 {
		assert!(self.hasNormals());
		&self.normals[index as usize]
	}
}
impl CanHaveTangents for MockData
{
	type TangentIterator = std::vec::IntoIter<glm::Vec3>;

	fn hasTangents (&self) -> bool { self.hasTangents }
	fn tangents (&self) -> Self::TangentIterator {
		assert!(self.hasTangents());
		self.tangents.clone().into_iter()
	}
	fn tangent (&self, index: u32) -> &glm::Vec3 {
		assert!(self.hasTangents());
		&self.tangents[index as usize]
	}
}
impl CanHaveRadii for MockData
{
	type RadiusIterator = std::vec::IntoIter<f32>;

	fn hasRadii (&self) -> bool { self.hasRadii }
	fn radii (&self) -> Self::RadiusIterator {
		assert!(self.hasRadii());
		self.radii.clone().into_iter()
	}
	fn radius (&self, index: u32) -> f32 {
		assert!(self.hasRadii());
		self.radii[index as usize]
	}
}
impl CanHaveRadiusDerivs for MockData
{
	fn hasRadiusDerivs (&self) -> bool { self.hasRadiusDerivs }
	fn radiusDerivs (&self) -> Self::RadiusIterator {
		assert!(self.hasRadiusDerivs());
		self.radiusDerivs.clone().into_iter()
	}
	fn radiusDeriv (&self, index: u32) -> f32 {
		assert!(self.hasRadiusDerivs());
		self.radiusDerivs[index as usize]
	}
}
impl CanHaveOrientations for MockData
{
	type OrientationIterator = std::vec::IntoIter<glm::Quat>;

	fn hasOrientations (&self) -> bool { self.hasOrientations }
	fn orientations (&self) -> Self::OrientationIterator {
		assert!(self.hasOrientations());
		self.orientations.clone().into_iter()
	}
	fn orientation (&self, index: u32) -> &glm::Quat {
		assert!(self.hasOrientations());
		&self.orientations[index as usize]
	}
}
impl CanHaveScalings for MockData
{
	type ScaleIterator = std::vec::IntoIter<glm::Vec3>;

	fn hasScalings (&self) -> bool { self.hasScalings }
	fn scalings (&self) -> Self::ScaleIterator {
		assert!(self.hasScalings());
		self.scalings.clone().into_iter()
	}
	fn scaling (&self, index: u32) -> &glm::Vec3 {
		assert!(self.hasScalings());
		&self.scalings[index as usize]
	}
}
impl CanHaveColors for MockData
{
	type ColorIterator = std::vec::IntoIter<cgv::RGBA>;

	fn hasColors (&self) -> bool { self.hasColors }
	fn colors (&self) -> Self::ColorIterator {
		assert!(self.hasColors());
		self.colors.clone().into_iter()
	}
	fn color (&self, index: u32) -> &cgv::RGBA {
		assert!(self.hasColors());
		&self.colors[index as usize]
	}
}



//////
//
// Tests
//

#[test]
fn test_guaranteeWrappers_forwardDataAndMarkerTraits ()
{
	fn assertIndexed<T: Indexed> () {}
	fn assertInterleaved<T: Interleaved> () {}
	fn assertHasNormals<T: HasNormals> () {}

	assertIndexed::<GuaranteeNormals<MockData>>();
	assertInterleaved::<GuaranteeNormals<MockData>>();
	assertHasNormals::<GuaranteeNormals<MockData>>();

	let guaranteed = GuaranteeNormals::new(MockData::fullyPopulated());
	assert_eq!(guaranteed.num(), 2);
	assert_eq!(guaranteed.numIndices(), 3);
	assert_eq!(guaranteed.pos(0), &glm::vec3(0., 0., 0.));
	assert_eq!(guaranteed.normal(1), &glm::vec3(0., 1., 0.));
	assert_eq!(guaranteed.radius(0), 0.5);
	assert_eq!(guaranteed.index(1), 1);
	assert!(guaranteed.hasNormals());
	assert!(guaranteed.hasTangents());
	assert!(guaranteed.hasRadii());

	let mut positions = guaranteed.positions();
	assert_eq!(positions.next(), Some(glm::vec3(0., 0., 0.)));
	assert_eq!(positions.next(), Some(glm::vec3(1., 2., 3.)));
	assert_eq!(positions.next(), None);
}

#[test]
fn test_guaranteeWrappers_radiusDerivsGuaranteeRadii ()
{
	fn assertHasRadii<T: HasRadii> () {}
	fn assertHasRadiusDerivs<T: HasRadiusDerivs> () {}

	assertHasRadii::<GuaranteeRadiusDerivs<MockData>>();
	assertHasRadiusDerivs::<GuaranteeRadiusDerivs<MockData>>();

	let guaranteed = GuaranteeRadiusDerivs::new(MockData::fullyPopulated());
	assert!(guaranteed.hasRadii());
	assert!(guaranteed.hasRadiusDerivs());
	assert_eq!(guaranteed.radius(1), 1.5);
	assert_eq!(guaranteed.radiusDeriv(0), 0.25);
}

#[test]
fn test_guaranteeWrappers_allTypeAliasesResolve ()
{
	macro_rules! assertAliasesExist {
		($($alias:ident),+ $(,)?) => {{
			$(let _: PhantomData<$alias<MockData>> = PhantomData;)+
		}};
	}

	assertAliasesExist!(
		GuaranteeNormals, GuaranteeTangents, GuaranteeRadii, GuaranteeRadiusDerivs, GuaranteeOrientations,
		GuaranteeScalings, GuaranteeColors, GuaranteeNormalsTangents, GuaranteeNormalsRadii,
		GuaranteeNormalsRadiusDerivs, GuaranteeNormalsOrientations, GuaranteeNormalsScalings,
		GuaranteeNormalsColors, GuaranteeTangentsRadii, GuaranteeTangentsRadiusDerivs,
		GuaranteeTangentsOrientations, GuaranteeTangentsScalings, GuaranteeTangentsColors,
		GuaranteeRadiiRadiusDerivs, GuaranteeRadiiOrientations, GuaranteeRadiiScalings,
		GuaranteeRadiiColors, GuaranteeRadiusDerivsOrientations, GuaranteeRadiusDerivsScalings,
		GuaranteeRadiusDerivsColors, GuaranteeOrientationsScalings, GuaranteeOrientationsColors,
		GuaranteeScalingsColors, GuaranteeNormalsTangentsRadii, GuaranteeNormalsTangentsRadiusDerivs,
		GuaranteeNormalsTangentsOrientations, GuaranteeNormalsTangentsScalings, GuaranteeNormalsTangentsColors,
		GuaranteeNormalsRadiiRadiusDerivs, GuaranteeNormalsRadiiOrientations, GuaranteeNormalsRadiiScalings,
		GuaranteeNormalsRadiiColors, GuaranteeNormalsRadiusDerivsOrientations,
		GuaranteeNormalsRadiusDerivsScalings, GuaranteeNormalsRadiusDerivsColors,
		GuaranteeNormalsOrientationsScalings, GuaranteeNormalsOrientationsColors,
		GuaranteeNormalsScalingsColors, GuaranteeTangentsRadiiRadiusDerivs,
		GuaranteeTangentsRadiiOrientations, GuaranteeTangentsRadiiScalings, GuaranteeTangentsRadiiColors,
		GuaranteeTangentsRadiusDerivsOrientations, GuaranteeTangentsRadiusDerivsScalings,
		GuaranteeTangentsRadiusDerivsColors, GuaranteeTangentsOrientationsScalings,
		GuaranteeTangentsOrientationsColors, GuaranteeTangentsScalingsColors,
		GuaranteeRadiiRadiusDerivsOrientations, GuaranteeRadiiRadiusDerivsScalings,
		GuaranteeRadiiRadiusDerivsColors, GuaranteeRadiiOrientationsScalings,
		GuaranteeRadiiOrientationsColors, GuaranteeRadiiScalingsColors,
		GuaranteeRadiusDerivsOrientationsScalings, GuaranteeRadiusDerivsOrientationsColors,
		GuaranteeRadiusDerivsScalingsColors, GuaranteeOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiiRadiusDerivs, GuaranteeNormalsTangentsRadiiOrientations,
		GuaranteeNormalsTangentsRadiiScalings, GuaranteeNormalsTangentsRadiiColors,
		GuaranteeNormalsTangentsRadiusDerivsOrientations,
		GuaranteeNormalsTangentsRadiusDerivsScalings, GuaranteeNormalsTangentsRadiusDerivsColors,
		GuaranteeNormalsTangentsOrientationsScalings, GuaranteeNormalsTangentsOrientationsColors,
		GuaranteeNormalsTangentsScalingsColors, GuaranteeNormalsRadiiRadiusDerivsOrientations,
		GuaranteeNormalsRadiiRadiusDerivsScalings, GuaranteeNormalsRadiiRadiusDerivsColors,
		GuaranteeNormalsRadiiOrientationsScalings, GuaranteeNormalsRadiiOrientationsColors,
		GuaranteeNormalsRadiiScalingsColors, GuaranteeNormalsRadiusDerivsOrientationsScalings,
		GuaranteeNormalsRadiusDerivsOrientationsColors, GuaranteeNormalsRadiusDerivsScalingsColors,
		GuaranteeNormalsOrientationsScalingsColors, GuaranteeTangentsRadiiRadiusDerivsOrientations,
		GuaranteeTangentsRadiiRadiusDerivsScalings, GuaranteeTangentsRadiiRadiusDerivsColors,
		GuaranteeTangentsRadiiOrientationsScalings, GuaranteeTangentsRadiiOrientationsColors,
		GuaranteeTangentsRadiiScalingsColors, GuaranteeTangentsRadiusDerivsOrientationsScalings,
		GuaranteeTangentsRadiusDerivsOrientationsColors, GuaranteeTangentsRadiusDerivsScalingsColors,
		GuaranteeTangentsOrientationsScalingsColors, GuaranteeRadiiRadiusDerivsOrientationsScalings,
		GuaranteeRadiiRadiusDerivsOrientationsColors, GuaranteeRadiiRadiusDerivsScalingsColors,
		GuaranteeRadiiOrientationsScalingsColors, GuaranteeRadiusDerivsOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiiRadiusDerivsOrientations,
		GuaranteeNormalsTangentsRadiiRadiusDerivsScalings,
		GuaranteeNormalsTangentsRadiiRadiusDerivsColors,
		GuaranteeNormalsTangentsRadiiOrientationsScalings,
		GuaranteeNormalsTangentsRadiiOrientationsColors,
		GuaranteeNormalsTangentsRadiiScalingsColors,
		GuaranteeNormalsTangentsRadiusDerivsOrientationsScalings,
		GuaranteeNormalsTangentsRadiusDerivsOrientationsColors,
		GuaranteeNormalsTangentsRadiusDerivsScalingsColors,
		GuaranteeNormalsTangentsOrientationsScalingsColors,
		GuaranteeNormalsRadiiRadiusDerivsOrientationsScalings,
		GuaranteeNormalsRadiiRadiusDerivsOrientationsColors,
		GuaranteeNormalsRadiiRadiusDerivsScalingsColors,
		GuaranteeNormalsRadiiOrientationsScalingsColors,
		GuaranteeNormalsRadiusDerivsOrientationsScalingsColors,
		GuaranteeTangentsRadiiRadiusDerivsOrientationsScalings,
		GuaranteeTangentsRadiiRadiusDerivsOrientationsColors,
		GuaranteeTangentsRadiiRadiusDerivsScalingsColors,
		GuaranteeTangentsRadiiOrientationsScalingsColors,
		GuaranteeTangentsRadiusDerivsOrientationsScalingsColors,
		GuaranteeRadiiRadiusDerivsOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsScalings,
		GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsColors,
		GuaranteeNormalsTangentsRadiiRadiusDerivsScalingsColors,
		GuaranteeNormalsTangentsRadiiOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiusDerivsOrientationsScalingsColors,
		GuaranteeNormalsRadiiRadiusDerivsOrientationsScalingsColors,
		GuaranteeTangentsRadiiRadiusDerivsOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsScalingsColors
	);
}

#[test]
fn test_guaranteeWrappers_selectCombinationsPreserveGuaranteedTraits ()
{
	// Helper macros
	fn assertHasNormals<T: HasNormals> () {}
	fn assertHasTangents<T: HasTangents> () {}
	fn assertHasRadii<T: HasRadii> () {}
	fn assertHasRadiusDerivs<T: HasRadiusDerivs> () {}
	fn assertHasColors<T: HasColors> () {}
	fn assertHasOrientations<T: HasOrientations> () {}

	// Define test combo 1
	type GuaranteedCombo1 = GuaranteeNormalsTangentsRadiusDerivsColors<MockData>;
	assertHasNormals::<GuaranteedCombo1>();
	assertHasTangents::<GuaranteedCombo1>();
	assertHasRadii::<GuaranteedCombo1>();
	assertHasRadiusDerivs::<GuaranteedCombo1>();
	assertHasColors::<GuaranteedCombo1>();

	// Define test combo 2
	type GuaranteedCombo2 = GuaranteeNormalsRadiiRadiusDerivsOrientations<MockData>;
	assertHasNormals::<GuaranteedCombo2>();
	assertHasRadii::<GuaranteedCombo2>();
	assertHasRadiusDerivs::<GuaranteedCombo2>();
	assertHasOrientations::<GuaranteedCombo2>();

	// Runtime-test combo 1 via a single constructor on the combination alias
	let guaranteedCombo1 = GuaranteedCombo1::new(MockData::fullyPopulated());
	assert!(guaranteedCombo1.hasNormals());
	assert!(guaranteedCombo1.hasTangents());
	assert!(guaranteedCombo1.hasRadii());
	assert!(guaranteedCombo1.hasRadiusDerivs());
	assert!(guaranteedCombo1.hasColors());

	// Runtime-test combo 2 via a single constructor on the combination alias
	let guaranteedCombo2 = GuaranteedCombo2::new(MockData::fullyPopulated());
	assert!(guaranteedCombo2.hasNormals());
	assert!(guaranteedCombo2.hasTangents());
	assert!(guaranteedCombo2.hasRadii());
	assert!(guaranteedCombo2.hasRadiusDerivs());
	assert!(guaranteedCombo2.hasColors());
}

#[test]
fn test_guaranteeWrappers_constructorsRejectMissingAttributes ()
{
	macro_rules! assertPanics {
		($expression:expr) => {
			assert!(catch_unwind(|| $expression).is_err());
		};
	}

	assertPanics!(GuaranteeNormals::new(MockData::withFlags(false, true, true, true, true, true, true)));
	assertPanics!(GuaranteeTangents::new(MockData::withFlags(true, false, true, true, true, true, true)));
	assertPanics!(GuaranteeRadii::new(MockData::withFlags(true, true, false, false, true, true, true)));
	assertPanics!(GuaranteeRadiusDerivs::new(MockData::withFlags(true, true, false, true, true, true, true)));
	assertPanics!(GuaranteeRadiusDerivs::new(MockData::withFlags(true, true, true, false, true, true, true)));
	assertPanics!(GuaranteeOrientations::new(MockData::withFlags(true, true, true, true, false, true, true)));
	assertPanics!(GuaranteeScalings::new(MockData::withFlags(true, true, true, true, true, false, true)));
	assertPanics!(GuaranteeColors::new(MockData::withFlags(true, true, true, true, true, true, false)));
	assertPanics!(GuaranteeNormalsTangentsRadiusDerivsColors::new(MockData::withFlags(false, true, true, true, true, true, true)));
	assertPanics!(GuaranteeNormalsTangentsRadiusDerivsColors::new(MockData::withFlags(true, true, true, true, true, true, false)));
}
