use super::{EncProperty, Encode, EncoderQuery, EncodingSet};
use crate::SpriteRender;
use amethyst_core::specs::{SystemData, World};
use amethyst_core::GlobalTransform;
use hibitset::BitSet;

pub struct HandleFake {
    id: std::sync::Arc<u32>,
    marker: std::marker::PhantomData<()>,
}

impl HandleFake {
    /// Create fake handle for test mocking purposes
    fn new<H>(fake_id: u32) -> amethyst_assets::Handle<H> {
        let fake = Self {
            id: std::sync::Arc::new(fake_id),
            marker: std::marker::PhantomData,
        };

        unsafe { std::mem::transmute(fake) }
    }
}

fn mock_world() -> World {
    use crate::Rgba;
    use amethyst_core::specs::world::Builder;

    let mut world = World::new();

    world.register::<GlobalTransform>();
    world.register::<SpriteRender>();
    world.register::<Rgba>();

    // a few examples for testing:
    // one empty entity
    // one that has all valid components
    // one missing GlobalTransform
    // one missing Rgba - optionals feature

    world.create_entity().build();
    world
        .create_entity()
        .with(GlobalTransform::new())
        .with(SpriteRender {
            sprite_sheet: HandleFake::new(0),
            sprite_number: 0,
        })
        .with(Rgba::WHITE)
        .build();

    world
        .create_entity()
        .with(SpriteRender {
            sprite_sheet: HandleFake::new(1),
            sprite_number: 1,
        })
        .with(Rgba::RED)
        .build();
    world
        .create_entity()
        .with(GlobalTransform::new())
        .with(SpriteRender {
            sprite_sheet: HandleFake::new(2),
            sprite_number: 2,
        })
        .build();

    world
}

#[test]
pub fn test_iterator() {
    let ref res = mock_world().res;

    let cs = <(
        Encode<'_, GlobalTransform>,
        (Encode<'_, SpriteRender>, Encode<'_, SpriteRender>),
    ) as SystemData<'_>>::fetch(&res);

    let iter = cs.join();

    assert_eq!(
        iter.map(|(_, (r, _))| r.sprite_number).collect::<Vec<_>>(),
        &[0, 2]
    );
}

#[test]
pub fn test_iterator_bound() {
    let ref res = mock_world().res;

    let cs = <(
        Encode<'_, GlobalTransform>,
        (Encode<'_, SpriteRender>, Encode<'_, SpriteRender>),
    ) as SystemData<'_>>::fetch(&res);

    let mut bound = BitSet::new();
    bound.add(0);
    bound.add(1);
    bound.add(2);

    let iter = cs.join_with(bound);

    assert_eq!(
        iter.map(|((_, (r, _)), _)| r.sprite_number)
            .collect::<Vec<_>>(),
        &[0]
    );
}

#[test]
pub fn test_querying() {
    let ref res = mock_world().res;

    use super::encoders_impl::{RgbaTintEncoder, SpriteTransformEncoder};
    use super::properties_impl::*;
    use super::WorldEncoder;

    let mut enc = WorldEncoder::build()
        .with_encoder::<RgbaTintEncoder>()
        .with_encoder::<SpriteTransformEncoder>()
        .build();

    let query = EncoderQuery::new(vec![
        TintProperty::prop(),
        Pos2DProperty::prop(),
        DirXProperty::prop(),
        DirYProperty::prop(),
    ]);

    let size = enc.query_size_hint(&res, &query);
    let mut buf = vec![0u8; size];
    let response = enc.query(&res, &query, &mut buf);

    println!("{:?}", response);
    println!("{:?}", buf);
}
