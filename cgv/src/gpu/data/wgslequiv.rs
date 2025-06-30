
//////
//
// Imports
//

// GLM library
use glm;

// Local imports
use crate::*;



//////
//
// Traits
//

////
// HasWGSLEquivalent

/// The trait of having an equivalent data type in WebGPU *WGSL*.
pub trait HasWGSLEquivalent {
	/// Return the name of the type in *WGSL*.
	fn name () -> &'static str;
}


////
// Implementations

// --- scalars ------------------------------------------
impl HasWGSLEquivalent for bool {
	fn name () -> &'static str {
		"bool"
	}
}
impl HasWGSLEquivalent for i32 {
	fn name () -> &'static str {
		"i32"
	}
}
impl HasWGSLEquivalent for u32 {
	fn name () -> &'static str {
		"u32"
	}
}
// --- 2-vectors ----------------------------------------
impl HasWGSLEquivalent for glm::Vec2 {
	fn name () -> &'static str {
		"vec2f"
	}
}
impl HasWGSLEquivalent for glm::IVec2 {
	fn name () -> &'static str {
		"vec2i"
	}
}
impl HasWGSLEquivalent for glm::UVec2 {
	fn name () -> &'static str {
		"vec2u"
	}
}
impl HasWGSLEquivalent for glm::BVec2 {
	fn name () -> &'static str {
		"vec2<bool>"
	}
}
// --- 3-vectors ----------------------------------------
impl HasWGSLEquivalent for glm::Vec3 {
	fn name () -> &'static str {
		"vec3f"
	}
}
impl HasWGSLEquivalent for glm::IVec3 {
	fn name () -> &'static str {
		"vec3i"
	}
}
impl HasWGSLEquivalent for glm::UVec3 {
	fn name () -> &'static str {
		"vec3u"
	}
}
impl HasWGSLEquivalent for glm::BVec3 {
	fn name () -> &'static str {
		"vec3<bool>"
	}
}
// --- 4-vectors ----------------------------------------
impl HasWGSLEquivalent for glm::Vec4 {
	fn name () -> &'static str {
		"vec4f"
	}
}
impl HasWGSLEquivalent for glm::IVec4 {
	fn name () -> &'static str {
		"vec4i"
	}
}
impl HasWGSLEquivalent for glm::UVec4 {
	fn name () -> &'static str {
		"vec4u"
	}
}
impl HasWGSLEquivalent for glm::BVec4 {
	fn name () -> &'static str {
		"vec4<bool>"
	}
}
// --- 2-column matrices --------------------------------
impl HasWGSLEquivalent for glm::Mat2 {
	fn name () -> &'static str {
		"mat2x2f"
	}
}
impl HasWGSLEquivalent for glm::Mat2x3 {
	fn name () -> &'static str {
		"mat2x3f"
	}
}
impl HasWGSLEquivalent for glm::Mat2x4 {
	fn name () -> &'static str {
		"mat2x4f"
	}
}
// --- 3-column matrices --------------------------------
impl HasWGSLEquivalent for glm::Mat3x2 {
	fn name () -> &'static str {
		"mat3x2f"
	}
}
impl HasWGSLEquivalent for glm::Mat3 {
	fn name () -> &'static str {
		"mat3x3f"
	}
}
impl HasWGSLEquivalent for glm::Mat3x4 {
	fn name () -> &'static str {
		"mat3x4f"
	}
}
// --- 4-column matrices --------------------------------
impl HasWGSLEquivalent for glm::Mat4x2 {
	fn name () -> &'static str {
		"mat4x2f"
	}
}
impl HasWGSLEquivalent for glm::Mat4x3 {
	fn name () -> &'static str {
		"mat4x3f"
	}
}
impl HasWGSLEquivalent for glm::Mat4 {
	fn name () -> &'static str {
		"mat4x4f"
	}
}
