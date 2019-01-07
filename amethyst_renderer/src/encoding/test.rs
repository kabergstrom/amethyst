use amethyst::{
    core::GlobalTransform,
    renderer::SpriteRender,
    shred::{Resources},
};

// test if implementations and parts of api compile correctly

// Fetching
pub fn test_iterator(res: &Resources) {
    let cs = res.fetch::<(
        Encode<'_, GlobalTransform>,
        (Encode<'_, SpriteRender>, Encode<'_, SpriteRender>),
    )>();
    let iter = cs.join();

    for (&el, (a, b)) in iter {
        println!("{:?}", el);
    }
}
