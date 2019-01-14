use super::bitset::VecBitSet;
use amethyst_core::specs::{
    storage::GenericReadStorage, storage::UnprotectedStorage, world::Index, Component, ReadStorage,
    SystemData,
};
use hibitset::BitSetAll;

/// A read-only access to a component storage. Component types listed in the list of `Encoder`s
/// on a `StreamEncoder` trait are used for scheduling the encoding for rendering.
///
/// Constrained in the same way as `ReadStorage`. You can't use `WriteStorage` with the same inner type at the same time.
pub struct Encode<A: Component>(std::marker::PhantomData<A>);

/// A read-only joinable composable list of component types.
// pub trait EncodingSet<'j> {
//     /// Join representation of encoding set
//     type IterItem;
// }

pub trait FetchedData<'j> {
    type Ref;
}

pub trait EncodingData<'a> {
    type SystemData: SystemData<'a>;
    type FetchedData: for<'j> FetchedData<'j>;

    fn get_masks<'j>(data: &'j Self::SystemData) -> VecBitSet<'j>;
    fn get_data<'j>(
        data: &'j Self::SystemData,
        index: Index,
    ) -> <Self::FetchedData as FetchedData<'j>>::Ref;
}

pub trait EncodingDefItem {
    type Fetched: Component;
    const BOUND: bool;
}

pub trait IterableEncodingDefItem<'j> {
    type IterType: 'j;
}

pub trait EncodingDef
where
    // for<'j> Self: EncodingSet<'j>,
    for<'a> Self: EncodingData<'a>,
{
    fn fetch<'a>(res: &'a shred::Resources) -> <Self as EncodingData<'a>>::SystemData {
        SystemData::<'a>::fetch(res)
    }

    fn get_masks<'a, 'j>(data: &'j <Self as EncodingData<'a>>::SystemData) -> VecBitSet<'j> {
        <Self as EncodingData<'a>>::get_masks(data)
    }

    fn get_data<'a, 'j>(
        data: &'j <Self as EncodingData<'a>>::SystemData,
        index: Index,
    ) -> <<Self as EncodingData<'a>>::FetchedData as FetchedData<'j>>::Ref {
        <Self as EncodingData<'a>>::get_data(data, index)
    }
}

impl<A: Component> EncodingDefItem for Encode<A> {
    type Fetched = A;
    const BOUND: bool = true;
}
impl<'j, A: Component + 'j> IterableEncodingDefItem<'j> for Encode<A> {
    type IterType = &'j A;
}

macro_rules! impl_encoding_set {
    // use variables to indicate the arity of the tuple
    ($($from:ident),*) => {
        impl<$($from,)*> EncodingDef for ($($from),*,)
            where
                $($from: EncodingDefItem),*,
                // for<'j> Self: EncodingSet<'j>,
                for<'a> Self: EncodingData<'a>,
        {
        }

        // impl<'j, $($from,)*> EncodingSet<'j> for ($($from),*,)
        //     where
        //         $($from: IterableEncodingDefItem<'j>),*,
        //         for<'a> Self: EncodingData<'a>,
        // {
        //     type IterItem = ($(Option<$from::IterType>),*,);
        // }

        impl<'j, $($from: 'j,)*> FetchedData<'j> for ($(Option<$from>),*,) {
            type Ref = ($(Option<&'j $from>),*,);
        }

        impl<'a, $($from,)*> EncodingData<'a> for ($($from),*,)
            where $($from: EncodingDefItem, $from::Fetched: 'a),*,
        {
            type SystemData = ($(ReadStorage<'a, $from::Fetched>),*,);
            type FetchedData = ($(Option<$from::Fetched>),*,);

            #[allow(non_snake_case)]
            fn get_masks<'j>(data: &'j Self::SystemData) -> VecBitSet<'j> {
                let (ref $($from),*,) = data;
                let mut vec = vec![];
                $(
                    if $from::BOUND {
                        vec.push($from.mask());
                    }
                )*
                VecBitSet(vec)
            }

            #[allow(non_snake_case)]
            fn get_data<'j>(data: &'j Self::SystemData, index: Index) -> <Self::FetchedData as FetchedData<'j>>::Ref {
                let ($($from),*,) = data;
                ($(
                    if $from.mask().contains(index) {
                        unsafe {
                            Some($from.unprotected_storage().get(index))
                        }
                    } else {
                        None
                    }
                ),*,)
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
