use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::sync::OnceLock;
use tokio::sync::broadcast;
use uuid::Uuid;

// ─── Standard Event Types ───

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    SessionCreated,
    SessionClosed,
    TurnStarted,
    TurnDelta,
    TurnReasoningDelta,
    TurnToolStarted,
    TurnToolDelta,
    TurnToolFinished,
    TurnFileChanged,
    TurnArtifactGenerated,
    TurnWarning,
    TurnError,
    TurnCompleted,
    TurnFailed,
    TurnCancelled,
    TurnTimeout,
    RuntimeStarted,
    RuntimeHealthChanged,
    RuntimeCapabilityChanged,
    RuntimeExited,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionCreated => "session.created",
            Self::SessionClosed => "session.closed",
            Self::TurnStarted => "turn.started",
            Self::TurnDelta => "turn.delta",
            Self::TurnReasoningDelta => "turn.reasoning_delta",
            Self::TurnToolStarted => "turn.tool_started",
            Self::TurnToolDelta => "turn.tool_delta",
            Self::TurnToolFinished => "turn.tool_finished",
            Self::TurnFileChanged => "turn.file_changed",
            Self::TurnArtifactGenerated => "turn.artifact_generated",
            Self::TurnWarning => "turn.warning",
            Self::TurnError => "turn.error",
            Self::TurnCompleted => "turn.completed",
            Self::TurnFailed => "turn.failed",
            Self::TurnCancelled => "turn.cancelled",
            Self::TurnTimeout => "turn.timeout",
            Self::RuntimeStarted => "runtime.started",
            Self::RuntimeHealthChanged => "runtime.health_changed",
            Self::RuntimeCapabilityChanged => "runtime.capability_changed",
            Self::RuntimeExited => "runtime.exited",
        }
    }
}

// ─── Standard Event ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardEvent {
    pub event_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub runtime: String,
    pub event_type: String,
    pub timestamp: String,
    pub payload: serde_json::Value,
}

impl StandardEvent {
    fn new(
        session_id: &str,
        turn_id: &str,
        runtime: &str,
        event_type: EventType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            session_id: session_id.into(),
            turn_id: turn_id.into(),
            runtime: runtime.into(),
            event_type: event_type.as_str().into(),
            timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            payload,
        }
    }

    pub fn session_created(session_id: &str, runtime: &str) -> Self {
        Self::new(
            session_id,
            "",
            runtime,
            EventType::SessionCreated,
            serde_json::json!({}),
        )
    }

    pub fn session_closed(session_id: &str) -> Self {
        Self::new(
            session_id,
            "",
            "",
            EventType::SessionClosed,
            serde_json::json!({}),
        )
    }

    pub fn turn_started(session_id: &str, turn_id: &str, runtime: &str) -> Self {
        Self::new(
            session_id,
            turn_id,
            runtime,
            EventType::TurnStarted,
            serde_json::json!({}),
        )
    }

    pub fn turn_delta(session_id: &str, turn_id: &str, text: &str, runtime: &str) -> Self {
        Self::new(
            session_id,
            turn_id,
            runtime,
            EventType::TurnDelta,
            serde_json::json!({"text": text}),
        )
    }

    pub fn turn_tool_started(session_id: &str, turn_id: &str, tool: &str, runtime: &str) -> Self {
        Self::new(
            session_id,
            turn_id,
            runtime,
            EventType::TurnToolStarted,
            serde_json::json!({"tool": tool}),
        )
    }

    pub fn turn_tool_finished(session_id: &str, turn_id: &str, tool: &str, runtime: &str) -> Self {
        Self::new(
            session_id,
            turn_id,
            runtime,
            EventType::TurnToolFinished,
            serde_json::json!({"tool": tool}),
        )
    }

    pub fn turn_file_changed(
        session_id: &str,
        turn_id: &str,
        path: &str,
        change_type: &str,
        runtime: &str,
    ) -> Self {
        Self::new(
            session_id,
            turn_id,
            runtime,
            EventType::TurnFileChanged,
            serde_json::json!({"path": path, "change_type": change_type}),
        )
    }

    pub fn turn_completed(session_id: &str, turn_id: &str) -> Self {
        Self::new(
            session_id,
            turn_id,
            "",
            EventType::TurnCompleted,
            serde_json::json!({}),
        )
    }

    pub fn turn_failed(session_id: &str, turn_id: &str, error: &str) -> Self {
        Self::new(
            session_id,
            turn_id,
            "",
            EventType::TurnFailed,
            serde_json::json!({"error": error}),
        )
    }

    pub fn turn_error(session_id: &str, turn_id: &str, error: &str, runtime: &str) -> Self {
        Self::new(
            session_id,
            turn_id,
            runtime,
            EventType::TurnError,
            serde_json::json!({"error": error}),
        )
    }

    pub fn runtime_health_changed(runtime: &str, available: bool) -> Self {
        Self::new(
            "",
            "",
            runtime,
            EventType::RuntimeHealthChanged,
            serde_json::json!({"available": available}),
        )
    }

    pub fn to_sse(&self) -> String {
        format!(
            "event: {}\ndata: {}\n\n",
            self.event_type,
            serde_json::to_string(self).unwrap_or_default()
        )
    }
}

// ─── Event Query Result ───

pub type EventQuery = StandardEvent;

// ─── Event Bus ───

/// In-process event bus with tokio broadcast for fan-out to SSE clients.
/// Events are stored in an in-memory backlog for late-join/replay and
/// broadcast to all active subscribers in real-time.
pub struct EventBus {
    tx: broadcast::Sender<StandardEvent>,
    backlog: Mutex<Vec<StandardEvent>>,
}

static EVENT_BUS: OnceLock<EventBus> = OnceLock::new();

impl EventBus {
    pub fn global() -> &'static EventBus {
        EVENT_BUS.get_or_init(|| EventBus {
            tx: broadcast::channel(1024).0,
            backlog: Mutex::new(Vec::new()),
        })
    }

    /// Publish an event to all subscribers and persist to the backlog.
    pub fn publish(&self, event: StandardEvent) {
        // Store in backlog
        if let Ok(mut backlog) = self.backlog.lock() {
            if backlog.len() > 10_000 {
                let split_point = backlog.len() - 5_000;
                *backlog = backlog.split_off(split_point);
            }
            backlog.push(event.clone());
        }
        // Broadcast to all subscribers (non-blocking; ignored if no receivers)
        let _ = self.tx.send(event);
    }

    /// Subscribe to new events via broadcast channel.
    /// Returns a receiver that gets every event published after this call.
    pub fn subscribe(&self) -> broadcast::Receiver<StandardEvent> {
        self.tx.subscribe()
    }

    /// Query events for a specific turn (for replay / late join).
    pub fn query_by_turn(&self, turn_id: &str) -> Vec<StandardEvent> {
        if let Ok(backlog) = self.backlog.lock() {
            backlog
                .iter()
                .filter(|e| e.turn_id == turn_id)
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    /// Query events for a session.
    pub fn query_by_session(&self, session_id: &str) -> Vec<StandardEvent> {
        if let Ok(backlog) = self.backlog.lock() {
            backlog
                .iter()
                .filter(|e| e.session_id == session_id)
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    /// Get events after a specific event_id (for Last-Event-Id replay).
    pub fn query_after(&self, last_event_id: &str) -> Vec<StandardEvent> {
        if let Ok(backlog) = self.backlog.lock() {
            let maybe_idx = backlog.iter().position(|e| e.event_id == last_event_id);
            match maybe_idx {
                Some(idx) => backlog[(idx + 1)..].to_vec(),
                None => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Count of events in the backlog.
    #[allow(dead_code)]
    pub fn backlog_len(&self) -> usize {
        self.backlog.lock().map(|b| b.len()).unwrap_or(0)
    }
}

// ─── SSE Axum handler ───

use axum::{
    extract::Path,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// SSE handler that replays existing turn events then streams new ones in real-time.
///
/// The handler first replays all events for the given turn from the backlog,
/// then subscribes to the broadcast channel for live events filtered to this turn.
pub async fn sse_handler(
    Path((_session_id, turn_id)): Path<(String, String)>,
) -> Sse<impl stream::Stream<Item = Result<Event, Infallible>>> {
    let bus = EventBus::global();

    // Subscribe BEFORE querying backlog to avoid race-condition gaps.
    // Events published between subscribe and query will be in both backlog and
    // broadcast stream — near-instant replay means duplicates are unlikely in practice.
    let rx = bus.subscribe();
    let existing = bus.query_by_turn(&turn_id);

    // Phase 1: replay existing events
    let replay = stream::iter(existing.into_iter().map(event_to_sse));

    // Phase 2: stream live events, filtered to this turn
    let turn_id_clone = turn_id.clone();
    let live = BroadcastStream::new(rx).filter_map(move |result| {
        match result {
            Ok(evt) if evt.turn_id == turn_id_clone => Some(event_to_sse(evt)),
            Ok(_) => None, // skip events for other turns
            Err(BroadcastStreamRecvError::Lagged(skipped)) => {
                Some(Ok(Event::default()
                    .event("turn.warning")
                    .data(serde_json::json!({
                        "event_id": Uuid::new_v4().to_string(),
                        "event_type": "turn.warning",
                        "payload": {"message": format!("Event stream lagged, {} events skipped", skipped)}
                    }).to_string())))
            }
        }
    });

    let combined = replay.chain(live);
    Sse::new(combined).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

fn event_to_sse(evt: StandardEvent) -> Result<Event, Infallible> {
    Ok(Event::default()
        .event(evt.event_type.clone())
        .data(serde_json::to_string(&evt).unwrap_or_default()))
}
