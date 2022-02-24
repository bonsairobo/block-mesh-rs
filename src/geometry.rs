//! Voxel geometry and coordinate systems.
//!
//! `block-mesh` is designed to work for any 3-dimensional coordinate system.
//! Doing this requires a somewhat complicated interplay between the types in
//! this module. This documentation attempts to clarify what's going on.
//!
//! # Quads and Faces
//!
//! As most will know, a cube is composed of six **faces**.
//!
//! ```text
//!     O--------O
//!   / |      / |
//! O--------O   |
//! |   |    |   |
//! |   O----|---O
//! | /      | /
//! O--------O
//!
//! Fig 1: Cube
//! ```
//!
//! When rendering a cube to the screen, this will involve rendering six
//! **quads**. A quad has four **vertices**, each with a unique **UV** texture
//! coordinate and the same **normal** vector. The quad is rendered as two
//! triangles, typically as four vertices and six **indices**.
//!
//! ```text
//!            (U, V)                                         3    
//!              3                    3------> N            / ^     
//!            / |                  / |                   /   |     
//!  (U, V)  /   |                /   |                 2     |     
//!        2     |              2------> N                \   |     
//!        | \   |              | \   |                 2   \ |     
//!        |   \ |              |   \ |                 | \   1     
//!        |     1              |     1------> N        |   \       
//!        |   /   (U, V)       |   /                   |     1     
//!        | /                  | /                     |   /       
//!        0                    0------> N              V /         
//!     (U, V)                                          0           
//!
//! Fig 2: Quad with vertices [A, B, C, D].
//! - (left) UVs.
//! - (middle) Normals.
//! - (right) Two triangles rendered using indices [0, 1, 2, 3, 2, 1].
//! ```
//!
//! # Coordinate System
//!
//! Different domains (e.g., your game engine or 3D modeling application) use
//! different coordinate systems. Generating correct vertex positions, normals,
//! and UV coordinates requires knowing the specifics of the coordinate system.
//!
//! ```text
//!       +Y                 +Y                 -Y      
//!       | +Z               | -Z               | -Z    
//! -X____|/____+X     -X____|/____+X     +X____|/____-X
//!      /|                 /|                 /|       
//!    -Z |               +Z |               +Z |       
//!       -Y                 -Y                 +Y      
//!
//! Fig 3: Various coordinate systems.
//! - (left) Left-handed coordinate system with Y up.
//! - (middle) Right-handed coordinate system with Y up.
//! - (right) Right-handed coordinate system with Y down.
//! ```
//!
//! ## Handedness
//!
//! **Handedness** (also called chirality) is this weird property of 3D things.
//! If two 3D things have different handedness, then it is impossible to rotate
//! or translate one to look like the other; it's only possible if you mirror
//! it.
//!
//! Most often, in games and 3D modeling, this corresponds to which way the Z
//! axis grows. Often, +X is right, +Y is up, and the handedness determines
//! whether +Z is into or out of the screen.
//!
//! See wikipedia for more information on handedness:
//! - <https://en.wikipedia.org/wiki/Right-hand_rule>
//! - <https://en.wikipedia.org/wiki/Orientation_(vector_space)>
//!
//! ## Orientation
//!
//! The **orientation** of the coordinate system determines which way textures
//! are displayed on cube faces. Most often, the orientation is "Y up", meaning
//! that textures are displayed with the top of the image toward the +Y axis.
//!
//! # `{N, U, V}` Space
//!
//! "`{N, U, V}` space" is basically the **face-local coordinate space**
//! (**N**ormal, **U**, **V**).
//!
//! The notion of an `{N, U, V}` space is convenient because we can return
//! vertices, UVs, and indices in a consistent order regardless of the face. See
//! [`OrientedBlockFace::quad_corners`].
//!
//! # Putting It All Together
//!
//! The output of the `block-mesh` algorithms are [`UnorientedQuad`]s. These
//! simply specify the size and minimum `{X, Y, Z}` coordinate of the cube they
//! are a part of.
//!
//! In order to get vertex positions, UV coordinates, normals, and mesh indices
//! for an [`UnorientedQuad`], it must be paired with an [`OrientedBlockFace`],
//! which specifies the `{N, U, V}` --> `{X, Y, Z}` mapping and the sign of the
//! normal vector.
//!
//! Six [`OrientedBlockFace`] definitions combine to form a
//! [`QuadCoordinateConfig`], which also implicitly defines the coordinate
//! system.

mod axis;
mod face;
mod quad;

pub use axis::*;
pub use face::*;
pub use quad::*;

/// A configuration of XYZ --> NUV axis mappings and orientations of the cube
/// faces for a given coordinate system.
///
/// See the [`geometry` module documentation][crate::geometry] for more
/// information on `{N, U, V}` space.
#[derive(Clone)]
pub struct QuadCoordinateConfig {
    pub faces: [OrientedBlockFace; 6],

    /// For a given coordinate system, one of the two axes that isn't UP must be
    /// flipped in the U texel coordinate direction to avoid incorrect texture
    /// mirroring. For example, in a right-handed coordinate system with +Y
    /// pointing up, you should set `u_flip_face` to [`Axis::X`], because those
    /// faces need their U coordinates to be flipped relative to the other
    /// faces:
    ///
    /// ```text
    ///                         +X face  O           
    ///         +Z face                / |           
    ///     ^  O--------O          ^ O   |           
    ///     |  |        |          | |   |           
    ///  +V |  |        |       +V | |   O  ^        
    ///     |  |        |          | | /  /          
    ///     |  O--------O          | O  /                 
    ///      ------------ >        |  /  +U
    ///           +U                
    ///
    ///                    +Y      
    ///                    | -Z    
    ///              -X____|/____+X
    ///                   /|       
    ///                 +Z |       
    ///                    -Y      
    /// ```
    ///
    /// As you can see, for the +Z face, +U is toward positive X. But for the +X
    /// face, +U is towards **negative** Z.
    pub u_flip_face: Axis,
}

/// Coordinate configuration for a right-handed coordinate system with Y up.
///
/// ```text
///       +Y      
///       | -Z    
/// -X____|/____+X
///      /|       
///    +Z |       
///       -Y      
/// ```
pub const RIGHT_HANDED_Y_UP_CONFIG: QuadCoordinateConfig = QuadCoordinateConfig {
    // Y is always in the V direction when it's not the normal. When Y is the
    // normal, right-handedness determines that we must use Yzx permutations.
    faces: [
        OrientedBlockFace::new(-1, AxisPermutation::Xzy),
        OrientedBlockFace::new(-1, AxisPermutation::Yzx),
        OrientedBlockFace::new(-1, AxisPermutation::Zxy),
        OrientedBlockFace::new(1, AxisPermutation::Xzy),
        OrientedBlockFace::new(1, AxisPermutation::Yzx),
        OrientedBlockFace::new(1, AxisPermutation::Zxy),
    ],
    u_flip_face: Axis::X,
};
