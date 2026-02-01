//! Utilities for building controllable JACK processors with lock-free communication.

use rtrb::{Consumer, Producer, RingBuffer};

use crate::{
    Client, Control, Frames, ProcessHandler, ProcessScope, TransportPosition, TransportState,
};

/// Communication channels available to a processor in the real-time audio thread.
pub struct ProcessorChannels<Command, Notification> {
    /// Send notifications from the processor to the controller.
    pub notifications: Producer<Notification>,
    /// Receive commands from the controller.
    pub commands: Consumer<Command>,
}

/// Handle for controlling a processor from outside the real-time audio thread.
pub struct ProcessorHandle<Command, Notification> {
    /// Receive notifications from the processor.
    pub notifications: Consumer<Notification>,
    /// Send commands to the processor.
    pub commands: Producer<Command>,
}

/// A JACK processor that can be controlled via lock-free channels.
///
/// Implement this trait to create a processor that communicates with external
/// code through commands and notifications while running in the real-time thread.
pub trait ControlledProcessorTrait: Send + Sized {
    /// Commands sent from the controller to the processor.
    type Command: Send;
    /// Notifications sent from the processor to the controller.
    type Notification: Send;

    /// See [`ProcessHandler::SLOW_SYNC`].
    const SLOW_SYNC: bool = false;

    /// Called when the transport state changes. See [`ProcessHandler::sync`].
    fn sync(
        &mut self,
        _client: &Client,
        _state: TransportState,
        _pos: &TransportPosition,
        _channels: &mut ProcessorChannels<Self::Command, Self::Notification>,
    ) -> bool {
        true
    }

    /// Called when the buffer size changes. See [`ProcessHandler::buffer_size`].
    fn buffer_size(
        &mut self,
        client: &Client,
        size: Frames,
        channels: &mut ProcessorChannels<Self::Command, Self::Notification>,
    ) -> Control;

    /// Process audio. See [`ProcessHandler::process`].
    fn process(
        &mut self,
        client: &Client,
        scope: &ProcessScope,
        channels: &mut ProcessorChannels<Self::Command, Self::Notification>,
    ) -> Control;

    /// Create a processor instance and its control handle with the given channel capacities.
    fn instance(
        self,
        notification_channel_size: usize,
        command_channel_size: usize,
    ) -> (
        ControlledProcessorInstance<Self>,
        ProcessorHandle<Self::Command, Self::Notification>,
    ) {
        let (notifications, notifications_other) =
            RingBuffer::<Self::Notification>::new(notification_channel_size);
        let (commands_other, commands) = RingBuffer::<Self::Command>::new(command_channel_size);
        let handle = ProcessorHandle {
            notifications: notifications_other,
            commands: commands_other,
        };
        let processor = ControlledProcessorInstance {
            inner: self,
            channels: ProcessorChannels {
                notifications,
                commands,
            },
        };
        (processor, handle)
    }
}

/// A [`ProcessHandler`] wrapper that provides channel-based communication.
///
/// Created via [`ControlledProcessorTrait::instance`].
pub struct ControlledProcessorInstance<T: ControlledProcessorTrait> {
    inner: T,
    channels: ProcessorChannels<T::Command, T::Notification>,
}

impl<T: ControlledProcessorTrait> ProcessHandler for ControlledProcessorInstance<T> {
    fn process(&mut self, client: &Client, scope: &ProcessScope) -> Control {
        self.inner.process(client, scope, &mut self.channels)
    }

    const SLOW_SYNC: bool = T::SLOW_SYNC;

    fn buffer_size(&mut self, client: &Client, size: Frames) -> Control {
        self.inner.buffer_size(client, size, &mut self.channels)
    }

    fn sync(
        &mut self,
        client: &Client,
        state: crate::TransportState,
        pos: &crate::TransportPosition,
    ) -> bool {
        self.inner.sync(client, state, pos, &mut self.channels)
    }
}
