// example implementations
use super::{
    properties_impl::{DirXProperty, DirYProperty, Pos2DProperty, TintProperty},
    stream_encoder::{EncodeLoop, LoopResult, LoopingStreamEncoder},
    Encode,
};
use crate::{Rgba, SpriteRender, SpriteSheet};
use amethyst_assets::AssetStorage;
use amethyst_core::{nalgebra::Vector4, specs::Read, GlobalTransform};

/// An encoder that encodes `Rgba` component into a stream of `vec4 tint`.
pub struct RgbaTintEncoder;
impl<'a> LoopingStreamEncoder<'a> for RgbaTintEncoder {
    type Properties = TintProperty;
    type Components = (Encode<Rgba>,);
    type SystemData = ();

    fn encode(
        encode_loop: impl EncodeLoop<Self::Components, Self::Properties>,
        _: Self::SystemData,
    ) -> LoopResult {
        encode_loop.run(|(rgba,)| {
            let rgba = rgba.unwrap_or(&Rgba::WHITE);
            Some([rgba.0, rgba.1, rgba.2, rgba.3])
        })
    }
}

/// An encoder that encodes `GlobalTransform` and `RenderSpriteFlat2D` components
/// into streams of `vec4 pos`, `vec4 dir_x` and `vec4 dir_y`.
pub struct SpriteTransformEncoder;
impl<'a> LoopingStreamEncoder<'a> for SpriteTransformEncoder {
    type Properties = (Pos2DProperty, DirXProperty, DirYProperty);
    type Components = (Encode<GlobalTransform>, Encode<SpriteRender>);
    type SystemData = (Read<'a, AssetStorage<SpriteSheet>>);
    fn encode(
        encode_loop: impl EncodeLoop<Self::Components, Self::Properties>,
        spritesheet_storage: Self::SystemData,
    ) -> LoopResult {
        encode_loop.run(|(transform, sprite_render)| {
            if let (Some(transform), Some(sprite_render)) = (transform, sprite_render) {
                let ref sprite_sheet = spritesheet_storage
                    .get(&sprite_render.sprite_sheet)
                    .unwrap();
                let ref sprite = sprite_sheet.sprites[sprite_render.sprite_number];
                let dir_x = transform.0.column(0) * sprite.width;
                let dir_y = transform.0.column(1) * sprite.height;
                let pos =
                    transform.0 * Vector4::new(-sprite.offsets[0], -sprite.offsets[1], 0.0, 1.0);
                (Some(pos.into()), Some(dir_x.into()), Some(dir_y.into()))
            } else {
                (None, None, None)
            }
        })
    }
}
