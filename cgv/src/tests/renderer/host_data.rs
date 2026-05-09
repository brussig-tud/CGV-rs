//////
//
// Imports
//

// Standard library
use std::{marker::PhantomData, panic::catch_unwind};

// Local imports
use crate::{self as cgv, *, renderer::{*, data::{*, host::*}}};



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

impl HostData for MockData
{
	type PosIterator<'data> = std::iter::Copied<std::slice::Iter<'data, glm::Vec3>>;

	fn num (&self) -> u32 { self.positions.len() as u32 }
	fn positions (&self) -> Self::PosIterator<'_> { self.positions.iter().copied() }
	fn pos (&self, index: u32) -> glm::Vec3 { self.positions[index as usize] }
	fn topology(&self) -> wgpu::PrimitiveTopology { wgpu::PrimitiveTopology::PointList }
}
impl host::Indexed for MockData
{
	type IndexIterator<'data> = std::iter::Copied<std::slice::Iter<'data, u32>>;

	fn numIndices (&self) -> u32 { self.indices.len() as u32 }
	fn indices (&self) -> Self::IndexIterator<'_> { self.indices.iter().copied() }
	fn index (&self, index: u32) -> u32 { self.indices[index as usize] }
}
impl host::Interleaved for MockData {}

impl host::CanHaveNormals for MockData
{
	type NormalIterator<'data> = std::iter::Copied<std::slice::Iter<'data, glm::Vec3>>;

	fn hasNormals (&self) -> bool { self.normals.is_some() }
	fn normals (&self) -> Self::NormalIterator<'_> {
		self.normals.as_ref().expect("should have normals").iter().copied()
	}
	fn normal (&self, index: u32) -> glm::Vec3 {
		self.normals.as_ref().expect("should have normals")[index as usize]
	}
}
impl host::CanHaveTangents for MockData
{
	type TangentIterator<'data> = std::iter::Copied<std::slice::Iter<'data, glm::Vec3>>;

	fn hasTangents (&self) -> bool { self.tangents.is_some() }
	fn tangents (&self) -> Self::TangentIterator<'_> {
		self.tangents.as_ref().expect("should have tangents").iter().copied()
	}
	fn tangent (&self, index: u32) -> glm::Vec3 {
		self.tangents.as_ref().expect("should have tangents")[index as usize]
	}
}
impl host::CanHaveRadii for MockData
{
	type RadiusIterator<'data> = std::iter::Copied<std::slice::Iter<'data, f32>>;

	fn hasRadii (&self) -> bool { self.radii.is_some() }
	fn radii (&self) -> Self::RadiusIterator<'_> {
		self.radii.as_ref().expect("should have radii").iter().copied()
	}
	fn radius (&self, index: u32) -> f32 {
		self.radii.as_ref().expect("should have radii")[index as usize]
	}
}
impl host::CanHaveRadiusDerivs for MockData
{
	type RadiusDerivIterator<'data> = std::iter::Copied<std::slice::Iter<'data, f32>>;

	fn hasRadiusDerivs (&self) -> bool { self.radiusDerivs.is_some() }
	fn radiusDerivs (&self) -> Self::RadiusDerivIterator<'_> {
		self.radiusDerivs.as_ref().expect("should have radius derivatives").iter().copied()
	}
	fn radiusDeriv (&self, index: u32) -> f32 {
		self.radiusDerivs.as_ref().expect("should have radius derivatives")[index as usize]
	}
}
impl host::CanHaveOrientations for MockData
{
	type OrientationIterator<'data> = std::iter::Copied<std::slice::Iter<'data, glm::Quat>>;

	fn hasOrientations (&self) -> bool { self.orientations.is_some() }
	fn orientations (&self) -> Self::OrientationIterator<'_> {
		self.orientations.as_ref().expect("should have orientations").iter().copied()
	}
	fn orientation (&self, index: u32) -> glm::Quat {
		self.orientations.as_ref().expect("should have orientations")[index as usize]
	}
}
impl host::CanHaveScalings for MockData
{
	type ScaleIterator<'data> = std::iter::Copied<std::slice::Iter<'data, glm::Vec3>>;

	fn hasScalings (&self) -> bool { self.scalings.is_some() }
	fn scalings (&self) -> Self::ScaleIterator<'_> {
		self.scalings.as_ref().expect("should have scaling vectors").iter().copied()
	}
	fn scaling (&self, index: u32) -> glm::Vec3 {
		self.scalings.as_ref().expect("should have scaling vectors")[index as usize]
	}
}
impl host::CanHaveColors for MockData
{
	type ColorIterator<'data> = std::iter::Copied<std::slice::Iter<'data, cgv::RGBA>>;

	fn hasColors (&self) -> bool { self.colors.is_some() }
	fn colors (&self) -> Self::ColorIterator<'_> {
		self.colors.as_ref().expect("should have colors").iter().copied()
	}
	fn color (&self, index: u32) -> cgv::RGBA {
		self.colors.as_ref().expect("should have colors")[index as usize]
	}
}



//////
//
// Functions
//

/// For testing at compile time that the generic argument is a type that represents indexed render data.
fn staticAssertIndexed<T: host::Indexed> () {}

/// For testing at compile time that the generic argument is a type that represents interleaved render data.
fn staticAssertInterleaved<T: host::Interleaved> () {}

/// For testing at compile time that the generic argument is a type that guarantees normals.
fn staticAssertHasNormals<T: host::HasNormals> () {}

/// For testing at compile time that the generic argument is a type that guarantees tangents.
fn staticAssertHasTangents<T: host::HasTangents> () {}

/// For testing at compile time that the generic argument is a type that guarantees radii.
fn staticAssertHasRadii<T: host::HasRadii> () {}

/// For testing at compile time that the generic argument is a type that guarantees radius derivatives.
fn staticAssertHasRadiusDerivs<T: host::HasRadiusDerivs> () {}

/// For testing at compile time that the generic argument is a type that guarantees orientations.
fn staticAssertHasOrientations<T: host::HasOrientations> () {}

/// For testing at compile time that the generic argument is a type that guarantees scalings.
fn staticAssertHasScalings<T: host::HasScalings> () {}

/// For testing at compile time that the generic argument is a type that guarantees colors.
fn staticAssertHasColors<T: host::HasColors> () {}



//////
//
// Tests
//

#[test]
fn test_guaranteeWrappers_allTypeAliasesResolve ()
{
	// Assert helper
	macro_rules! assertAliasesExist {
		($($alias:ident),+ $(,)?) => {{
			$(let _: PhantomData<$alias<MockData>> = PhantomData;)+
		}};
	}

	// Single-attribute combinations
	assertAliasesExist!(
		GuaranteeNormals, GuaranteeTangents, GuaranteeRadii, GuaranteeRadiusDerivs, GuaranteeOrientations,
		GuaranteeScalings, GuaranteeColors
	);

	// 2-attribute combinations
	assertAliasesExist!(
		GuaranteeNormalsTangents, GuaranteeNormalsRadii, GuaranteeNormalsRadiusDerivs, GuaranteeNormalsOrientations,
		GuaranteeNormalsScalings, GuaranteeNormalsColors, GuaranteeTangentsRadii, GuaranteeTangentsRadiusDerivs,
		GuaranteeTangentsOrientations, GuaranteeTangentsScalings, GuaranteeTangentsColors, GuaranteeRadiiRadiusDerivs,
		GuaranteeRadiiOrientations, GuaranteeRadiiScalings, GuaranteeRadiiColors, GuaranteeRadiusDerivsOrientations,
		GuaranteeRadiusDerivsScalings, GuaranteeRadiusDerivsColors, GuaranteeOrientationsScalings,
		GuaranteeOrientationsColors, GuaranteeScalingsColors
	);

	// 3-attribute combinations
	assertAliasesExist!(
		GuaranteeNormalsTangentsRadii, GuaranteeNormalsTangentsRadiusDerivs, GuaranteeNormalsTangentsOrientations,
		GuaranteeNormalsTangentsScalings, GuaranteeNormalsTangentsColors, GuaranteeNormalsRadiiRadiusDerivs,
		GuaranteeNormalsRadiiOrientations, GuaranteeNormalsRadiiScalings, GuaranteeNormalsRadiiColors,
		GuaranteeNormalsRadiusDerivsOrientations, GuaranteeNormalsRadiusDerivsScalings,
		GuaranteeNormalsRadiusDerivsColors, GuaranteeNormalsOrientationsScalings, GuaranteeNormalsOrientationsColors,
		GuaranteeNormalsScalingsColors, GuaranteeTangentsRadiiRadiusDerivs, GuaranteeTangentsRadiiOrientations,
		GuaranteeTangentsRadiiScalings, GuaranteeTangentsRadiiColors, GuaranteeTangentsRadiusDerivsOrientations,
		GuaranteeTangentsRadiusDerivsScalings, GuaranteeTangentsRadiusDerivsColors,
		GuaranteeTangentsOrientationsScalings, GuaranteeTangentsOrientationsColors, GuaranteeTangentsScalingsColors,
		GuaranteeRadiiRadiusDerivsOrientations, GuaranteeRadiiRadiusDerivsScalings, GuaranteeRadiiRadiusDerivsColors,
		GuaranteeRadiiOrientationsScalings, GuaranteeRadiiOrientationsColors, GuaranteeRadiiScalingsColors,
		GuaranteeRadiusDerivsOrientationsScalings, GuaranteeRadiusDerivsOrientationsColors,
		GuaranteeRadiusDerivsScalingsColors, GuaranteeOrientationsScalingsColors
	);

	// 4-attribute combinations
	assertAliasesExist!(
		GuaranteeNormalsTangentsRadiiRadiusDerivs, GuaranteeNormalsTangentsRadiiOrientations,
		GuaranteeNormalsTangentsRadiiScalings, GuaranteeNormalsTangentsRadiiColors,
		GuaranteeNormalsTangentsRadiusDerivsOrientations, GuaranteeNormalsTangentsRadiusDerivsScalings,
		GuaranteeNormalsTangentsRadiusDerivsColors, GuaranteeNormalsTangentsOrientationsScalings,
		GuaranteeNormalsTangentsOrientationsColors, GuaranteeNormalsTangentsScalingsColors,
		GuaranteeNormalsRadiiRadiusDerivsOrientations, GuaranteeNormalsRadiiRadiusDerivsScalings,
		GuaranteeNormalsRadiiRadiusDerivsColors, GuaranteeNormalsRadiiOrientationsScalings,
		GuaranteeNormalsRadiiOrientationsColors, GuaranteeNormalsRadiiScalingsColors,
		GuaranteeNormalsRadiusDerivsOrientationsScalings, GuaranteeNormalsRadiusDerivsOrientationsColors,
		GuaranteeNormalsRadiusDerivsScalingsColors, GuaranteeNormalsOrientationsScalingsColors,
		GuaranteeTangentsRadiiRadiusDerivsOrientations, GuaranteeTangentsRadiiRadiusDerivsScalings,
		GuaranteeTangentsRadiiRadiusDerivsColors, GuaranteeTangentsRadiiOrientationsScalings,
		GuaranteeTangentsRadiiOrientationsColors, GuaranteeTangentsRadiiScalingsColors,
		GuaranteeTangentsRadiusDerivsOrientationsScalings, GuaranteeTangentsRadiusDerivsOrientationsColors,
		GuaranteeTangentsRadiusDerivsScalingsColors, GuaranteeTangentsOrientationsScalingsColors,
		GuaranteeRadiiRadiusDerivsOrientationsScalings, GuaranteeRadiiRadiusDerivsOrientationsColors,
		GuaranteeRadiiRadiusDerivsScalingsColors, GuaranteeRadiiOrientationsScalingsColors,
		GuaranteeRadiusDerivsOrientationsScalingsColors
	);

	// 5-attribute combinations
	assertAliasesExist!(
		GuaranteeRadiiRadiusDerivsOrientationsScalingsColors, GuaranteeTangentsRadiusDerivsOrientationsScalingsColors,
		GuaranteeTangentsRadiiOrientationsScalingsColors, GuaranteeTangentsRadiiRadiusDerivsScalingsColors,
		GuaranteeTangentsRadiiRadiusDerivsOrientationsColors, GuaranteeTangentsRadiiRadiusDerivsOrientationsScalings,
		GuaranteeNormalsRadiusDerivsOrientationsScalingsColors, GuaranteeNormalsRadiiOrientationsScalingsColors,
		GuaranteeNormalsRadiiRadiusDerivsScalingsColors, GuaranteeNormalsRadiiRadiusDerivsOrientationsColors,
		GuaranteeNormalsRadiiRadiusDerivsOrientationsScalings, GuaranteeNormalsTangentsOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiusDerivsScalingsColors, GuaranteeNormalsTangentsRadiusDerivsOrientationsColors,
		GuaranteeNormalsTangentsRadiusDerivsOrientationsScalings, GuaranteeNormalsTangentsRadiiScalingsColors,
		GuaranteeNormalsTangentsRadiiOrientationsColors, GuaranteeNormalsTangentsRadiiOrientationsScalings,
		GuaranteeNormalsTangentsRadiiRadiusDerivsColors, GuaranteeNormalsTangentsRadiiRadiusDerivsScalings,
		GuaranteeNormalsTangentsRadiiRadiusDerivsOrientations
	);

	// 6-attribute combinations
	assertAliasesExist!(
		GuaranteeTangentsRadiiRadiusDerivsOrientationsScalingsColors,
		GuaranteeNormalsRadiiRadiusDerivsOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiusDerivsOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiiOrientationsScalingsColors,
		GuaranteeNormalsTangentsRadiiRadiusDerivsScalingsColors,
		GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsColors,
		GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsScalings
	);

	// All-attribute combination
	assertAliasesExist!(GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsScalingsColors);
}

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
	assert_eq!(guaranteed.pos(0), glm::vec3(0., 0., 0.));
	assert_eq!(guaranteed.normal(1), glm::vec3(0., 1., 0.));
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
	assert_eq!(indices.next().unwrap(), 0);
	assert_eq!(indices.next().unwrap(), 1);
	assert_eq!(indices.next().unwrap(), 0);
	assert_eq!(indices.next(), None);
	let mut normals = guaranteed.normals();
	assert_eq!(normals.next(), Some(glm::vec3(0., 0., 1.)));
	assert_eq!(normals.next(), Some(glm::vec3(0., 1., 0.)));
	assert_eq!(normals.next(), None);
}

#[test]
fn test_guaranteeWrappers_selectCombosGuaranteeAttributes ()
{
	// Compile-time check guarantees on test combo 1
	type GuaranteedCombo1 = GuaranteeNormalsTangentsRadiusDerivsColors<MockData>;
	staticAssertHasNormals::<GuaranteedCombo1>();
	staticAssertHasTangents::<GuaranteedCombo1>();
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
fn test_guaranteeWrappers_selectCombosPreserveExistingGuarantees ()
{
	// Compile-time check existing guarantees get propagated on test combo 1
	type GuaranteedBaseCombo1 = GuaranteeOrientationsScalings<MockData>;
	type GuaranteedWrappedCombo1 = GuaranteeNormalsTangentsRadiusDerivsColors<GuaranteedBaseCombo1>;
	staticAssertHasNormals::<GuaranteedWrappedCombo1>();
	staticAssertHasTangents::<GuaranteedWrappedCombo1>();
	staticAssertHasRadiusDerivs::<GuaranteedWrappedCombo1>();
	staticAssertHasOrientations::<GuaranteedWrappedCombo1>(); // <- should have been propagated
	staticAssertHasScalings::<GuaranteedWrappedCombo1>();     // <- should have been propagated
	staticAssertHasColors::<GuaranteedWrappedCombo1>();

	// Compile-time check existing guarantees get propagated on test combo 2
	type GuaranteedBaseCombo2 = GuaranteeTangentsRadiiColors<MockData>;
	type GuaranteedWrappedCombo2 = GuaranteeNormalsRadiusDerivsScalings<GuaranteedBaseCombo2>;
	staticAssertHasNormals::<GuaranteedWrappedCombo2>();
	staticAssertHasTangents::<GuaranteedWrappedCombo2>();     // <- should have been propagated
	staticAssertHasRadii::<GuaranteedWrappedCombo2>();        // <- should have been propagated
	staticAssertHasRadiusDerivs::<GuaranteedWrappedCombo2>();
	staticAssertHasScalings::<GuaranteedWrappedCombo2>();
	staticAssertHasColors::<GuaranteedWrappedCombo2>();       // <- should have been propagated

	// Runtime-test combo 1
	let guaranteedWrappedCombo1 = GuaranteedWrappedCombo1::new(
		GuaranteedBaseCombo1::new(MockData::withFlags(
			true, true, false, true, true, true,
			true
		))
	);
	assert!(guaranteedWrappedCombo1.hasNormals());
	assert!(guaranteedWrappedCombo1.hasTangents());
	assert!(guaranteedWrappedCombo1.hasRadiusDerivs());
	assert!(guaranteedWrappedCombo1.hasOrientations()); // <- should have been propagated
	assert!(guaranteedWrappedCombo1.hasScalings());     // <- should have been propagated
	assert!(guaranteedWrappedCombo1.hasColors());

	// Runtime-test combo 2
	let guaranteedWrappedCombo2 = GuaranteedWrappedCombo2::new(
		GuaranteedBaseCombo2::new(MockData::withFlags(
			true, true, true, true, false, true,
			true
		))
	);
	assert!(guaranteedWrappedCombo2.hasNormals());
	assert!(guaranteedWrappedCombo2.hasTangents());     // <- should have been propagated
	assert!(guaranteedWrappedCombo2.hasRadii());        // <- should have been propagated
	assert!(guaranteedWrappedCombo2.hasRadiusDerivs());
	assert!(guaranteedWrappedCombo2.hasScalings());
	assert!(guaranteedWrappedCombo2.hasColors());       // <- should have been propagated
}

#[test]
fn test_guaranteeWrappers_constructorsRejectMissingAttributes ()
{
	// Assert helper
	macro_rules! assertPanics {
		($expression:expr) => {
			assert!(catch_unwind(|| $expression).is_err());
		};
	}

	// Check that constructors correctly reject
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
