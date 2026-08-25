
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/prelude.slang";        // enable GLSL-style linear algebra mathematical operators
import "cgv/api/uniforms.slang";   // interface with managed uniforms handled by CGV-rs
import "cgv/media/geom.slang";     // access the pre-defined vertex structs
import "cgv/geom/billboard.slang"; // access the pre-defined billboards

// Local imports
import "lib/sdf.slang";
import "lib/glyph.slang";



//////
//
// Common uniforms
//

// Viewing parameters.
[vk::binding(0, 0)] // <- start at binding(0) of group(0)
ParameterBlock<cgv::api::Viewing> viewing;



//////
//
// Vertex shader
//

////
// Entry points

/// Our sole vertex shader entry point.
[shader("vertex")]
func vertexMain (int vertexID: SV_VertexID) -> cgv::media::VertexU<float>::GpuVertex {
	var vertex = cgv::geom::Billboards<float>::quadUCorner(vertexID).emit();
	vertex.pos = viewing.projModelview * vertex.pos;
	vertex.uv = vertex.uv*2-1;
	return vertex;
}



//////
//
// Fragment shader
//

////
// Specialization points

/// The Glyph type to draw.
extern struct Glyph: ex::IGlyph;


////
// Entry points

/// Our sole fragment shader entry point.
[shader("fragment")]
func fragmentMain (fsInput: cgv::media::VertexU<float>::GpuVertex) -> float4 {
	let glyph = Glyph::default(); // construct user glyph
	return glyph.colorAt(fsInput.uv); // shade according to user glyph
}
