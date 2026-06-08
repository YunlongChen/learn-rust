//! Lifecycle state machine for agent state transitions
//!
//! This module implements a state machine that manages agent lifecycle states
//! and handles state transitions based on lifecycle events.

use domain_agent_protocol::lifecycle::{LifecycleEventType, LifecycleEvent as ProtocolLifecycleEvent};
use thiserror::Error;

/// Represents the possible lifecycle states of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentLifecycleState {
    /// Agent has been created but not yet started registration
    Created,
    /// Agent is in the process of registering
    Pending,
    /// Agent has been authorized but not yet connected
    Authorized,
    /// Agent is connected but not yet registered with the system
    Connected,
    /// Agent is fully registered and operational
    Registered,
    /// Agent is attempting to reconnect after a disconnection
    Reconnecting,
    /// Agent has been closed/terminated
    Closed,
}

impl std::fmt::Display for AgentLifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentLifecycleState::Created => write!(f, "Created"),
            AgentLifecycleState::Pending => write!(f, "Pending"),
            AgentLifecycleState::Authorized => write!(f, "Authorized"),
            AgentLifecycleState::Connected => write!(f, "Connected"),
            AgentLifecycleState::Registered => write!(f, "Registered"),
            AgentLifecycleState::Reconnecting => write!(f, "Reconnecting"),
            AgentLifecycleState::Closed => write!(f, "Closed"),
        }
    }
}

/// Errors that can occur during state transitions.
#[derive(Debug, Error)]
pub enum StateTransitionError {
    #[error("Invalid transition from {from} via event {event}")]
    InvalidTransition { from: AgentLifecycleState, event: LifecycleEventType },

    #[error("Agent is in terminal state {state} and cannot transition")]
    TerminalState { state: AgentLifecycleState },
}

/// Result type for state transition operations.
pub type TransitionResult = Result<Option<AgentLifecycleState>, StateTransitionError>;

/// Result type for handle_event which returns the event type for audit logging.
pub type HandleEventResult = Result<Option<LifecycleEventType>, StateTransitionError>;

/// State machine for managing agent lifecycle transitions.
///
/// The state machine enforces valid state transitions based on lifecycle events
/// and tracks the current state of an agent.
#[derive(Debug, Clone)]
pub struct LifecycleStateMachine {
    current_state: AgentLifecycleState,
}

impl LifecycleStateMachine {
    /// Creates a new state machine in the Created state.
    pub fn new() -> Self {
        Self {
            current_state: AgentLifecycleState::Created,
        }
    }

    /// Creates a state machine with an initial state (useful for loading existing agents).
    pub fn with_state(state: AgentLifecycleState) -> Self {
        Self { current_state: state }
    }

    /// Returns the current state of the state machine.
    pub fn current_state(&self) -> AgentLifecycleState {
        self.current_state
    }

    /// Handles a lifecycle event and performs the appropriate state transition.
    ///
    /// Returns the event type that should be recorded for audit logging,
    /// or None if no transition occurred.
    ///
    /// # Arguments
    ///
    /// * `event` - The lifecycle event to process
    ///
    /// # Errors
    ///
    /// Returns an error if the transition is invalid or the agent is in a terminal state.
    pub fn handle_event(&mut self, event: &ProtocolLifecycleEvent) -> HandleEventResult {
        // Terminal states cannot transition
        if self.current_state == AgentLifecycleState::Closed {
            return Err(StateTransitionError::TerminalState {
                state: self.current_state,
            });
        }

        let event_type = event.event_type;
        let new_state = self.transition(event_type)?;

        if let Some(new_state) = new_state {
            self.current_state = new_state;
        }

        Ok(Some(event_type))
    }

    /// Determines the next state based on the current state and event type.
    ///
    /// Returns the new state if a transition occurred, or None if no transition.
    fn transition(&self, event_type: LifecycleEventType) -> TransitionResult {
        match (self.current_state, event_type) {
            // Created -> Pending
            (AgentLifecycleState::Created, LifecycleEventType::AgentRegistering) => {
                Ok(Some(AgentLifecycleState::Pending))
            }
            // Pending -> Authorized or Closed
            (AgentLifecycleState::Pending, LifecycleEventType::AgentApproved) => {
                Ok(Some(AgentLifecycleState::Authorized))
            }
            (AgentLifecycleState::Pending, LifecycleEventType::AgentDenied) => {
                Ok(Some(AgentLifecycleState::Closed))
            }
            // Authorized -> Connected
            (AgentLifecycleState::Authorized, LifecycleEventType::AgentConnected) => {
                Ok(Some(AgentLifecycleState::Connected))
            }
            // Connected -> Registered
            (AgentLifecycleState::Connected, LifecycleEventType::AgentRegistered) => {
                Ok(Some(AgentLifecycleState::Registered))
            }
            // Registered -> Reconnecting
            (AgentLifecycleState::Registered, LifecycleEventType::AgentReconnecting) => {
                Ok(Some(AgentLifecycleState::Reconnecting))
            }
            (AgentLifecycleState::Registered, LifecycleEventType::AgentDisconnected) => {
                Ok(Some(AgentLifecycleState::Reconnecting))
            }
            // Reconnecting -> Registered
            (AgentLifecycleState::Reconnecting, LifecycleEventType::AgentRegistered) => {
                Ok(Some(AgentLifecycleState::Registered))
            }
            // Reconnecting -> Closed
            (AgentLifecycleState::Reconnecting, LifecycleEventType::AgentClosed) => {
                Ok(Some(AgentLifecycleState::Closed))
            }
            // Any state -> Closed on error or close
            (_, LifecycleEventType::AgentError) => Ok(Some(AgentLifecycleState::Closed)),
            (_, LifecycleEventType::AgentClosed) => Ok(Some(AgentLifecycleState::Closed)),
            // Invalid transitions
            (state, event) => Err(StateTransitionError::InvalidTransition {
                from: state,
                event,
            }),
        }
    }
}

impl Default for LifecycleStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_agent_protocol::lifecycle::{EventSource, LifecycleEvent};
    use uuid::Uuid;

    fn create_event(agent_id: Uuid, event_type: LifecycleEventType) -> ProtocolLifecycleEvent {
        LifecycleEvent::new(agent_id, event_type, EventSource::Agent)
    }

    #[test]
    fn test_initial_state_is_created() {
        let sm = LifecycleStateMachine::new();
        assert_eq!(sm.current_state(), AgentLifecycleState::Created);
    }

    #[test]
    fn test_created_pending_transition() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::new();

        let event = create_event(agent_id, LifecycleEventType::AgentRegistering);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Pending);
        assert_eq!(result, Some(LifecycleEventType::AgentRegistering));
    }

    #[test]
    fn test_pending_authorized_transition() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Pending);

        let event = create_event(agent_id, LifecycleEventType::AgentApproved);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Authorized);
        assert_eq!(result, Some(LifecycleEventType::AgentApproved));
    }

    #[test]
    fn test_pending_closed_transition() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Pending);

        let event = create_event(agent_id, LifecycleEventType::AgentDenied);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Closed);
        assert_eq!(result, Some(LifecycleEventType::AgentDenied));
    }

    #[test]
    fn test_authorized_connected_transition() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Authorized);

        let event = create_event(agent_id, LifecycleEventType::AgentConnected);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Connected);
        assert_eq!(result, Some(LifecycleEventType::AgentConnected));
    }

    #[test]
    fn test_connected_registered_transition() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Connected);

        let event = create_event(agent_id, LifecycleEventType::AgentRegistered);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Registered);
        assert_eq!(result, Some(LifecycleEventType::AgentRegistered));
    }

    #[test]
    fn test_registered_reconnecting_on_disconnect() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Registered);

        let event = create_event(agent_id, LifecycleEventType::AgentDisconnected);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Reconnecting);
        assert_eq!(result, Some(LifecycleEventType::AgentDisconnected));
    }

    #[test]
    fn test_registered_reconnecting_on_reconnecting_event() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Registered);

        let event = create_event(agent_id, LifecycleEventType::AgentReconnecting);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Reconnecting);
        assert_eq!(result, Some(LifecycleEventType::AgentReconnecting));
    }

    #[test]
    fn test_reconnecting_registered_transition() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Reconnecting);

        let event = create_event(agent_id, LifecycleEventType::AgentRegistered);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Registered);
        assert_eq!(result, Some(LifecycleEventType::AgentRegistered));
    }

    #[test]
    fn test_reconnecting_closed_transition() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Reconnecting);

        let event = create_event(agent_id, LifecycleEventType::AgentClosed);
        let result = sm.handle_event(&event).unwrap();

        assert_eq!(sm.current_state(), AgentLifecycleState::Closed);
        assert_eq!(result, Some(LifecycleEventType::AgentClosed));
    }

    #[test]
    fn test_any_state_to_closed_on_error() {
        let agent_id = Uuid::new_v4();

        for initial_state in [
            AgentLifecycleState::Created,
            AgentLifecycleState::Pending,
            AgentLifecycleState::Authorized,
            AgentLifecycleState::Connected,
            AgentLifecycleState::Registered,
            AgentLifecycleState::Reconnecting,
        ] {
            let mut sm = LifecycleStateMachine::with_state(initial_state);
            let event = create_event(agent_id, LifecycleEventType::AgentError);

            let result = sm.handle_event(&event).unwrap();
            assert_eq!(sm.current_state(), AgentLifecycleState::Closed);
            assert_eq!(result, Some(LifecycleEventType::AgentError));
        }
    }

    #[test]
    fn test_any_state_to_closed_on_agent_closed() {
        let agent_id = Uuid::new_v4();

        for initial_state in [
            AgentLifecycleState::Created,
            AgentLifecycleState::Pending,
            AgentLifecycleState::Authorized,
            AgentLifecycleState::Connected,
            AgentLifecycleState::Registered,
            AgentLifecycleState::Reconnecting,
        ] {
            let mut sm = LifecycleStateMachine::with_state(initial_state);
            let event = create_event(agent_id, LifecycleEventType::AgentClosed);

            let result = sm.handle_event(&event).unwrap();
            assert_eq!(sm.current_state(), AgentLifecycleState::Closed);
            assert_eq!(result, Some(LifecycleEventType::AgentClosed));
        }
    }

    #[test]
    fn test_invalid_transition_returns_error() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Created);

        // Cannot approve before registering
        let event = create_event(agent_id, LifecycleEventType::AgentApproved);
        let result = sm.handle_event(&event);

        assert!(result.is_err());
        assert_eq!(sm.current_state(), AgentLifecycleState::Created);
    }

    #[test]
    fn test_closed_state_is_terminal() {
        let agent_id = Uuid::new_v4();
        let mut sm = LifecycleStateMachine::with_state(AgentLifecycleState::Closed);

        let event = create_event(agent_id, LifecycleEventType::AgentRegistered);
        let result = sm.handle_event(&event);

        assert!(result.is_err());
        assert_eq!(sm.current_state(), AgentLifecycleState::Closed);
    }

    #[test]
    fn test_state_display() {
        assert_eq!(AgentLifecycleState::Created.to_string(), "Created");
        assert_eq!(AgentLifecycleState::Pending.to_string(), "Pending");
        assert_eq!(AgentLifecycleState::Authorized.to_string(), "Authorized");
        assert_eq!(AgentLifecycleState::Connected.to_string(), "Connected");
        assert_eq!(AgentLifecycleState::Registered.to_string(), "Registered");
        assert_eq!(AgentLifecycleState::Reconnecting.to_string(), "Reconnecting");
        assert_eq!(AgentLifecycleState::Closed.to_string(), "Closed");
    }

    #[test]
    fn test_with_state() {
        let sm = LifecycleStateMachine::with_state(AgentLifecycleState::Registered);
        assert_eq!(sm.current_state(), AgentLifecycleState::Registered);
    }
}
