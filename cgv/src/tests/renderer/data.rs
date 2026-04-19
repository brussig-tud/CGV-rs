//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::{self as cgv, *, renderer::prelude::*};



//////
//
// Helper functions
//

/*fn createTestData_nonindexed_interleaved () -> NonIndexedInterleavedPosNormalRadiusColor
{
	NonIndexedInterleavedPosNormalRadiusColor {
		data: vec![
			(/* pos: */glm::vec4(0.,0.,0.,1.), /* normal: */glm::vec4(0.,0.,1.,0.),
			 /* radius: */1., /* color: */cgv::RGBA::from_rgba_unmultiplied(1., 1., 1., 1.)),
			(/* pos: */glm::vec4(0.,0.,1.,1.), /* normal: */glm::vec4(0.,1.,0.,0.),
			 /* radius: */2., /* color: */ cgv::RGBA::from_rgba_unmultiplied(1., 1., 0., 1.)),
			(/* pos: */glm::vec4(0.,1.,0.,1.), /* normal: */glm::vec4(0.,1.,1.,0.),
			 /* radius: */3., /* color: */cgv::RGBA::from_rgba_unmultiplied(1., 0., 1., 1.)),
			(/* pos: */glm::vec4(0.,1.,1.,1.), /* normal: */glm::vec4(1.,0.,0.,0.),
			 /* radius: */4., /* color: */cgv::RGBA::from_rgba_unmultiplied(1., 0., 0., 1.))
		]
	}
}



//////
//
// Tests
//

#[test]
fn test_interleaving_nonindexed_random_access ()
{
	// Create interleaved test data
	let interleaved = createTestData_nonindexed_interleaved();
	assert_eq!(interleaved.num(), 4);

	// Check accessing positions
	assert_eq!(interleaved.pos(0), &glm::vec4(0.,0.,0.,1.));
	assert_eq!(interleaved.pos(1), &glm::vec4(0.,0.,1.,1.));
	assert_eq!(interleaved.pos(2), &glm::vec4(0.,1.,0.,1.));
	assert_eq!(interleaved.pos(3), &glm::vec4(0.,1.,1.,1.));

	// Check accessing normals
	assert_eq!(interleaved.normal(0), &glm::vec4(0.,0.,1.,0.));
	assert_eq!(interleaved.normal(1), &glm::vec4(0.,1.,0.,0.));
	assert_eq!(interleaved.normal(2), &glm::vec4(0.,1.,1.,0.));
	assert_eq!(interleaved.normal(3), &glm::vec4(1.,0.,0.,0.));

	// Check accessing radii
	assert_eq!(interleaved.radius(0), 1.);
	assert_eq!(interleaved.radius(1), 2.);
	assert_eq!(interleaved.radius(2), 3.);
	assert_eq!(interleaved.radius(3), 4.);

	// Check accessing colors
	assert_eq!(interleaved.color(0), &cgv::RGBA::from_rgba_unmultiplied(1., 1., 1., 1.));
	assert_eq!(interleaved.color(1), &cgv::RGBA::from_rgba_unmultiplied(1., 1., 0., 1.));
	assert_eq!(interleaved.color(2), &cgv::RGBA::from_rgba_unmultiplied(1., 0., 1., 1.));
	assert_eq!(interleaved.color(3), &cgv::RGBA::from_rgba_unmultiplied(1., 0., 0., 1.));
}

#[test]
fn test_interleaving_nonindexed_iterate ()
{
	// Create interleaved test data
	let interleaved = createTestData_nonindexed_interleaved();

	// Check iterating positions
	assert_eq!(interleaved.positions().count(), 4);
	let mut iter = interleaved.positions();
	assert_eq!(iter.len(), 4);
	assert_eq!(iter.next().unwrap(), glm::vec4(0.,0.,0.,1.));
	assert_eq!(iter.len(), 3);
	assert_eq!(iter.next().unwrap(), glm::vec4(0.,0.,1.,1.));
	assert_eq!(iter.len(), 2);
	assert_eq!(iter.next().unwrap(), glm::vec4(0.,1.,0.,1.));
	assert_eq!(iter.len(), 1);
	assert_eq!(iter.next().unwrap(), glm::vec4(0.,1.,1.,1.));
	assert_eq!(iter.len(), 0);
	assert_eq!(iter.next(), None);
}*/
