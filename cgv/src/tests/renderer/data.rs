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
	normals: Option<Vec<glm::Vec3>>,
	tangents: Option<Vec<glm::Vec3>>,
	radii: Option<Vec<f32>>,
	radiusDerivs: Option<Vec<f32>>,
	orientations: Option<Vec<glm::Quat>>,
	scalings: Option<Vec<glm::Vec3>>,
	colors: Option<Vec<cgv::RGBA>>
}
impl MockData
{
	fn withFlags (
		hasNormals: bool, hasTangents: bool, hasRadii: bool, hasRadiusDerivs: bool, hasOrientations: bool,
		hasScalings: bool, hasColors: bool
	) -> Self { Self {
		positions: vec![
			glm::vec3(0., 0., 0.), glm::vec3(1., 2., 3.)
		],
		indices: vec![0, 1, 0],
		normals: if hasNormals {
			Some(vec![glm::vec3(0., 0., 1.), glm::vec3(0., 1., 0.)])
		} else { None },
		tangents: if hasTangents {
			Some(vec![glm::vec3(1., 0., 0.), glm::vec3(0., 1., 1.)])
		} else { None },
		radii: if hasRadii { Some(vec![0.5, 1.5]) } else { None },
		radiusDerivs: if hasRadiusDerivs { Some(vec![0.25, 0.75]) } else { None },
		orientations: if hasOrientations {
			Some(vec![glm::quat_identity(), glm::quat_angle_axis(0.5, &glm::vec3(0., 1., 0.))])
		} else { None },
		scalings: if hasScalings {
			Some(vec![glm::vec3(1., 1., 1.), glm::vec3(2., 3., 4.)])
		} else { None },
		colors: if hasColors {
			Some(vec![
				cgv::RGBA::from_rgba_unmultiplied(1., 0., 0., 1.),
				cgv::RGBA::from_rgba_unmultiplied(0., 1., 0., 1.)
			])
		} else { None }
	}}

	fn fullyPopulated () -> Self {
		Self::withFlags(
			true, true, true, true, true, true,
			true
		)
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

	fn hasNormals (&self) -> bool { self.normals.is_some() }
	fn normals (&self) -> Self::NormalIterator {
		self.normals.as_ref().expect("should have normals").clone().into_iter()
	}
	fn normal (&self, index: u32) -> &glm::Vec3 {
		&self.normals.as_ref().expect("should have normals")[index as usize]
	}
}
impl CanHaveTangents for MockData
{
	type TangentIterator = std::vec::IntoIter<glm::Vec3>;

	fn hasTangents (&self) -> bool { self.tangents.is_some() }
	fn tangents (&self) -> Self::TangentIterator {
		self.tangents.as_ref().expect("should have tangents").clone().into_iter()
	}
	fn tangent (&self, index: u32) -> &glm::Vec3 {
		&self.tangents.as_ref().expect("should have tangents")[index as usize]
	}
}
impl CanHaveRadii for MockData
{
	type RadiusIterator = std::vec::IntoIter<f32>;

	fn hasRadii (&self) -> bool { self.radii.is_some() }
	fn radii (&self) -> Self::RadiusIterator {
		self.radii.as_ref().expect("should have radii").clone().into_iter()
	}
	fn radius (&self, index: u32) -> f32 {
		self.radii.as_ref().expect("should have radii")[index as usize]
	}
}
impl CanHaveRadiusDerivs for MockData
{
	fn hasRadiusDerivs (&self) -> bool { self.radiusDerivs.is_some() }
	fn radiusDerivs (&self) -> Self::RadiusIterator {
		self.radiusDerivs.as_ref().expect("should have radius derivatives").clone().into_iter()
	}
	fn radiusDeriv (&self, index: u32) -> f32 {
		self.radiusDerivs.as_ref().expect("should have radius derivatives")[index as usize]
	}
}
impl CanHaveOrientations for MockData
{
	type OrientationIterator = std::vec::IntoIter<glm::Quat>;

	fn hasOrientations (&self) -> bool { self.orientations.is_some() }
	fn orientations (&self) -> Self::OrientationIterator {
		self.orientations.as_ref().expect("should have orientations").clone().into_iter()
	}
	fn orientation (&self, index: u32) -> &glm::Quat {
		&self.orientations.as_ref().expect("should have orientations")[index as usize]
	}
}
impl CanHaveScalings for MockData
{
	type ScaleIterator = std::vec::IntoIter<glm::Vec3>;

	fn hasScalings (&self) -> bool { self.scalings.is_some() }
	fn scalings (&self) -> Self::ScaleIterator {
		self.scalings.as_ref().expect("should have scaling vectors").clone().into_iter()
	}
	fn scaling (&self, index: u32) -> &glm::Vec3 {
		&self.scalings.as_ref().expect("should have scaling vectors")[index as usize]
	}
}
impl CanHaveColors for MockData
{
	type ColorIterator = std::vec::IntoIter<cgv::RGBA>;

	fn hasColors (&self) -> bool { self.colors.is_some() }
	fn colors (&self) -> Self::ColorIterator {
		self.colors.as_ref().expect("should have colors").clone().into_iter()
	}
	fn color (&self, index: u32) -> &cgv::RGBA {
		&self.colors.as_ref().expect("should have colors")[index as usize]
	}
}



//////
//
// Functions
//

/// For testing at compile time that the generic argument is a type that represents indexed render data.
fn staticAssertIndexed<T: Indexed> () {}

/// For testing at compile time that the generic argument is a type that represents interleaved render data.
fn staticAssertInterleaved<T: Interleaved> () {}

/// For testing at compile time that the generic argument is a type that guarantees normals.
fn staticAssertHasNormals<T: HasNormals> () {}

/// For testing at compile time that the generic argument is a type that guarantees tangents.
fn staticAssertHasTangents<T: HasTangents> () {}

/// For testing at compile time that the generic argument is a type that guarantees radii.
fn staticAssertHasRadii<T: HasRadii> () {}

/// For testing at compile time that the generic argument is a type that guarantees radius derivatives.
fn staticAssertHasRadiusDerivs<T: HasRadiusDerivs> () {}

/// For testing at compile time that the generic argument is a type that guarantees orientations.
fn staticAssertHasOrientations<T: HasOrientations> () {}

/// For testing at compile time that the generic argument is a type that guarantees colors.
fn staticAssertHasColors<T: HasColors> () {}



//////
//
// Tests
//

#[test]
fn test_guaranteeWrappers_forwardMarkerTraitsAndData ()
{
	// Compile-time check that the marker traits get propagated
	staticAssertIndexed::<GuaranteeNormals<MockData>>();
	staticAssertInterleaved::<GuaranteeNormals<MockData>>();

	// Compile-time check that the normals guarantee is made
	staticAssertHasNormals::<GuaranteeNormals<MockData>>();

	// Check at runtime too
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

	// Check iterators
	let mut positions = guaranteed.positions();
	assert_eq!(positions.next(), Some(glm::vec3(0., 0., 0.)));
	assert_eq!(positions.next(), Some(glm::vec3(1., 2., 3.)));
	assert_eq!(positions.next(), None);
	let mut indices = guaranteed.indices();
	assert_eq!(indices.next(), Some(0));
	assert_eq!(indices.next(), Some(1));
	assert_eq!(indices.next(), Some(0));
	assert_eq!(indices.next(), None);
	let mut normals = guaranteed.normals();
	assert_eq!(normals.next(), Some(glm::vec3(0., 0., 1.)));
	assert_eq!(normals.next(), Some(glm::vec3(0., 1., 0.)));
	assert_eq!(normals.next(), None);
}

#[test]
fn test_guaranteeWrappers_radiusDerivsGuaranteeRadii ()
{
	// Compile-time check that both derivatives and radii become guaranteed
	staticAssertHasRadii::<GuaranteeRadiusDerivs<MockData>>();
	staticAssertHasRadiusDerivs::<GuaranteeRadiusDerivs<MockData>>();

	// Check at runtime too
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
	// Compile-time check guarantees on test combo 1
	type GuaranteedCombo1 = GuaranteeNormalsTangentsRadiusDerivsColors<MockData>;
	staticAssertHasNormals::<GuaranteedCombo1>();
	staticAssertHasTangents::<GuaranteedCombo1>();
	staticAssertHasRadii::<GuaranteedCombo1>();
	staticAssertHasRadiusDerivs::<GuaranteedCombo1>();
	staticAssertHasColors::<GuaranteedCombo1>();

	// Compile-time check guarantees on test combo 2
	type GuaranteedCombo2 = GuaranteeNormalsRadiiRadiusDerivsOrientations<MockData>;
	staticAssertHasNormals::<GuaranteedCombo2>();
	staticAssertHasRadii::<GuaranteedCombo2>();
	staticAssertHasRadiusDerivs::<GuaranteedCombo2>();
	staticAssertHasOrientations::<GuaranteedCombo2>();

	// Runtime-test combo 1
	let guaranteedCombo1 = GuaranteedCombo1::new(MockData::fullyPopulated());
	assert!(guaranteedCombo1.hasNormals());
	assert!(guaranteedCombo1.hasTangents());
	assert!(guaranteedCombo1.hasRadii());
	assert!(guaranteedCombo1.hasRadiusDerivs());
	assert!(guaranteedCombo1.hasColors());

	// Runtime-test combo 2
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
	assertPanics!(GuaranteeNormals::new(
		MockData::withFlags(/* missing normals: */false, true, true, true, true, true, true)
	));
	assertPanics!(GuaranteeTangents::new(
		MockData::withFlags(true, /* missing tangents: */false, true, true, true, true, true)
	));
	assertPanics!(GuaranteeRadii::new(
		MockData::withFlags(true, true, /* missing radii: */false, false, true, true, true)
	));
	assertPanics!(GuaranteeRadiusDerivs::new(
		MockData::withFlags(true, true, /* missing radii (for derivatives): */false, true, true, true, true)
	));
	assertPanics!(GuaranteeRadiusDerivs::new(
		MockData::withFlags(true, true, true, /* missing radius derivatives: */false, true, true, true)
	));
	assertPanics!(GuaranteeOrientations::new(
		MockData::withFlags(true, true, true, true, /* missing orientations: */false, true, true)
	));
	assertPanics!(GuaranteeScalings::new(
		MockData::withFlags(true, true, true, true, true, /* missing scaling vectors: */false, true)
	));
	assertPanics!(GuaranteeColors::new(
		MockData::withFlags(true, true, true, true, true, true, /* missing colors: */false)
	));
	assertPanics!(GuaranteeNormalsTangentsRadiusDerivsColors::new(
		MockData::withFlags(/* missing normals: */false, true, true, true, true, true, true))
	);
	assertPanics!(GuaranteeNormalsTangentsRadiusDerivsColors::new(
		MockData::withFlags(true, true, true, true, true, true, /* missing colors: */false))
	);
}
