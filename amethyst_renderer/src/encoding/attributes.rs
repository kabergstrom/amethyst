/// Represents shader input attribute.
/// Represents a spir-v side type and rust-side representation.

// TODO: support more types, like boolean vectors or any scalars
// TODO: address alignment
pub enum ShaderInput {
    Vec4,
    Vec2,
    Mat4x4,
    Vec4i,
    Vec2i,
    Mat4x4i,
    Vec4u,
    Vec2u,
    Mat4x4u,
}

pub trait ShaderInputType {
    const Ty: ShaderInput;
    type Repr;
}

pub struct EncVec4;
impl ShaderInputType for EncVec4 {
    const Ty: ShaderInput = ShaderInput::Vec4;
    type Repr = [f32; 4];
}

pub struct EncVec2;
impl ShaderInputType for EncVec2 {
    const Ty: ShaderInput = ShaderInput::Vec2;
    type Repr = [f32; 2];
}

pub struct EncMat4x4;
impl ShaderInputType for EncMat4x4 {
    const Ty: ShaderInput = ShaderInput::Mat4x4;
    type Repr = [[f32; 4]; 4];
}

pub struct EncVec4i;
impl ShaderInputType for EncVec4i {
    const Ty: ShaderInput = ShaderInput::Vec4i;
    type Repr = [i32; 4];
}

pub struct EncVec2i;
impl ShaderInputType for EncVec2i {
    const Ty: ShaderInput = ShaderInput::Vec2i;
    type Repr = [i32; 2];
}

pub struct EncMat4x4i;
impl ShaderInputType for EncMat4x4i {
    const Ty: ShaderInput = ShaderInput::Mat4x4i;
    type Repr = [[i32; 4]; 4];
}

pub struct EncVec4u;
impl ShaderInputType for EncVec4u {
    const Ty: ShaderInput = ShaderInput::Vec4u;
    type Repr = [u32; 4];
}

pub struct EncVec2u;
impl ShaderInputType for EncVec2u {
    const Ty: ShaderInput = ShaderInput::Vec2u;
    type Repr = [u32; 2];
}

pub struct EncMat4x4u;
impl ShaderInputType for EncMat4x4u {
    const Ty: ShaderInput = ShaderInput::Mat4x4u;
    type Repr = [[u32; 4]; 4];
}

/// Combined type that maps a shader attribute layout (a tuple of `ShaderInputType`s)
/// into the corresponding output of an encoder.
pub trait EncodingValue {
    type Value;
}

impl<A: ShaderInputType> EncodingValue for A {
    type Value = A::Repr;
}

impl<A: EncodingValue, B: EncodingValue> EncodingValue for (A, B) {
    type Value = (A::Value, B::Value);
}

impl<A: EncodingValue, B: EncodingValue, C: EncodingValue> EncodingValue for (A, B, C) {
    type Value = (A::Value, B::Value, C::Value);
}

// TODO: more tuple implementations in a macro

/// A compile-time definition of a shader attribute to encode.
///
/// It is defined by a combination of `ShaderInputType` and a property name.
/// Allows to target attributes like `mat4 model` or `vec2 pos`;
pub trait EncAttribute {
    const PROPERTY: &'static str;
    type EncodedType: ShaderInputType + EncodingValue;
}

/// A compile-time list of `EncAttribute`s.
pub trait EncAttributes {
    type EncodedType: EncodingValue;
    /// Retreive a vec of associated (type name, property, byte offset, byte size) tuples at runtime
    fn get_props() -> Vec<(ShaderInput, &'static str)>;
}

impl<A: EncAttribute> EncAttributes for A {
    type EncodedType = A::EncodedType;
    fn get_props() -> Vec<(ShaderInput, &'static str)> {
        vec![(A::EncodedType::Ty, A::PROPERTY)]
    }
}
impl<A: EncAttributes, B: EncAttributes> EncAttributes for (A, B) {
    type EncodedType = (A::EncodedType, B::EncodedType);
    fn get_props() -> Vec<(ShaderInput, &'static str)> {
        let mut vec = A::get_props();
        vec.extend(B::get_props());
        vec
    }
}

impl<A: EncAttributes, B: EncAttributes, C: EncAttributes> EncAttributes for (A, B, C) {
    type EncodedType = (A::EncodedType, B::EncodedType, C::EncodedType);
    fn get_props() -> Vec<(ShaderInput, &'static str)> {
        let mut vec = A::get_props();
        vec.extend(B::get_props());
        vec.extend(C::get_props());
        vec
    }
}

// TODO: more tuple implementations in a macro
