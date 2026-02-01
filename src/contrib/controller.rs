//! Utilities for building controllable JACK processors with lock-free communication.

use rtrb::{Consumer, Producer, RingBuffer};

use crate::{
    Client, Control, Frames, ProcessHandler, ProcessScope, TransportPosition, TransportState,
};

/// Communication channels available to a processor in the real-time audio thread.
pub struct ProcessorChannels<Command, Notification> {
    notifications: Producer<Notification>,
    commands: Consumer<Command>,
}

impl<Command, Notification> ProcessorChannels<Command, Notification> {
    /// Drain and return an iterator over all pending commands.
    pub fn drain_commands(&mut self) -> impl Iterator<Item = Command> + '_ {
        std::iter::from_fn(move || self.commands.pop().ok())
    }

    /// Try to send a notification.
    ///
    /// Returns `Ok(())` if the notification was sent, or `Err(notification)` if the buffer was full.
    pub fn try_notify(&mut self, notification: Notification) -> Result<(), Notification> {
        self.notifications
            .push(notification)
            .map_err(|rtrb::PushError::Full(n)| n)
    }
}

/// Handle for controlling a processor from outside the real-time audio thread.
pub struct Controller<Command, Notification> {
    notifications: Consumer<Notification>,
    commands: Producer<Command>,
}

impl<Command, Notification> Controller<Command, Notification> {
    /// Try to send a command to the processor.
    ///
    /// Returns `Ok(())` if the command was sent, or `Err(command)` if the buffer was full.
    pub fn send_command(&mut self, command: Command) -> Result<(), Command> {
        self.commands
            .push(command)
            .map_err(|rtrb::PushError::Full(cmd)| cmd)
    }

    /// Try to receive a notification from the processor.
    ///
    /// Returns `Some(notification)` if one was available, or `None` if the buffer was empty.
    pub fn recv_notification(&mut self) -> Option<Notification> {
        self.notifications.pop().ok()
    }

    /// Drain and return an iterator over all pending notifications.
    pub fn drain_notifications(&mut self) -> impl Iterator<Item = Notification> + '_ {
        std::iter::from_fn(move || self.notifications.pop().ok())
    }
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
    #[must_use = "the processor instance must be used with Client::activate"]
    fn instance(
        self,
        notification_channel_size: usize,
        command_channel_size: usize,
    ) -> (
        ControlledProcessorInstance<Self>,
        Controller<Self::Command, Self::Notification>,
    ) {
        let (notifications, notifications_other) =
            RingBuffer::<Self::Notification>::new(notification_channel_size);
        let (commands_other, commands) = RingBuffer::<Self::Command>::new(command_channel_size);
        let controller = Controller {
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
        (processor, controller)
    }
}

/// A [`ProcessHandler`] wrapper that provides channel-based communication.
///
/// Created via [`ControlledProcessorTrait::instance`].
pub struct ControlledProcessorInstance<T: ControlledProcessorTrait> {
    /// The inner processor implementation.
    pub inner: T,
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

    fn sync(&mut self, client: &Client, state: TransportState, pos: &TransportPosition) -> bool {
        self.inner.sync(client, state, pos, &mut self.channels)
    }
}
