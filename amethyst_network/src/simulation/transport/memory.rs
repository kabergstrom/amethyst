use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::{
        TransportResource, NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME,
        NETWORK_SIM_TIME_SYSTEM_NAME,
    },
};
use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, Read, System, World, Write, WriteExpect},
    shrev::EventChannel,
};
use amethyst_error::Error;
use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use log::error;

pub use crossbeam_channel::unbounded as channel;

/// Use this network bundle to add the memory transport layer to your game.
pub struct MemoryNetworkBundle {
    tx: Sender<Bytes>,
    rx: Receiver<Bytes>,
}

impl MemoryNetworkBundle {
    pub fn new(tx: Sender<Bytes>, rx: Receiver<Bytes>) -> Self {
        Self { tx, rx }
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for MemoryNetworkBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'_, '_>,
    ) -> Result<(), Error> {
        builder.add(MemoryNetworkSendSystem, NETWORK_SEND_SYSTEM_NAME, &[]);
        builder.add(MemoryNetworkRecvSystem, NETWORK_RECV_SYSTEM_NAME, &[]);
        builder.add(
            NetworkSimulationTimeSystem,
            NETWORK_SIM_TIME_SYSTEM_NAME,
            &[NETWORK_SEND_SYSTEM_NAME, NETWORK_RECV_SYSTEM_NAME],
        );
        world.insert(MemoryChannelResource::new(self.tx, self.rx));
        Ok(())
    }
}

pub struct MemoryNetworkSendSystem;

impl<'s> System<'s> for MemoryNetworkSendSystem {
    type SystemData = (
        Write<'s, TransportResource>,
        WriteExpect<'s, MemoryChannelResource>,
        Read<'s, NetworkSimulationTime>,
    );

    fn run(&mut self, (mut transport, mut channels, sim_time): Self::SystemData) {
        let messages = transport.drain_messages_to_send(|_| sim_time.should_send_message_now());
        for message in messages.into_iter() {
            match message.delivery {
                _ => {
                    if let Err(e) = channels.tx.try_send(message.payload) {
                        error!("There was an error when attempting to send packet: {:?}", e);
                    }
                }
            }
        }
    }
}

pub struct MemoryNetworkRecvSystem;

impl<'s> System<'s> for MemoryNetworkRecvSystem {
    type SystemData = (
        WriteExpect<'s, MemoryChannelResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut channels, mut event_channel): Self::SystemData) {
        loop {
            match channels.rx.try_recv() {
                Ok(buf) => {
                    let event = NetworkSimulationEvent::Message(
                        std::net::SocketAddr::new("0.0.0.0".parse().unwrap(), 0),
                        buf,
                    );
                    // TODO: Handle other types of events.
                    event_channel.single_write(event);
                }
                Err(_) => break,
            }
        }
    }
}

/// Resource to own the UDP socket.
pub struct MemoryChannelResource {
    tx: Sender<Bytes>,
    rx: Receiver<Bytes>,
}

impl MemoryChannelResource {
    /// Create a new instance of the `MemoryChannelResource`
    pub fn new(tx: Sender<Bytes>, rx: Receiver<Bytes>) -> Self {
        Self { tx, rx }
    }
}
