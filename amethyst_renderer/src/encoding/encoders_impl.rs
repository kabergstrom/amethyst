// example implementations
use super::{
    properties_impl::{DirXProperty, DirYProperty, Pos2DProperty, TintProperty},
    DataType, EncType, Encode, EncodeBuffer, EncodingData, EncodingJoin, IterItem, MaybeEncode,
    StreamEncoder,
};
use crate::{Rgba, SpriteRender, SpriteSheet};
use amethyst_assets::AssetStorage;
use amethyst_core::{nalgebra::Vector4, specs::Read, GlobalTransform};

/// An encoder that encodes `Rgba` component into a stream of `vec4 tint`.
pub struct RgbaTintEncoder;
impl<'a> StreamEncoder<'a> for RgbaTintEncoder {
    type Properties = TintProperty;
    type Components = (MaybeEncode<Rgba>,);
    type SystemData = ();

    fn encode<'j>(
        buffer: &mut impl EncodeBuffer<EncType<'a, 'j, Self>>,
        iter: impl Iterator<Item = IterItem<'a, 'j, Self>>,
        system_data: DataType<'a, Self>,
    ) {
        for (rgba,) in iter {
            let rgba = rgba.unwrap_or(&Rgba::WHITE);
            buffer.push([rgba.0, rgba.1, rgba.2, rgba.3].into());
        }
    }
}

/// An encoder that encodes `GlobalTransform` and `RenderSpriteFlat2D` components
/// into streams of `vec4 pos`, `vec4 dir_x` and `vec4 dir_y`.
pub struct SpriteTransformEncoder;
impl<'a> StreamEncoder<'a> for SpriteTransformEncoder {
    type Properties = (Pos2DProperty, DirXProperty, DirYProperty);
    type Components = (Encode<GlobalTransform>, Encode<SpriteRender>);
    type SystemData = (Read<'a, AssetStorage<SpriteSheet>>);

    fn encode<'j>(
        buffer: &mut impl EncodeBuffer<EncType<'a, 'j, Self>>,
        iter: impl Iterator<Item = IterItem<'a, 'j, Self>>,
        storage: DataType<'a, Self>,
    ) {
        for (transform, sprite_render) in iter {
            let ref sprite_sheet = storage.get(&sprite_render.sprite_sheet).unwrap();
            let ref sprite = sprite_sheet.sprites[sprite_render.sprite_number];
            let dir_x = transform.0.column(0) * sprite.width;
            let dir_y = transform.0.column(1) * sprite.height;
            let pos = transform.0 * Vector4::new(-sprite.offsets[0], -sprite.offsets[1], 0.0, 1.0);

            buffer.push((pos.into(), dir_x.into(), dir_y.into()));
        }
    }
}
