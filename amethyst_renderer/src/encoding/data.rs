use amethyst_core::specs::{
    join::{Join, JoinIter, MaybeJoin},
    world::Index,
    Component, ReadStorage, SystemData,
};
use hibitset::BitSetAll;
use hibitset::BitSetLike;
use shred::{ResourceId, Resources};

/// A read-only access to a component storage. Component types listed in the list of `Encoder`s or `MaybeEncoder`s
/// on a `StreamEncoder` trait are used for scheduling the encoding for rendering.
///
/// Constrained in the same way as `ReadStorage`. You can't use `WriteStorage` with the same inner type at the same time.
pub struct Encode<A: Component>(std::marker::PhantomData<A>);

/// A read-only access to a optional component storage. Component types listed in the list of `Encoder`s
/// on a `StreamEncoder` trait are used for scheduling the encoding for rendering.
/// Encoder has to push a value to the buffer even if the encoded optional component is `None`.
///
/// Constrained in the same way as `ReadStorage`. You can't use `WriteStorage` with the same inner type at the same time.
pub struct MaybeEncode<A: Component>(std::marker::PhantomData<A>);

// impl<'a, T> SystemData<'a> for Encode<T>
// where
//     T: Component,
// {
//     fn setup(res: &mut Resources) {
//         <ReadStorage<'a, T> as SystemData<'a>>::setup(res)
//     }

//     fn fetch(res: &'a Resources) -> Self {
//         Encode(<ReadStorage<'a, T> as SystemData<'a>>::fetch(res))
//     }

//     fn reads() -> Vec<ResourceId> {
//         <ReadStorage<'a, T> as SystemData<'a>>::reads()
//     }

//     fn writes() -> Vec<ResourceId> {
//         <ReadStorage<'a, T> as SystemData<'a>>::writes()
//     }
// }

// impl<'a, T> SystemData<'a> for MaybeEncode<T>
// where
//     T: Component,
// {
//     fn setup(res: &mut Resources) {
//         <ReadStorage<'a, T> as SystemData<'a>>::setup(res)
//     }

//     fn fetch(res: &'a Resources) -> Self {
//         MaybeEncode(<ReadStorage<'a, T> as SystemData<'a>>::fetch(res))
//     }

//     fn reads() -> Vec<ResourceId> {
//         <ReadStorage<'a, T> as SystemData<'a>>::reads()
//     }

//     fn writes() -> Vec<ResourceId> {
//         <ReadStorage<'a, T> as SystemData<'a>>::writes()
//     }
// }

// impl<'a: 'j, 'j, A: Component> Join for &'j Encode<'a, A> {
//     type Mask = <&'j ReadStorage<'a, A> as Join>::Mask;
//     type Value = <&'j ReadStorage<'a, A> as Join>::Value;
//     type Type = <&'j ReadStorage<'a, A> as Join>::Type;
//     unsafe fn open(self) -> (Self::Mask, Self::Value) {
//         Join::open(&self.0)
//     }
//     unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
//         <&'j ReadStorage<'a, A> as Join>::get(value, id)
//     }
// }

// impl<'a: 'j, 'j, A: Component> Join for &'j MaybeEncode<'a, A> {
//     type Mask = <MaybeJoin<&'j ReadStorage<'a, A>> as Join>::Mask;
//     type Value = <MaybeJoin<&'j ReadStorage<'a, A>> as Join>::Value;
//     type Type = <MaybeJoin<&'j ReadStorage<'a, A>> as Join>::Type;
//     unsafe fn open(self) -> (Self::Mask, Self::Value) {
//         Join::open(self.0.maybe())
//     }
//     unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
//         <MaybeJoin<&'j ReadStorage<'a, A>> as Join>::get(value, id)
//     }
// }

/// A read-only joinable composable list of component types.
/// TODO: Allow for constraining the iterated list of components by external BitVec
pub trait EncodingSet<'j> {
    /// Join representation of encoding set
    type IterItem;
    // /// Get joinable value wrapped by `EncodingSet`
    // fn inner(&'j self) -> Self::Joined;
    // /// Join on all elements of encoding set, retreive the iterator for encoding.
    // fn join(&'j self) -> JoinIter<Self::Joined> {
    //     self.inner().join()
    // }
    // /// Join on all elements of encoding set, bounded externally. retreive the iterator for encoding.
    // fn join_with<J: Join>(&'j self, other: J) -> JoinIter<(Self::Joined, J)> {
    //     (self.inner(), other).join()
    // }
}

pub trait EncodingData<'a> {
    type SystemData: SystemData<'a>; // + for<'j> EncodingStorageJoin<'j>;
}

pub trait EncodingDefItem {
    type Fetched: Component;
}
pub trait IterableEncodingDefItem<'j> {
    type IterType: 'j;
}
pub trait JoinedEncodingDefItem<'a, 'j> {
    type Storage: SystemData<'a> + 'j;
    type Joinable: Join;
    fn get_joinable(storage: &'j Self::Storage) -> Self::Joinable;
}

pub trait EncodingDef
where
    for<'j> Self: EncodingSet<'j>,
    for<'a> Self: EncodingData<'a>,
{
    fn fetch<'a>(res: &'a shred::Resources) -> <Self as EncodingData<'a>>::SystemData {
        SystemData::<'a>::fetch(res)
    }

    fn joinable<'a: 'j, 'j>(
        res: &'j <Self as EncodingData<'a>>::SystemData,
    ) -> <Self as EncodingJoin<'a, 'j>>::Joinable
    where
        Self: EncodingJoin<'a, 'j>;
}

impl<A: Component> EncodingDefItem for Encode<A> {
    type Fetched = A;
}
impl<'j, A: Component + 'j> IterableEncodingDefItem<'j> for Encode<A> {
    type IterType = &'j A;
}
impl<A: Component> EncodingDefItem for MaybeEncode<A> {
    type Fetched = A;
}
impl<'j, A: Component + 'j> IterableEncodingDefItem<'j> for MaybeEncode<A> {
    type IterType = Option<&'j A>;
}

impl<'a: 'j, 'j, A: Component + 'a + 'j> JoinedEncodingDefItem<'a, 'j> for Encode<A> {
    type Storage = ReadStorage<'a, A>;
    type Joinable = &'j ReadStorage<'a, A>;
    fn get_joinable(storage: &'j Self::Storage) -> Self::Joinable {
        storage
    }
}
impl<'a: 'j, 'j, A: Component + 'a + 'j> JoinedEncodingDefItem<'a, 'j> for MaybeEncode<A> {
    type Storage = ReadStorage<'a, A>;
    type Joinable = MaybeJoin<&'j ReadStorage<'a, A>>;
    fn get_joinable(storage: &'j Self::Storage) -> Self::Joinable {
        storage.maybe()
    }
}
pub trait EncodingJoin<'a: 'j, 'j> {
    type SystemData: SystemData<'a> + 'j;
    type Joinable;
    fn joinable(data: &'j Self::SystemData) -> Self::Joinable;
}

pub trait EncodingStorageJoin<'j>
where
    Self: 'j,
{
    type Joinable;
    fn joinable(data: &'j Self) -> Self::Joinable;
}

macro_rules! impl_encoding_set {
    // use variables to indicate the arity of the tuple
    ($($from:ident),*) => {
        impl<$($from,)*> EncodingDef for ($($from),*,)
            where
                $($from: EncodingDefItem),*,
                for<'j> Self: EncodingSet<'j>,
                for<'a> Self: EncodingData<'a>,
        {
            fn joinable<'a: 'j, 'j>(data: &'j <Self as EncodingData<'a>>::SystemData) -> <Self as EncodingJoin<'a, 'j>>::Joinable
            where
                Self: EncodingJoin<'a, 'j>,
            {
                // Transmute because of this error. We can't express the euqlity of those in the typesystem,
                // but it always guarenteed to be the case.
                // It should be possible to avoid once generic associated lifetimes are stable.
                // note: expected type `&'j <(A, B, ..) as encoding::data::EncodingJoin<'a, 'j>>::SystemData`
                //          found type `&'j <(A, B, ..) as encoding::data::EncodingData<'a>>::SystemData`
                // <Self as EncodingJoin<'a, 'j>>::joinable(data)
                <Self as EncodingJoin<'a, 'j>>::joinable(unsafe { std::mem::transmute(data) })
            }
        }

        impl<'j, $($from,)*> EncodingSet<'j> for ($($from),*,)
            where $($from: IterableEncodingDefItem<'j>),*,
        {
            type IterItem = ($($from::IterType),*,);
        }

        impl<'a, $($from,)*> EncodingData<'a> for ($($from),*,)
            where $($from: EncodingDefItem, $from::Fetched: 'a),*,
        {
            type SystemData = ($(ReadStorage<'a, $from::Fetched>),*,);
        }

        impl<'a: 'j, 'j, $($from,)*> EncodingJoin<'a, 'j> for ($($from),*,)
            where
                $($from: JoinedEncodingDefItem<'a, 'j>, <$from as JoinedEncodingDefItem<'a, 'j>>::Storage: 'j),*,
        {
            type SystemData = ($($from::Storage),*,);
            type Joinable = ($($from::Joinable),*,);

            #[allow(non_snake_case)]
            fn joinable(data: &'j Self::SystemData) -> Self::Joinable {
                let (ref $($from),*,) = data;
                ($($from::get_joinable($from)),*,)
            }
        }

        impl<'a: 'j, 'j, $($from,)*> EncodingStorageJoin<'j> for ($(ReadStorage<'a, $from>),*,)
            where
                $($from: Component),*,
                ($($from),*,): EncodingJoin<'a, 'j, SystemData = Self>,
        {
            type Joinable = <($($from),*,) as EncodingJoin<'a, 'j>>::Joinable;
            fn joinable(data: &'j Self) -> Self::Joinable {
                <($($from),*,) as EncodingJoin<'a, 'j>>::joinable(data)
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
