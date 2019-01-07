use super::{Encode, EncoderQuery, EncodingSet};
use crate::SpriteRender;
use amethyst_core::GlobalTransform;

// test if implementations and parts of api compile correctly

// Fetching
#[allow(dead_code)]
pub fn test_iterator(res: &shred::Resources) {
    let cs = res.fetch::<(
        Encode<'_, GlobalTransform>,
        (Encode<'_, SpriteRender>, Encode<'_, SpriteRender>),
    )>();
    let iter = cs.join();

    for (&el, (_a, _b)) in iter {
        println!("{:?}", el);
    }
}

#[allow(dead_code)]
pub fn test_register(res: &shred::Resources) {
    use super::encoders_impl::RgbaTintEncoder;
    use super::WorldEncoder;

    let enc = WorldEncoder::build()
        .with_encoder::<RgbaTintEncoder>()
        .build();

    let query = EncoderQuery::<()>::new();

    let size = enc.query_size_hint(res, &query);
    let mut buf = vec![0u8; size];
    let response = enc.query(res, &query, &mut buf);

    println!("{:?}", response);
    println!("{:?}", buf);
}
