use criterion::Criterion;
use shred::Resources;

struct TestCentralComponent(Handle<Shader>);
impl Component for TestCentralComponent {
    type Storage = VecStorage<Self>;
}

fn test_method(_res: &Resources) {
}

fn mock_world(repeats: usize) -> World {
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

    for _ in 0...repeats {
        world.create_entity().build();
        world
            .create_entity()
            .with(GlobalTransform::new())
            .with(SpriteRender {
                sprite_sheet: HandleFake::new(0),
                sprite_number: 0,
            })
            .with(Rgba::BLUE)
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
    }


    world
}

fn perform_query (c: &mut Criterion) {
    let small_world = mock_world(1);
}

fn perform_query (c: &mut Criterion) {

}

fn criterion_benchmark (c: &mut Criterion) {
    let small_world = mock_world(1);
    let medium_world = mock_world(100);
    let large_world = mock_world(10000);
    let big_enough_buffer = [u8; 50000];

    let query = EncodingQuery::new(|c: &TestCentralComponent| c.0.clone());


    c.bench_function("small linear write", || linear_write(small_world.res, &big_enough_buffer));
    c.bench_function("medium linear write", || linear_write(medium_world.res, &big_enough_buffer));
    c.bench_function("large linear write", || linear_write(large_world.res, &big_enough_buffer));
    c.bench_function("small random write", || random_write(small_world.res, &big_enough_buffer));
    c.bench_function("medium random write", || random_write(medium_world.res, &big_enough_buffer));
    c.bench_function("large random write", || random_write(large_world.res, &big_enough_buffer));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);