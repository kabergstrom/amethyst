use crate::{Mesh, Texture};
use amethyst_assets::Handle;

// TODO: support more types, like boolean vectors or any scalars
// TODO: address alignment

/// An enum of all supported uniform or attribute types
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum ShaderInput {
    /// A vector of 4 single precision floats
    Vec4,
    /// A vector of 2 single precision floats
    Vec2,
    /// A matrix of 4x4 single precision floats
    Mat4x4,
    /// A vector of 4 signed integers
    Vec4i,
    /// A vector of 2 signed integers
    Vec2i,
    /// A vector of 4x4 signed integers
    Mat4x4i,
    /// A vector of 4 unsigned integers
    Vec4u,
    /// A vector of 2 unsigned integers
    Vec2u,
    /// A matrix of 4x4 unsigned integers
    Mat4x4u,
    /// A 2d texture
    Texture,
    /// A mesh (list of vertex attributes)
    Mesh,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum InputKind {
    Uniform,
    Attribute,
}

/// Represents a single shader uniform or attribute input.
pub trait ShaderInputType {
    /// A type of encoded data
    const TY: ShaderInput;
    /// Binding destination
    const KIND: InputKind;
    /// Type level data representation that's produced in the encoding phase by `StreamEncoder`
    type Repr;
}

pub struct EncVec4;
impl ShaderInputType for EncVec4 {
    const TY: ShaderInput = ShaderInput::Vec4;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [f32; 4];
}

pub struct EncVec2;
impl ShaderInputType for EncVec2 {
    const TY: ShaderInput = ShaderInput::Vec2;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [f32; 2];
}

pub struct EncMat4x4;
impl ShaderInputType for EncMat4x4 {
    const TY: ShaderInput = ShaderInput::Mat4x4;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [[f32; 4]; 4];
}

pub struct EncVec4i;
impl ShaderInputType for EncVec4i {
    const TY: ShaderInput = ShaderInput::Vec4i;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [i32; 4];
}

pub struct EncVec2i;
impl ShaderInputType for EncVec2i {
    const TY: ShaderInput = ShaderInput::Vec2i;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [i32; 2];
}

pub struct EncMat4x4i;
impl ShaderInputType for EncMat4x4i {
    const TY: ShaderInput = ShaderInput::Mat4x4i;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [[i32; 4]; 4];
}

pub struct EncVec4u;
impl ShaderInputType for EncVec4u {
    const TY: ShaderInput = ShaderInput::Vec4u;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [u32; 4];
}

pub struct EncVec2u;
impl ShaderInputType for EncVec2u {
    const TY: ShaderInput = ShaderInput::Vec2u;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [u32; 2];
}

pub struct EncMat4x4u;
impl ShaderInputType for EncMat4x4u {
    const TY: ShaderInput = ShaderInput::Mat4x4u;
    const KIND: InputKind = InputKind::Uniform;
    type Repr = [[u32; 4]; 4];
}

pub struct EncTexture;
impl ShaderInputType for EncTexture {
    const TY: ShaderInput = ShaderInput::Texture;
    const KIND: InputKind = InputKind::Attribute;
    type Repr = Handle<Texture>;
}

pub struct EncMesh;
impl ShaderInputType for EncMesh {
    const TY: ShaderInput = ShaderInput::Mesh;
    const KIND: InputKind = InputKind::Attribute;
    type Repr = Handle<Mesh>;
}

/// Combined type that maps a shader attribute layout (a tuple of `ShaderInputType`s)
/// into the corresponding output of an encoder.
pub trait EncodingValue {
    type Value;
    type OptValue;
    fn resolve(optional: Self::OptValue, fallback: Self::Value) -> Self::Value;
}

impl<A: ShaderInputType> EncodingValue for A {
    type Value = A::Repr;
    type OptValue = Option<A::Repr>;
    fn resolve(optional: Self::OptValue, fallback: Self::Value) -> Self::Value {
        optional.unwrap_or(fallback)
    }
}

macro_rules! impl_encoding_value {
    ($($from:ident $idx:tt),*) => {
        impl<$($from,)*> EncodingValue for ($($from),*,)
            where $($from: EncodingValue),*,
        {
            type Value = ($($from::Value),*,);
            type OptValue = ($($from::OptValue),*,);

            #[allow(non_snake_case)]
            fn resolve(optional: Self::OptValue, fallback: Self::Value) -> Self::Value {
                (
                    $(<$from as EncodingValue>::resolve(optional.$idx, fallback.$idx)),*,
                )
            }
        }
    }
}

impl_encoding_value! {A 0}
impl_encoding_value! {A 0, B 1}
impl_encoding_value! {A 0, B 1, C 2}
impl_encoding_value! {A 0, B 1, C 2, D 3}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6 }
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13}
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14 }
impl_encoding_value! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14 , P 15}

/// A compile-time definition of a shader property to encode.
///
/// It is defined by a combination of `ShaderInputType` and a property name.
/// Allows to target properties like `mat4 model` or `vec2 pos`;
pub trait EncProperty {
    /// Name of property used in the shader source code
    const PROPERTY: &'static str;
    /// Type of property used in the shader source code
    type EncodedType: ShaderInputType + EncodingValue;

    /// Get all runtime shader properties for this type of encodable attribute
    fn prop() -> EncodedProp {
        (Self::EncodedType::TY, Self::PROPERTY)
    }
    fn fallback() -> <Self::EncodedType as EncodingValue>::Value;
}

/// A runtime representation for unique encodable shader input property
pub type EncodedProp = (ShaderInput, &'static str);

/// A compile-time list of `EncProperty`s.
pub trait EncProperties {
    /// A combined compile-time value representing all encoded properties
    type EncodedType: EncodingValue;
    /// Retreive a vec of associated (type name, property, byte offset, byte size) tuples at runtime
    fn get_props() -> Vec<EncodedProp>;

    fn fallback() -> <Self::EncodedType as EncodingValue>::Value;
    fn resolve(
        optional: <Self::EncodedType as EncodingValue>::OptValue,
    ) -> <Self::EncodedType as EncodingValue>::Value {
        <Self::EncodedType as EncodingValue>::resolve(optional, Self::fallback())
    }
}

impl<A: EncProperty> EncProperties for A {
    type EncodedType = A::EncodedType;

    fn get_props() -> Vec<EncodedProp> {
        vec![A::prop()]
    }

    fn fallback() -> <Self::EncodedType as EncodingValue>::Value {
        A::fallback()
    }
}
impl<A: EncProperties, B: EncProperties> EncProperties for (A, B) {
    type EncodedType = (A::EncodedType, B::EncodedType);
    fn get_props() -> Vec<EncodedProp> {
        let mut vec = A::get_props();
        vec.extend(B::get_props());
        vec
    }
    fn fallback() -> <Self::EncodedType as EncodingValue>::Value {
        (A::fallback(), B::fallback())
    }
}

impl<A: EncProperties, B: EncProperties, C: EncProperties> EncProperties for (A, B, C) {
    type EncodedType = (A::EncodedType, B::EncodedType, C::EncodedType);
    fn get_props() -> Vec<EncodedProp> {
        let mut vec = A::get_props();
        vec.extend(B::get_props());
        vec.extend(C::get_props());
        vec
    }
    fn fallback() -> <Self::EncodedType as EncodingValue>::Value {
        (A::fallback(), B::fallback(), C::fallback())
    }
}

// TODO: more tuple implementations in a macro
