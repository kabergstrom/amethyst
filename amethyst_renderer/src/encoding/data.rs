use amethyst_core::specs::{
    join::{Join, JoinIter, MaybeJoin},
    world::Index,
    Component, ReadStorage, SystemData,
};
use shred::{ResourceId, Resources};

/// A read-only access to a component storage. Component types listed in the list of `Encoder`s or `MaybeEncoder`s
/// on a `StreamEncoder` trait are used for scheduling the encoding for rendering.
///
/// Constrained in the same way as `ReadStorage`. You can't use `WriteStorage` with the same inner type at the same time.
pub struct Encode<'a, A: Component>(ReadStorage<'a, A>);

/// A read-only access to a optional component storage. Component types listed in the list of `Encoder`s
/// on a `StreamEncoder` trait are used for scheduling the encoding for rendering.
/// Encoder has to push a value to the buffer even if the encoded optional component is `None`.
///
/// Constrained in the same way as `ReadStorage`. You can't use `WriteStorage` with the same inner type at the same time.
pub struct MaybeEncode<'a, A: Component>(ReadStorage<'a, A>);

impl<'a, T> SystemData<'a> for Encode<'a, T>
where
    T: Component,
{
    fn setup(res: &mut Resources) {
        <ReadStorage<'a, T> as SystemData<'a>>::setup(res)
    }

    fn fetch(res: &'a Resources) -> Self {
        Encode(<ReadStorage<'a, T> as SystemData<'a>>::fetch(res))
    }

    fn reads() -> Vec<ResourceId> {
        <ReadStorage<'a, T> as SystemData<'a>>::reads()
    }

    fn writes() -> Vec<ResourceId> {
        <ReadStorage<'a, T> as SystemData<'a>>::writes()
    }
}

impl<'a, T> SystemData<'a> for MaybeEncode<'a, T>
where
    T: Component,
{
    fn setup(res: &mut Resources) {
        <ReadStorage<'a, T> as SystemData<'a>>::setup(res)
    }

    fn fetch(res: &'a Resources) -> Self {
        MaybeEncode(<ReadStorage<'a, T> as SystemData<'a>>::fetch(res))
    }

    fn reads() -> Vec<ResourceId> {
        <ReadStorage<'a, T> as SystemData<'a>>::reads()
    }

    fn writes() -> Vec<ResourceId> {
        <ReadStorage<'a, T> as SystemData<'a>>::writes()
    }
}

impl<'a: 'j, 'j, A: Component> Join for &'j Encode<'a, A> {
    type Mask = <&'j ReadStorage<'a, A> as Join>::Mask;
    type Value = <&'j ReadStorage<'a, A> as Join>::Value;
    type Type = <&'j ReadStorage<'a, A> as Join>::Type;
    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        Join::open(&self.0)
    }
    unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
        <&'j ReadStorage<'a, A> as Join>::get(value, id)
    }
}

impl<'a: 'j, 'j, A: Component> Join for &'j MaybeEncode<'a, A> {
    type Mask = <MaybeJoin<&'j ReadStorage<'a, A>> as Join>::Mask;
    type Value = <MaybeJoin<&'j ReadStorage<'a, A>> as Join>::Value;
    type Type = <MaybeJoin<&'j ReadStorage<'a, A>> as Join>::Type;
    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        Join::open(self.0.maybe())
    }
    unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
        <MaybeJoin<&'j ReadStorage<'a, A>> as Join>::get(value, id)
    }
}

pub trait DeferredEncodingSet: Sized {}

/// A read-only joinable composable list of component types.
/// TODO: Allow for constraining the iterated list of components by external BitVec
pub trait EncodingSet<'j> {
    /// Join representation of encoding set
    type Joined: Join;
    /// Get joinable value wrapped by `EncodingSet`
    fn inner(&'j self) -> Self::Joined;
    /// Join on all elements of encoding set, retreive the iterator for encoding.
    fn join(&'j self) -> JoinIter<Self::Joined> {
        self.inner().join()
    }
    /// Join on all elements of encoding set, bounded externally. retreive the iterator for encoding.
    fn join_with<J: Join>(&'j self, other: J) -> JoinIter<(Self::Joined, J)> {
        (self.inner(), other).join()
    }
}

impl<'a, A: Component> DeferredEncodingSet for Encode<'a, A> {}
impl<'a, A: Component> DeferredEncodingSet for MaybeEncode<'a, A> {}

impl<'j, 'a: 'j, A: Component> EncodingSet<'j> for Encode<'a, A> {
    type Joined = &'j ReadStorage<'a, A>;
    fn inner(&'j self) -> Self::Joined {
        &self.0
    }
}

impl<'j, 'a: 'j, A: Component> EncodingSet<'j> for MaybeEncode<'a, A> {
    type Joined = MaybeJoin<&'j ReadStorage<'a, A>>;
    fn inner(&'j self) -> Self::Joined {
        self.0.maybe()
    }
}

macro_rules! impl_encoding_set {
    // use variables to indicate the arity of the tuple
    ($($from:ident),*) => {
        impl<$($from,)*> DeferredEncodingSet for ($($from),*,)
            where $($from: DeferredEncodingSet),*,
        {}

        impl<'j, $($from,)*> EncodingSet<'j> for ($($from),*,)
            where $($from: EncodingSet<'j>),*,
        {
            type Joined = ($($from::Joined),*,);
            #[allow(non_snake_case)]
            fn inner(&'j self) -> Self::Joined {
                let ($($from,)*) = self;
                ($($from.inner()),*,)
            }
        }
    }
}

impl_encoding_set! {A}
impl_encoding_set! {A, B}
impl_encoding_set! {A, B, C}
impl_encoding_set! {A, B, C, D}
impl_encoding_set! {A, B, C, D, E}
impl_encoding_set! {A, B, C, D, E, F}
impl_encoding_set! {A, B, C, D, E, F, G}
impl_encoding_set! {A, B, C, D, E, F, G, H}
impl_encoding_set! {A, B, C, D, E, F, G, H, I}
impl_encoding_set! {A, B, C, D, E, F, G, H, I, J}
impl_encoding_set! {A, B, C, D, E, F, G, H, I, J, K}
impl_encoding_set! {A, B, C, D, E, F, G, H, I, J, K, L}
impl_encoding_set! {A, B, C, D, E, F, G, H, I, J, K, L, M}
impl_encoding_set! {A, B, C, D, E, F, G, H, I, J, K, L, M, N}
impl_encoding_set! {A, B, C, D, E, F, G, H, I, J, K, L, M, N, O}
impl_encoding_set! {A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P}
