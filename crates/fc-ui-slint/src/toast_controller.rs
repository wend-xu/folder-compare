//! Policy-only toast queueing strategy used by the UI orchestration layer.

use std::collections::VecDeque;
use std::time::Duration;

pub const DEFAULT_TOAST_DURATION: Duration = Duration::from_millis(2000);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastPlacement {
    Toast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ToastTone {
    Success,
    Warn,
    Error,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ToastStrategy {
    Replace,
    Queue,
    Auto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastRequest {
    pub message: String,
    pub tone: ToastTone,
    pub placement: ToastPlacement,
    pub duration: Duration,
    pub strategy: ToastStrategy,
}

impl ToastRequest {
    pub fn new(message: impl Into<String>, tone: ToastTone, placement: ToastPlacement) -> Self {
        Self {
            message: message.into(),
            tone,
            placement,
            duration: DEFAULT_TOAST_DURATION,
            strategy: ToastStrategy::Auto,
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = if duration.is_zero() {
            DEFAULT_TOAST_DURATION
        } else {
            duration
        };
        self
    }

    pub fn with_strategy(mut self, strategy: ToastStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    fn has_same_content(&self, other: &Self) -> bool {
        self.message == other.message
            && self.tone == other.tone
            && self.placement == other.placement
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueDispatchResult {
    pub active: Option<ToastRequest>,
    pub reset_timer: bool,
}

#[derive(Debug, Default, Clone)]
pub struct ToastQueueState {
    current: Option<ToastRequest>,
    pending: VecDeque<ToastRequest>,
}

impl ToastQueueState {
    pub fn enqueue(&mut self, request: ToastRequest) -> QueueDispatchResult {
        let Some(current) = self.current.as_ref() else {
            self.current = Some(request.clone());
            return QueueDispatchResult {
                active: Some(request),
                reset_timer: true,
            };
        };

        match request.strategy {
            ToastStrategy::Replace => {
                self.current = Some(request.clone());
                QueueDispatchResult {
                    active: Some(request),
                    reset_timer: true,
                }
            }
            ToastStrategy::Queue => {
                self.pending.push_back(request);
                QueueDispatchResult {
                    active: self.current.clone(),
                    reset_timer: false,
                }
            }
            ToastStrategy::Auto => {
                if request.has_same_content(current) {
                    self.current = Some(request.clone());
                    QueueDispatchResult {
                        active: Some(request),
                        reset_timer: true,
                    }
                } else {
                    self.pending.push_back(request);
                    QueueDispatchResult {
                        active: self.current.clone(),
                        reset_timer: false,
                    }
                }
            }
        }
    }

    pub fn advance_after_timeout(&mut self) -> Option<ToastRequest> {
        self.current = self.pending.pop_front();
        self.current.clone()
    }

    #[cfg(test)]
    pub fn current(&self) -> Option<&ToastRequest> {
        self.current.as_ref()
    }

    #[cfg(test)]
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_strategy_keeps_order_until_timeout() {
        let mut queue = ToastQueueState::default();
        let first = ToastRequest::new("A", ToastTone::Info, ToastPlacement::Toast)
            .with_strategy(ToastStrategy::Queue);
        let second = ToastRequest::new("B", ToastTone::Warn, ToastPlacement::Toast)
            .with_strategy(ToastStrategy::Queue);

        let first_result = queue.enqueue(first.clone());
        assert_eq!(first_result.active, Some(first));
        assert!(first_result.reset_timer);

        let second_result = queue.enqueue(second.clone());
        assert!(!second_result.reset_timer);
        assert_eq!(
            queue
                .current()
                .expect("first toast should remain active")
                .message,
            "A"
        );
        assert_eq!(queue.pending_len(), 1);

        let promoted = queue
            .advance_after_timeout()
            .expect("queued toast should become active");
        assert_eq!(promoted.message, "B");
        assert_eq!(queue.pending_len(), 0);
    }

    #[test]
    fn replace_strategy_overrides_current_immediately() {
        let mut queue = ToastQueueState::default();
        queue.enqueue(
            ToastRequest::new("A", ToastTone::Info, ToastPlacement::Toast)
                .with_strategy(ToastStrategy::Queue),
        );

        let replace_result = queue.enqueue(
            ToastRequest::new("B", ToastTone::Error, ToastPlacement::Toast)
                .with_strategy(ToastStrategy::Replace),
        );

        assert!(replace_result.reset_timer);
        assert_eq!(
            queue
                .current()
                .expect("replacement toast should be active")
                .message,
            "B"
        );
        assert_eq!(queue.pending_len(), 0);
    }

    #[test]
    fn auto_strategy_replaces_same_content_and_resets_timer() {
        let mut queue = ToastQueueState::default();
        queue.enqueue(
            ToastRequest::new("Copied", ToastTone::Info, ToastPlacement::Toast)
                .with_duration(Duration::from_millis(1000)),
        );

        let auto_result = queue.enqueue(
            ToastRequest::new("Copied", ToastTone::Info, ToastPlacement::Toast)
                .with_duration(Duration::from_millis(3600))
                .with_strategy(ToastStrategy::Auto),
        );

        assert!(auto_result.reset_timer);
        assert_eq!(queue.pending_len(), 0);
        assert_eq!(
            queue
                .current()
                .expect("auto replacement should keep one active toast")
                .duration,
            Duration::from_millis(3600)
        );
    }
}
