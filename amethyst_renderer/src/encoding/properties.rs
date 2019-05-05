// use crate::{Mesh, Texture};
// use amethyst_assets::Handle;
use std::iter::{once, Chain, Once};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum InputKind {
    Uniform,
    Attribute,
}

/// Trait that provides a conversion of encoding result into a byte slice.
///
/// Implementer must guarantee that the type's memory layout is strictly defined.
/// Usually that means that the implementing struct needs a `#[repr(C)]`
/// and all types of it's fields are also defined in that way.
pub unsafe trait ResolvedEncoding: Sized {
    /// Convert a structure with known layout into a slice of bytes.
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const Self as *const u8;

        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<Self>()) }
    }
}

unsafe impl ResolvedEncoding for () {}
unsafe impl ResolvedEncoding for u8 {}
unsafe impl ResolvedEncoding for u16 {}
unsafe impl ResolvedEncoding for u32 {}
unsafe impl ResolvedEncoding for u64 {}
unsafe impl ResolvedEncoding for i8 {}
unsafe impl ResolvedEncoding for i16 {}
unsafe impl ResolvedEncoding for i32 {}
unsafe impl ResolvedEncoding for i64 {}
unsafe impl ResolvedEncoding for f32 {}
unsafe impl ResolvedEncoding for f64 {}
unsafe impl ResolvedEncoding for bool {}
unsafe impl ResolvedEncoding for char {}
unsafe impl<T: ResolvedEncoding> ResolvedEncoding for [T; 2] {}
unsafe impl<T: ResolvedEncoding> ResolvedEncoding for [T; 4] {}
unsafe impl<T: ResolvedEncoding> ResolvedEncoding for [T; 8] {}

/// Represents a single shader uniform or attribute input.
pub trait ShaderInputType {
    /// A type of encoded data
    const TY: ShaderInput;
    /// Binding destination
    const KIND: InputKind;
    /// Type level data representation that's produced in the encoding phase by `StreamEncoder`.
    /// Note that this type must have a strictly defined layout that matches what GPU will expect.
    type Repr: ResolvedEncoding;

    /// Retreive the size of type in uniform buffer.
    /// Returns 0 for non-uniform data.
    fn ubo_size() -> usize {
        if Self::KIND == InputKind::Uniform {
            std::mem::size_of::<Self::Repr>()
        } else {
            0
        }
    }
}

/// Allows visiting the u8 representation of all separate parts of encoded value.
/// The visiting is always performed in the same order as defined in the `Properties` of an encoder.
pub trait IterableEncoding: Sized {
    /// Return a count of iterations
    fn count() -> usize;
    #[doc(hidden)]
    fn for_each_offsetted<F: FnMut(usize, &[u8])>(self, idx: usize, f: F) -> F;
    /// Iterate over all encoded buffers in given returned value
    #[inline(always)]
    fn for_each(self, f: impl FnMut(usize, &[u8])) {
        self.for_each_offsetted(0, f);
    }
}

impl<T: ResolvedEncoding> IterableEncoding for T {
    #[inline(always)]
    fn count() -> usize {
        1
    }
    #[inline(always)]
    fn for_each_offsetted<F: FnMut(usize, &[u8])>(self, idx: usize, mut f: F) -> F {
        f(idx, self.as_bytes());
        f
    }
}

macro_rules! define_shader_inputs {
    ($($(#[$meta:meta])* $variant:ident $typename:ident => { $kind:ident, $repr:ty }),*,) => {

        /// An enum of all supported uniform or attribute types
        #[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
        pub enum ShaderInput {
            $(
                $(#[$meta])*
                $variant,
            )*
        }

        impl ShaderInput {
            /// Retreive the size of data structure for uniform properties.
            /// Returns 0 for non-uniform data.
            pub fn ubo_size(&self) -> usize {
                match self {
                    $(ShaderInput::$variant => $typename::ubo_size(),)*
                }
            }
        }

        $(
            $(#[$meta])*
            pub struct $typename;
            impl ShaderInputType for $typename {
                const TY: ShaderInput = ShaderInput::$variant;
                const KIND: InputKind = InputKind::$kind;
                type Repr = $repr;
            }
        )*

    };
}

// TODO: support more types, like boolean vectors or any scalars
// TODO: address alignment
define_shader_inputs! {
    /// A vector of 4 single precision floats
    Vec4 EncVec4 => { Uniform, [f32; 4] },
    /// A vector of 2 single precision floats
    Vec2 EncVec2 => { Uniform, [f32; 2] },
    /// A matrix of 4x4 single precision floats
    Mat4x4 EncMat4x4 => { Uniform, [[f32; 4]; 4] },
    /// A vector of 4 signed integers
    Vec4i EncVec4i => { Uniform, [i32; 4] },
    /// A vector of 2 signed integers
    Vec2i EncVec2i => { Uniform, [i32; 2] },
    /// A vector of 4x4 signed integers
    Mat4x4i EncMat4x4i => { Uniform, [[i32; 4]; 4] },
    /// A vector of 4 unsigned integers
    Vec4u EncVec4u => { Uniform, [u32; 4] },
    /// A vector of 2 unsigned integers
    Vec2u EncVec2u => { Uniform, [u32; 2] },
    /// A matrix of 4x4 unsigned integers
    Mat4x4u EncMat4x4u => { Uniform, [[u32; 4]; 4] },

    // TODO: support non-uniform data
    // /// A 2d texture
    // Texture EncTexture => { Attribute, Handle<Texture> },
    // /// A mesh (list of vertex attributes)
    // Mesh EncMesh => { Attribute, Handle<Mesh> },
}

/// Combined type that maps a shader attribute layout (a tuple of `ShaderInputType`s)
/// into the corresponding output of an encoder.
pub trait EncodingValue {
    /// A value that is the result of an encoding. Any encoder output must eventually
    /// be resolved to that type at some stage.
    type Value: IterableEncoding;
    /// Optional version of the encoding value. This is what encoders actually pass to the `BufferWriter`.
    type OptValue;
    /// Resolve the optional value into a valid encoding output, using fallback values where needed.
    fn resolve(optional: Self::OptValue, fallback: Self::Value) -> Self::Value;
}

impl<A> EncodingValue for A
where
    A: ShaderInputType,
{
    type Value = A::Repr;
    type OptValue = Option<A::Repr>;
    // type Resolved = Self::Value;
    fn resolve(optional: Self::OptValue, fallback: Self::Value) -> Self::Value {
        optional.unwrap_or(fallback)
    }
}

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

    /// Retreive the size in bytes of underlying property representation.
    fn size() -> usize {
        std::mem::size_of::<<Self::EncodedType as EncodingValue>::Value>()
    }

    /// Retreive fallback value for missing encoded output
    fn fallback() -> <Self::EncodedType as EncodingValue>::Value;
}

/// A runtime representation for unique encodable shader input property
pub type EncodedProp = (ShaderInput, &'static str);

/// A compile-time list of `EncProperty`s.
pub trait EncProperties {
    /// A combined compile-time value representing all encoded properties
    type EncodedType: EncodingValue;
    /// An props iterator type returned from `get_props`
    type PropsIter: Iterator<Item = EncodedProp>;

    /// Retreive a vec of associated (type name, property, byte offset, byte size) tuples at runtime
    fn get_props() -> Self::PropsIter;

    /// Retreive fallback value for missing encoded output
    fn fallback() -> <Self::EncodedType as EncodingValue>::Value;

    /// Resolve encoded optional output, potentially filling a missing value with fallback
    fn resolve(
        optional: <Self::EncodedType as EncodingValue>::OptValue,
    ) -> <Self::EncodedType as EncodingValue>::Value {
        <Self::EncodedType as EncodingValue>::resolve(optional, Self::fallback())
    }
}

impl<A: EncProperty> EncProperties for A {
    type EncodedType = A::EncodedType;
    type PropsIter = Once<EncodedProp>;

    #[inline(always)]
    fn get_props() -> Self::PropsIter {
        once(A::prop())
    }

    #[inline(always)]
    fn fallback() -> <Self::EncodedType as EncodingValue>::Value {
        A::fallback()
    }
}

macro_rules! recursive_iter {
    (@value $first:expr, $($rest:expr),*) => {
        $first.chain(recursive_iter!(@value $($rest),*))
    };
    (@value $last:expr) => {
        $last
    };
    (@type $first:ty, $($rest:ty),*) => {
        Chain<$first, recursive_iter!(@type $($rest),*)>
    };
    (@type $last:ty) => {
        $last
    };
}

macro_rules! impl_tuple_properties {
    ($($from:ident $idx:tt),*) => {

        impl<$($from,)*> EncProperties for ($($from),*,)
        where $($from: EncProperties),*,
        {
            type EncodedType = ($($from::EncodedType),*,);
            type PropsIter = recursive_iter!(@type $($from::PropsIter),*);

            #[inline(always)]
            fn get_props() -> Self::PropsIter {
                recursive_iter!(@value $($from::get_props()),*)
            }
            #[inline(always)]
            fn fallback() -> <Self::EncodedType as EncodingValue>::Value {
                ($($from::fallback()),*,)
            }
        }

        impl<$($from,)*> IterableEncoding for ($($from),*,)
        where $($from: IterableEncoding),*,
        {
            #[inline(always)]
            fn count() -> usize {
                let count = 0;
                $(let count = count + $from::count();)*
                count
            }

            #[inline(always)]
            fn for_each_offsetted<FN: FnMut(usize, &[u8])>(self, _idx: usize, f: FN) -> FN {
                $(
                    let f = self.$idx.for_each_offsetted(_idx, f);
                    let _idx = _idx + $from::count();
                )*
                f
            }
        }

        impl<$($from,)*> EncodingValue for ($($from),*,)
            where $($from: EncodingValue),*,
        {
            type Value = ($($from::Value),*,);
            type OptValue = ($($from::OptValue),*,);

            #[allow(non_snake_case)]
            fn resolve(optional: Self::OptValue, fallback: Self::Value) -> Self::Value {
                ($(<$from as EncodingValue>::resolve(optional.$idx, fallback.$idx)),*,)
            }
        }
    }
}

impl_tuple_properties! {A 0}
impl_tuple_properties! {A 0, B 1}
impl_tuple_properties! {A 0, B 1, C 2}
impl_tuple_properties! {A 0, B 1, C 2, D 3}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6 }
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13}
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14 }
impl_tuple_properties! {A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14 , P 15}
