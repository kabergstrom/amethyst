use std::time::Duration;

use amethyst::{
    core::{bundle::SystemBundle, frame_limiter::FrameRateLimitStrategy, SystemDesc},
    ecs::{DispatcherBuilder, Read, System, SystemData, World, Write},
    network::simulation::{
        laminar::{LaminarNetworkBundle, LaminarSocket},
        DeliveryRequirement, NetworkSimulationEvent, TransportResource, UrgencyRequirement,
    },
    prelude::*,
    shrev::{EventChannel, ReaderId},
    utils::application_root_dir,
    Result,
};
use log::info;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let socket = LaminarSocket::bind("0.0.0.0:3457")?;

    let assets_dir = application_root_dir()?.join("./");

    // XXX: This is gross. We really need a handshake in laminar. Reliable delivery will not work
    // unless you send an unreliable message first and begin the client BEFORE the 5 second disconnect
    // timer.
    //    net.send_with_requirements(
    //        "127.0.0.1:3455".parse().unwrap(),
    //        b"",
    //        DeliveryRequirement::Unreliable,
    //        UrgencyRequirement::Immediate,
    //    );

    let game_data = GameDataBuilder::default()
        .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
        .with_bundle(SpamReceiveBundle)?;
    let mut game = Application::build(assets_dir, GameState)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}

/// Default empty state
pub struct GameState;

impl SimpleState for GameState {}

#[derive(Debug)]
struct SpamReceiveBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for SpamReceiveBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(
            SpamReceiveSystemDesc::default().build(world),
            "receiving_system",
            &[],
        );
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct SpamReceiveSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, SpamReceiveSystem> for SpamReceiveSystemDesc {
    fn build(self, world: &mut World) -> SpamReceiveSystem {
        // Creates the EventChannel<NetworkEvent> managed by the ECS.
        <SpamReceiveSystem as System<'_>>::SystemData::setup(world);
        // Fetch the change we just created and call `register_reader` to get a
        // ReaderId<NetworkEvent>. This reader id is used to fetch new events from the network event
        // channel.
        let reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();
        SpamReceiveSystem::new(reader)
    }
}

/// A simple system that receives a ton of network events.
struct SpamReceiveSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl SpamReceiveSystem {
    pub fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for SpamReceiveSystem {
    type SystemData = (
        Write<'a, TransportResource>,
        Read<'a, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut net, channel): Self::SystemData) {
        for event in channel.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Message(addr, payload) => {
                    info!("{}: {:?}", addr, payload);
                    // In a typical client/server simulation, both the client and the server will
                    // be exchanging messages at a constant rate. Laminar makes use of this by
                    // packaging message acks with the next sent message. Therefore, in order for
                    // reliability to work properly, we'll send a generic "ok" response.
                    net.send("127.0.0.1:3455".parse().unwrap(), b"ok");
                }
                NetworkSimulationEvent::Connect(addr) => info!("New client connection: {}", addr),
                NetworkSimulationEvent::Disconnect(addr) => {
                    info!("Client Disconnected: {}", addr);
                }
            }
        }
    }
}
