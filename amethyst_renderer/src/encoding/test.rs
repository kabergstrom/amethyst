use super::{EncProperty, EncodingQuery, LayoutResolveCache, Shader};
use crate::{Sprite, SpriteRender, SpriteSheet};
use amethyst_assets::{AssetStorage, Handle, Loader, Processor};
use amethyst_core::{
    specs::{Component, RunNow, VecStorage, World},
    GlobalTransform, Time,
};
use rayon::ThreadPoolBuilder;
use std::sync::Arc;

struct TestCentralComponent(Handle<Shader>);
impl Component for TestCentralComponent {
    type Storage = VecStorage<Self>;
}

pub struct HandleFake {
    _id: std::sync::Arc<u32>,
    marker: std::marker::PhantomData<()>,
}
impl HandleFake {
    /// Create fake handle for test mocking purposes
    fn new<H>(fake_id: u32) -> amethyst_assets::Handle<H> {
        let fake = Self {
            _id: std::sync::Arc::new(fake_id),
            marker: std::marker::PhantomData,
        };
        unsafe { std::mem::transmute(fake) }
    }
}

fn mock_world() -> World {
    use super::{
        pipeline::{EncodingLayout, LayoutProp},
        properties_impl::*,
    };
    use crate::Rgba;
    use amethyst_core::specs::world::Builder;

    let mut world = World::new();
    let pool = Arc::new(ThreadPoolBuilder::default().build().unwrap());
    world.add_resource(pool.clone());
    world.add_resource(Loader::new(".", pool));
    world.add_resource(Time::default());
    world.add_resource(AssetStorage::<SpriteSheet>::default());
    world.add_resource(AssetStorage::<Shader>::default());
    world.add_resource(AssetStorage::<EncodingLayout>::default());
    world.add_resource(LayoutResolveCache::default());
    world.register::<GlobalTransform>();
    world.register::<SpriteRender>();
    world.register::<Rgba>();
    world.register::<TestCentralComponent>();

    let (sprite_sheet, shader_xy, shader_tint, shader_xy_tint) = {
        let loader = world.read_resource::<Loader>();
        let sprite_sheet = loader.load_from_data(
            SpriteSheet {
                texture: HandleFake::new(0),
                sprites: vec![
                    Sprite::from_pixel_values(128, 128, 64, 64, 0, 0, [0.0, 0.0]),
                    Sprite::from_pixel_values(128, 128, 64, 64, 64, 0, [0.0, 0.0]),
                    Sprite::from_pixel_values(128, 128, 64, 64, 0, 64, [0.0, 0.0]),
                    Sprite::from_pixel_values(128, 128, 64, 64, 64, 64, [0.0, 0.0]),
                ],
            },
            (),
            &world.res.fetch::<AssetStorage<SpriteSheet>>(),
        );

        let shader_xy = loader.load_from_data(
            Shader {
                mock_layout: EncodingLayout {
                    padded_size: (Pos2DProperty::size ()
                        + DirXProperty::size()
                        + DirYProperty::size()) as _,
                    props: vec![
                        LayoutProp {
                            prop: Pos2DProperty::prop(),
                            absolute_offset: 0,
                        },
                        LayoutProp {
                            prop: DirXProperty::prop(),
                            absolute_offset: Pos2DProperty::size() as _,
                        },
                        LayoutProp {
                            prop: DirYProperty::prop(),
                            absolute_offset: (Pos2DProperty::size() + DirXProperty::size()) as _,
                        },
                    ],
                },
            },
            (),
            &world.res.fetch::<AssetStorage<Shader>>(),
        );

        let shader_tint = loader.load_from_data(
            Shader {
                mock_layout: EncodingLayout {
                    padded_size: TintProperty::size() as _,
                    props: vec![LayoutProp {
                        prop: TintProperty::prop(),
                        absolute_offset: 0,
                    }],
                },
            },
            (),
            &world.res.fetch::<AssetStorage<Shader>>(),
        );

        let shader_xy_tint = loader.load_from_data(
            Shader {
                mock_layout: EncodingLayout {
                    padded_size: (Pos2DProperty::size()
                        + DirXProperty::size()
                        + DirYProperty::size()
                        + TintProperty::size()) as _,
                    props: vec![
                        LayoutProp {
                            prop: Pos2DProperty::prop(),
                            absolute_offset: 0,
                        },
                        LayoutProp {
                            prop: DirXProperty::prop(),
                            absolute_offset: Pos2DProperty::size() as _,
                        },
                        LayoutProp {
                            prop: DirYProperty::prop(),
                            absolute_offset: (Pos2DProperty::size() + DirXProperty::size()) as _,
                        },
                        LayoutProp {
                            prop: TintProperty::prop(),
                            absolute_offset: (DirYProperty::size()
                                + Pos2DProperty::size()
                                + DirXProperty::size())
                                as _,
                        },
                    ],
                },
            },
            (),
            &world.res.fetch::<AssetStorage<Shader>>(),
        );

        (sprite_sheet, shader_xy, shader_tint, shader_xy_tint)
    };

    // a few examples for testing:
    // one empty entity
    // one that has all valid components
    // one missing GlobalTransform
    // one missing Rgba - optionals feature

    world.create_entity().build();
    world
        .create_entity()
        .with(TestCentralComponent(shader_xy_tint))
        .with(GlobalTransform::new())
        .with(SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 0,
        })
        .with(Rgba::BLUE)
        .build();

    world
        .create_entity()
        .with(TestCentralComponent(shader_tint))
        .with(SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 1,
        })
        .with(Rgba::RED)
        .build();
    world
        .create_entity()
        .with(TestCentralComponent(shader_xy))
        .with(GlobalTransform::new())
        .with(SpriteRender {
            sprite_sheet: sprite_sheet,
            sprite_number: 2,
        })
        .build();

    <Processor<SpriteSheet>>::new().run_now(&world.res);
    <Processor<Shader>>::new().run_now(&world.res);
    world
}

#[test]
pub fn test_querying() {
    let ref mut res = mock_world().res;

    use super::{
        encoders_impl::{RgbaTintEncoder, SpriteTransformEncoder},
        EncoderStorage,
    };

    res.insert(
        EncoderStorage::build()
            .with_encoder::<RgbaTintEncoder>()
            .with_encoder::<SpriteTransformEncoder>()
            .build(),
    );

    let query = EncodingQuery::new(|c: &TestCentralComponent| c.0.clone());

    let evaluated = query.evaluate(res);
    println!("evaluated: {:?}", evaluated);

    let size = evaluated.ubo_size();
    let mut buffer = vec![0u8; size];
    let result = evaluated.encode(&res, &mut buffer);

    println!("result: {:?}", result);
    println!("buffer: {:?}", buffer);
}
