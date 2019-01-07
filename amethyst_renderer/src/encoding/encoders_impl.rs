// example implementations
use super::{
    attributes_impl::{DirXAttribute, DirYAttribute, Pos2DAttribute, TintAttribute},
    EncType, Encode, EncodeBuffer, IterType, StreamEncoder,
};
use amethyst_core::{nalgebra::Vector4, specs::Read, GlobalTransform};
use amethyst_assets::AssetStorage;
use crate::{SpriteRender, SpriteSheet, Rgba};

/// An encoder that encodes `Rgba` component into a stream of `vec4 tint`.
pub struct RgbaTintEncoder;
impl<'a: 'j, 'j> StreamEncoder<'a, 'j> for RgbaTintEncoder {
    type Attributes = TintAttribute;
    type Components = (Encode<'a, Rgba>,);
    type SystemData = ();
    fn encode<B: EncodeBuffer<EncType<'a, 'j, Self>>>(
        buffer: &mut B,
        iter: IterType<'a, 'j, Self>,
        system_data: Self::SystemData,
    ) {
        for (rgba,) in iter {
            buffer.push([rgba.0, rgba.1, rgba.2, rgba.3].into());
        }
    }
}


/// An encoder that encodes `GlobalTransform` and `RenderSpriteFlat2D` components
/// into streams of `vec4 pos`, `vec4 dir_x` and `vec4 dir_y`.
pub struct SpriteTransformEncoder;
impl<'a: 'j, 'j> StreamEncoder<'a, 'j> for SpriteTransformEncoder {
    type Attributes = (Pos2DAttribute, DirXAttribute, DirYAttribute);
    type Components = (Encode<'a, GlobalTransform>, Encode<'a, SpriteRender>);
    type SystemData = (Read<'a, AssetStorage<SpriteSheet>>);
    fn encode<B: EncodeBuffer<EncType<'a, 'j, Self>>>(
        buffer: &mut B,
        iter: IterType<'a, 'j, Self>,
        storage: Self::SystemData,
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
