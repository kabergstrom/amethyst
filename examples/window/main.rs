//! Opens an empty window.

use amethyst::{
    assets::{DefaultLoader, GenericHandle, NewLoader},
    input::is_key_down,
    prelude::*,
    renderer::{DisplayConfig, DrawFlat, Pipeline, PosNormTex, RenderBundle, Stage},
    utils::application_root_dir,
    winit::VirtualKeyCode,
};
#[derive(Default)]
struct Example {
    asset1: Option<GenericHandle>,
    asset2: Option<GenericHandle>,
}

impl SimpleState for Example {
    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let loader = data.world.read_resource::<DefaultLoader>();
        self.asset1 = Some(
            loader.load_asset_generic(
                *amethyst::assets::AssetUUID::parse_str("43067e9f-d965-4436-a78b-5798a224af5d")
                    .unwrap()
                    .as_bytes(),
            ),
        );
        self.asset2 = Some(
            loader.load_asset_generic(
                *amethyst::assets::AssetUUID::parse_str("72249910-5400-433a-9be9-984e13ea3578")
                    .unwrap()
                    .as_bytes(),
            ),
        );
    }
    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        use amethyst::assets::{AssetHandle, DefaultLoader, NewAssetStorage as AssetStorage};
        let loader = &*data.world.read_resource::<DefaultLoader>();
        let storage = data
            .world
            .read_resource::<AssetStorage<amethyst::renderer::RendyMesh>>();
        if let Some(mesh) = self.asset1.as_ref().and_then(|a| a.get_asset(&*storage)) {
            log::info!("mesh vertex count {}", mesh.0.len());
            self.asset1 = None;
        }
        log::info!(
            "load state {:?}",
            self.asset1.as_ref().map(|a| a.get_load_status(loader)),
        );
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let path = application_root_dir()?.join("examples/window/resources/display_config.ron");
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat::<PosNormTex>::new()),
    );

    let game_data =
        GameDataBuilder::default().with_bundle(RenderBundle::new(pipe, Some(config)))?;
    let mut game = Application::new("./", Example::default(), game_data)?;

    game.run();

    Ok(())
}
