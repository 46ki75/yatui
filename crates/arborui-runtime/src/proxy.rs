use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use crate::scheduler::WakeSignal;

/// Reason an external application message was not accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventProxySendErrorKind {
    /// The configured ingress capacity was already occupied.
    Full,
    /// The application runner no longer accepts external messages.
    Closed,
}

/// Error returned when an external application message is not accepted.
pub struct EventProxySendError<Message> {
    kind: EventProxySendErrorKind,
    message: Message,
}

impl<Message> EventProxySendError<Message> {
    /// Returns why the message was not accepted.
    #[must_use]
    pub const fn kind(&self) -> EventProxySendErrorKind {
        self.kind
    }

    /// Recovers the message that was not accepted.
    #[must_use]
    pub fn into_inner(self) -> Message {
        self.message
    }
}

impl<Message> fmt::Debug for EventProxySendError<Message> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventProxySendError")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl<Message> fmt::Display for EventProxySendError<Message> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            EventProxySendErrorKind::Full => {
                formatter.write_str("application event ingress is full")
            }
            EventProxySendErrorKind::Closed => {
                formatter.write_str("application runner is no longer receiving messages")
            }
        }
    }
}

impl<Message> std::error::Error for EventProxySendError<Message> {}

/// Snapshot of bounded external-message ingress state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventIngressMetrics {
    /// Maximum number of accepted messages waiting for the runner.
    pub capacity: usize,
    /// Number of accepted messages currently waiting for the runner.
    pub depth: usize,
    /// Largest observed waiting depth since runner construction.
    pub high_water_mark: usize,
    /// Number of new messages rejected because ingress was full.
    pub rejected: u64,
    /// Saturating number of messages accepted since runner construction.
    pub accepted: u64,
    /// Saturating number removed for serialized application update.
    pub dequeued: u64,
    /// Saturating sum of admission-to-dequeue latency for dequeued messages.
    pub total_queue_latency: Duration,
    /// Largest admission-to-dequeue latency observed so far.
    pub max_queue_latency: Duration,
    /// Whether the runner has stopped accepting external messages.
    pub closed: bool,
}

struct EventIngress<Message> {
    state: Mutex<EventIngressState<Message>>,
    wake: Arc<WakeSignal>,
    clock: IngressClock,
}

enum IngressClock {
    System(Instant),
    #[cfg(test)]
    Manual(Arc<std::sync::atomic::AtomicU64>),
}

impl IngressClock {
    fn new() -> Self {
        Self::System(Instant::now())
    }

    fn now(&self) -> Duration {
        match self {
            Self::System(origin) => origin.elapsed(),
            #[cfg(test)]
            Self::Manual(elapsed_nanos) => {
                Duration::from_nanos(elapsed_nanos.load(std::sync::atomic::Ordering::Acquire))
            }
        }
    }
}

struct EventIngressState<Message> {
    queue: VecDeque<QueuedMessage<Message>>,
    capacity: usize,
    high_water_mark: usize,
    rejected: u64,
    accepted: u64,
    dequeued: u64,
    total_queue_latency: Duration,
    max_queue_latency: Duration,
    closed: bool,
}

struct QueuedMessage<Message> {
    message: Message,
    admitted_at: Duration,
}

impl<Message> EventIngress<Message> {
    fn state(&self) -> MutexGuard<'_, EventIngressState<Message>> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// A thread-safe, cloneable, bounded application message sender.
pub struct EventProxy<Message> {
    ingress: Arc<EventIngress<Message>>,
}

impl<Message> EventProxy<Message> {
    /// Attempts to enqueue one external message without blocking.
    ///
    /// A successful send wakes a runner waiting through its scheduler. If the
    /// configured capacity is occupied, the new message is rejected without
    /// changing the existing FIFO queue. A runner currently inside a synchronous
    /// terminal backend poll observes accepted work when that poll's configured
    /// timeout expires.
    pub fn send(&self, message: Message) -> Result<(), EventProxySendError<Message>> {
        {
            let mut state = self.ingress.state();
            if state.closed {
                return Err(EventProxySendError {
                    kind: EventProxySendErrorKind::Closed,
                    message,
                });
            }
            if state.queue.len() >= state.capacity {
                state.rejected = state.rejected.saturating_add(1);
                return Err(EventProxySendError {
                    kind: EventProxySendErrorKind::Full,
                    message,
                });
            }
            state.queue.push_back(QueuedMessage {
                message,
                admitted_at: self.ingress.clock.now(),
            });
            state.accepted = state.accepted.saturating_add(1);
            state.high_water_mark = state.high_water_mark.max(state.queue.len());
        }
        self.ingress.wake.notify();
        Ok(())
    }

    /// Returns an instantaneous snapshot of shared ingress pressure.
    #[must_use]
    pub fn metrics(&self) -> EventIngressMetrics {
        let state = self.ingress.state();
        EventIngressMetrics {
            capacity: state.capacity,
            depth: state.queue.len(),
            high_water_mark: state.high_water_mark,
            rejected: state.rejected,
            accepted: state.accepted,
            dequeued: state.dequeued,
            total_queue_latency: state.total_queue_latency,
            max_queue_latency: state.max_queue_latency,
            closed: state.closed,
        }
    }
}

impl<Message> Clone for EventProxy<Message> {
    fn clone(&self) -> Self {
        Self {
            ingress: Arc::clone(&self.ingress),
        }
    }
}

impl<Message> fmt::Debug for EventProxy<Message> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventProxy")
            .field("metrics", &self.metrics())
            .finish()
    }
}

pub(crate) struct EventReceiver<Message> {
    ingress: Arc<EventIngress<Message>>,
}

impl<Message> EventReceiver<Message> {
    pub(crate) fn receive(&self) -> Option<Message> {
        let mut state = self.ingress.state();
        let now = self.ingress.clock.now();
        let queued = state.queue.pop_front()?;
        let latency = now.saturating_sub(queued.admitted_at);
        state.dequeued = state.dequeued.saturating_add(1);
        state.total_queue_latency = state.total_queue_latency.saturating_add(latency);
        state.max_queue_latency = state.max_queue_latency.max(latency);
        Some(queued.message)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ingress.state().queue.is_empty()
    }

    pub(crate) fn close(&self) {
        let pending = {
            let mut state = self.ingress.state();
            state.closed = true;
            std::mem::take(&mut state.queue)
        };
        drop(pending);
    }
}

impl<Message> Drop for EventReceiver<Message> {
    fn drop(&mut self) {
        self.close();
    }
}

pub(crate) fn event_channel<Message>(
    capacity: usize,
    wake: Arc<WakeSignal>,
) -> (EventProxy<Message>, EventReceiver<Message>) {
    event_channel_with_clock(capacity, wake, IngressClock::new())
}

fn event_channel_with_clock<Message>(
    capacity: usize,
    wake: Arc<WakeSignal>,
    clock: IngressClock,
) -> (EventProxy<Message>, EventReceiver<Message>) {
    debug_assert!(capacity > 0);
    let ingress = Arc::new(EventIngress {
        state: Mutex::new(EventIngressState {
            queue: VecDeque::new(),
            capacity,
            high_water_mark: 0,
            rejected: 0,
            accepted: 0,
            dequeued: 0,
            total_queue_latency: Duration::ZERO,
            max_queue_latency: Duration::ZERO,
            closed: false,
        }),
        wake,
        clock,
    });
    (
        EventProxy {
            ingress: Arc::clone(&ingress),
        },
        EventReceiver { ingress },
    )
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc, Barrier,
            atomic::{AtomicU64, Ordering},
        },
        thread,
        time::Duration,
    };

    use super::*;

    fn test_event_channel<Message>(
        capacity: usize,
        wake: Arc<WakeSignal>,
    ) -> (EventProxy<Message>, EventReceiver<Message>) {
        event_channel(capacity, wake)
    }

    struct ManualClock {
        elapsed_nanos: Arc<AtomicU64>,
    }

    impl Default for ManualClock {
        fn default() -> Self {
            Self {
                elapsed_nanos: Arc::new(AtomicU64::new(0)),
            }
        }
    }

    impl ManualClock {
        fn advance(&self, duration: Duration) {
            let nanos = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
            let _ =
                self.elapsed_nanos
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |elapsed| {
                        Some(elapsed.saturating_add(nanos))
                    });
        }

        fn source(&self) -> IngressClock {
            IngressClock::Manual(Arc::clone(&self.elapsed_nanos))
        }
    }

    struct ReentrantDrop {
        proxy: EventProxy<Self>,
    }

    impl Drop for ReentrantDrop {
        fn drop(&mut self) {
            let _metrics = self.proxy.metrics();
        }
    }

    #[test]
    fn full_ingress_rejects_new_message_and_recovers_ownership() {
        let wake = Arc::new(WakeSignal::new());
        let (proxy, receiver) = test_event_channel(2, wake);

        assert!(proxy.send(String::from("first")).is_ok());
        assert!(proxy.send(String::from("second")).is_ok());
        let error = proxy
            .send(String::from("rejected"))
            .expect_err("third message should exceed capacity");

        assert_eq!(error.kind(), EventProxySendErrorKind::Full);
        assert_eq!(error.into_inner(), "rejected");
        assert_eq!(
            proxy.metrics(),
            EventIngressMetrics {
                capacity: 2,
                depth: 2,
                high_water_mark: 2,
                rejected: 1,
                accepted: 2,
                dequeued: 0,
                total_queue_latency: Duration::ZERO,
                max_queue_latency: Duration::ZERO,
                closed: false,
            }
        );
        assert_eq!(receiver.receive().as_deref(), Some("first"));
        assert_eq!(receiver.receive().as_deref(), Some("second"));
    }

    #[test]
    fn clones_share_capacity_and_processing_frees_a_slot() {
        let wake = Arc::new(WakeSignal::new());
        let (proxy, receiver) = test_event_channel(1, Arc::clone(&wake));
        let clone = proxy.clone();

        assert!(proxy.send(1).is_ok());
        assert!(wake.is_notified());
        wake.wait(std::time::Duration::ZERO);
        let error = clone
            .send(2)
            .expect_err("cloned proxies should share capacity");
        assert!(!wake.is_notified());
        assert_eq!(receiver.receive(), Some(1));
        assert!(clone.send(error.into_inner()).is_ok());
        assert_eq!(receiver.receive(), Some(2));
        assert_eq!(proxy.metrics().high_water_mark, 1);
        assert_eq!(proxy.metrics().rejected, 1);
    }

    #[test]
    fn dropping_receiver_closes_ingress_and_recovers_message() {
        let wake = Arc::new(WakeSignal::new());
        let (proxy, receiver) = test_event_channel(1, wake);
        drop(receiver);

        let error = proxy
            .send(String::from("not delivered"))
            .expect_err("closed ingress should reject messages");

        assert_eq!(error.kind(), EventProxySendErrorKind::Closed);
        assert_eq!(error.into_inner(), "not delivered");
        assert!(proxy.metrics().closed);
    }

    #[test]
    fn closing_drops_pending_messages_outside_the_ingress_lock() {
        let wake = Arc::new(WakeSignal::new());
        let (proxy, receiver) = test_event_channel(1, wake);
        assert!(
            proxy
                .send(ReentrantDrop {
                    proxy: proxy.clone(),
                })
                .is_ok()
        );

        drop(receiver);

        assert!(proxy.metrics().closed);
        assert_eq!(proxy.metrics().depth, 0);
    }

    #[test]
    fn concurrent_producers_compete_for_one_shared_slot() {
        let wake = Arc::new(WakeSignal::new());
        let (proxy, receiver) = test_event_channel(1, wake);
        let start = Arc::new(Barrier::new(3));
        let first_start = Arc::clone(&start);
        let first_proxy = proxy.clone();
        let first = thread::spawn(move || {
            first_start.wait();
            first_proxy.send(1)
        });
        let second_start = Arc::clone(&start);
        let second_proxy = proxy.clone();
        let second = thread::spawn(move || {
            second_start.wait();
            second_proxy.send(2)
        });

        start.wait();
        let first = first
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
        let second = second
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic));

        assert_ne!(first.is_ok(), second.is_ok());
        let rejected = first.err().or_else(|| second.err());
        assert!(rejected.is_some_and(|error| error.kind() == EventProxySendErrorKind::Full));
        assert!(matches!(receiver.receive(), Some(1 | 2)));
        assert_eq!(proxy.metrics().high_water_mark, 1);
        assert_eq!(proxy.metrics().rejected, 1);
    }

    #[test]
    fn metrics_record_admission_to_dequeue_latency() {
        let clock = ManualClock::default();
        let (proxy, receiver) =
            event_channel_with_clock(2, Arc::new(WakeSignal::new()), clock.source());

        assert!(proxy.send("first").is_ok());
        clock.advance(Duration::from_millis(5));
        assert!(proxy.send("second").is_ok());
        clock.advance(Duration::from_millis(2));

        assert_eq!(receiver.receive(), Some("first"));
        assert_eq!(receiver.receive(), Some("second"));
        assert_eq!(
            proxy.metrics(),
            EventIngressMetrics {
                capacity: 2,
                depth: 0,
                high_water_mark: 2,
                rejected: 0,
                accepted: 2,
                dequeued: 2,
                total_queue_latency: Duration::from_millis(9),
                max_queue_latency: Duration::from_millis(7),
                closed: false,
            }
        );
    }
}
