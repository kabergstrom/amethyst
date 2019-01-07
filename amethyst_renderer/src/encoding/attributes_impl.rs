use super::attributes::{EncAttribute, EncVec4};

// Specific attributes

/// Shader attribute `vec4 tint`
pub struct TintAttribute;
impl EncAttribute for TintAttribute {
    const PROPERTY: &'static str = "tint";
    type EncodedType = EncVec4;
}

/// Shader attribute `vec4 pos`
pub struct Pos2DAttribute;
impl EncAttribute for Pos2DAttribute {
    const PROPERTY: &'static str = "pos";
    type EncodedType = EncVec4;
}

/// Shader attribute `vec4 dir_x`
pub struct DirXAttribute;
impl EncAttribute for DirXAttribute {
    const PROPERTY: &'static str = "dir_x";
    type EncodedType = EncVec4;
}

/// Shader attribute `vec4 dir_y`
pub struct DirYAttribute;
impl EncAttribute for DirYAttribute {
    const PROPERTY: &'static str = "dir_y";
    type EncodedType = EncVec4;
}
