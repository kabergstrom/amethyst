use super::properties::{EncProperty, EncVec4};

/// Shader attribute `vec4 tint`
pub struct TintProperty;
impl EncProperty for TintProperty {
    const PROPERTY: &'static str = "tint";
    type EncodedType = EncVec4;
    fn fallback() -> [f32; 4] {
        [1.0, 1.0, 1.0, 1.0]
    }
}

/// Shader attribute `vec4 pos`
pub struct Pos2DProperty;
impl EncProperty for Pos2DProperty {
    const PROPERTY: &'static str = "pos";
    type EncodedType = EncVec4;
    fn fallback() -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }
}

/// Shader attribute `vec4 dir_x`
pub struct DirXProperty;
impl EncProperty for DirXProperty {
    const PROPERTY: &'static str = "dir_x";
    type EncodedType = EncVec4;
    fn fallback() -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }
}

/// Shader attribute `vec4 dir_y`
pub struct DirYProperty;
impl EncProperty for DirYProperty {
    const PROPERTY: &'static str = "dir_y";
    type EncodedType = EncVec4;
    fn fallback() -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }
}
