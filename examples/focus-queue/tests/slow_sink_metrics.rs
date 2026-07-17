#![allow(missing_docs)]
//! Opt-in input-to-write-complete latency report using the production serializer.

use std::{
    io::{self, Write},
    num::NonZeroUsize,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

use arborui::{
    AppRunner, CrosstermBackend, EventProxy, EventProxySendErrorKind, FramePatch, RenderTimings,
    RuntimeOptions, Size, TerminalBackend, TerminalEvent, TerminalRenderOutcome, TerminalSession,
    TerminalState, WriteOutcome, terminal::Capabilities,
};
use arborui_example_focus_queue::{FocusQueue, Message};

const SAMPLE_COUNT: usize = 64;
const INGRESS_CAPACITY: usize = 8;
const VIEWPORT: Size = Size::new(72, 18);
const BLOCK_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_ANSI_BYTES: usize = 15_000;
const MAX_WRITER_CALLS: usize = 10_000;
const MAX_AVERAGE_OVERHEAD: Duration = Duration::from_millis(5);
const MAX_P95_OVERHEAD: Duration = Duration::from_millis(10);
const MAX_SINGLE_OVERHEAD: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, Default)]
struct OutputMetrics {
    bytes: usize,
    writer_calls: usize,
    flushes: usize,
    patches: usize,
    full_repaints: usize,
    last_write_completed: Option<Instant>,
}

struct BlockingGate {
    armed: AtomicBool,
    entered: SyncSender<()>,
    release: Mutex<Receiver<()>>,
}

impl BlockingGate {
    fn new() -> (Arc<Self>, Receiver<()>, SyncSender<()>) {
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        (
            Arc::new(Self {
                armed: AtomicBool::new(false),
                entered: entered_tx,
                release: Mutex::new(release_rx),
            }),
            entered_rx,
            release_tx,
        )
    }

    fn arm(&self) {
        self.armed.store(true, Ordering::Release);
    }
}

struct SlowWriter {
    metrics: Arc<Mutex<OutputMetrics>>,
    delay: Duration,
    gate: Option<Arc<BlockingGate>>,
}

impl Write for SlowWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let mut metrics = lock_io(&self.metrics)?;
        metrics.bytes = metrics.bytes.saturating_add(buffer.len());
        metrics.writer_calls = metrics.writer_calls.saturating_add(1);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(gate) = &self.gate {
            if gate.armed.swap(false, Ordering::AcqRel) {
                gate.entered
                    .send(())
                    .map_err(|_| io::Error::other("blocked-write observer disconnected"))?;
                lock_io(&gate.release)?
                    .recv_timeout(BLOCK_TIMEOUT)
                    .map_err(|error| io::Error::new(io::ErrorKind::TimedOut, error))?;
            }
        }
        thread::sleep(self.delay);
        let mut metrics = lock_io(&self.metrics)?;
        metrics.flushes = metrics.flushes.saturating_add(1);
        metrics.last_write_completed = Some(Instant::now());
        Ok(())
    }
}

struct FixedBackend {
    inner: CrosstermBackend<SlowWriter>,
    capabilities: Capabilities,
    metrics: Arc<Mutex<OutputMetrics>>,
}

impl FixedBackend {
    fn new(delay: Duration, gate: Option<Arc<BlockingGate>>) -> io::Result<Self> {
        let capabilities = Capabilities::default();
        let metrics = Arc::new(Mutex::new(OutputMetrics::default()));
        let writer = SlowWriter {
            metrics: Arc::clone(&metrics),
            delay,
            gate,
        };
        let inner = CrosstermBackend::new(writer)?.with_capabilities(capabilities);
        Ok(Self {
            inner,
            capabilities,
            metrics,
        })
    }

    fn metrics(&self) -> OutputMetrics {
        *lock(&self.metrics)
    }
}

impl TerminalBackend for FixedBackend {
    type Error = io::Error;

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(VIEWPORT)
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    fn poll_event(&mut self, _timeout: Duration) -> Result<Option<TerminalEvent>, Self::Error> {
        Ok(None)
    }

    fn apply_state(&mut self, _desired: &TerminalState) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_patch(&mut self, patch: &FramePatch) -> Result<WriteOutcome, Self::Error> {
        let outcome = self.inner.write_patch(patch)?;
        let mut metrics = lock_io(&self.metrics)?;
        metrics.patches = metrics.patches.saturating_add(1);
        metrics.full_repaints = metrics
            .full_repaints
            .saturating_add(usize::from(patch.full_repaint));
        Ok(outcome)
    }

    fn restore(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PhaseTotals {
    update: Duration,
    render: Duration,
    view: Duration,
    staging: Duration,
    layout: Duration,
    paint: Duration,
    diff: Duration,
    terminal: Duration,
}

impl PhaseTotals {
    fn add(&mut self, update: Duration, render: RenderTimings) {
        self.update = self.update.saturating_add(update);
        self.render = self.render.saturating_add(render.total);
        self.view = self.view.saturating_add(render.view_construction);
        self.staging = self.staging.saturating_add(render.staging_reconciliation);
        self.layout = self.layout.saturating_add(render.layout);
        self.paint = self.paint.saturating_add(render.paint);
        self.diff = self.diff.saturating_add(render.diff);
        self.terminal = self.terminal.saturating_add(
            render
                .terminal_serialization_and_write
                .expect("each activity sample must write a nonempty patch"),
        );
    }
}

struct Report {
    sink_delay: Duration,
    average_input_to_write: Duration,
    p50_input_to_write: Duration,
    p95_input_to_write: Duration,
    max_input_to_write: Duration,
    average_queue: Duration,
    phases: PhaseTotals,
    output: OutputMetrics,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn lock_io<T>(mutex: &Mutex<T>) -> io::Result<MutexGuard<'_, T>> {
    mutex
        .lock()
        .map_err(|_| io::Error::other("slow-sink metrics lock poisoned"))
}

fn setup(
    delay: Duration,
    capacity: usize,
    gate: Option<Arc<BlockingGate>>,
) -> (
    AppRunner<FocusQueue>,
    TerminalSession<FixedBackend>,
    EventProxy<Message>,
    u64,
) {
    let backend = FixedBackend::new(delay, gate).expect("slow-sink backend must initialize");
    let mut session = TerminalSession::open(backend, TerminalState::default())
        .expect("slow-sink terminal session must open");
    let options = RuntimeOptions::new().with_event_ingress_capacity(
        NonZeroUsize::new(capacity).expect("ingress capacity must be positive"),
    );
    let application = FocusQueue::with_activity_launcher(60, |_, _, _| Ok(()));
    let mut runner = AppRunner::from_terminal_with_options(application, &session, options)
        .expect("slow-sink runner must initialize");

    assert_eq!(
        runner
            .render_terminal(&mut session)
            .expect("initial frame must render"),
        TerminalRenderOutcome::Applied
    );
    runner.enqueue(Message::ShowActivity);
    runner.enqueue(Message::StartActivity);
    assert_eq!(runner.process_pending().updates, 2);
    assert_eq!(
        runner
            .render_terminal(&mut session)
            .expect("running activity frame must render"),
        TerminalRenderOutcome::Applied
    );

    let proxy = runner.event_proxy();
    let generation = runner.application().activity_generation();
    (runner, session, proxy, generation)
}

fn send(proxy: &EventProxy<Message>, message: Message) {
    if let Err(error) = proxy.send(message) {
        panic!("slow-sink ingress rejected a causal sample: {error}");
    }
}

fn difference(after: OutputMetrics, before: OutputMetrics) -> OutputMetrics {
    OutputMetrics {
        bytes: after.bytes.saturating_sub(before.bytes),
        writer_calls: after.writer_calls.saturating_sub(before.writer_calls),
        flushes: after.flushes.saturating_sub(before.flushes),
        patches: after.patches.saturating_sub(before.patches),
        full_repaints: after.full_repaints.saturating_sub(before.full_repaints),
        last_write_completed: after.last_write_completed,
    }
}

fn average(total: Duration, count: usize) -> Duration {
    total
        .checked_div(u32::try_from(count).expect("sample count must fit u32"))
        .expect("sample count must be nonzero")
}

fn percentile(sorted: &[Duration], percent: usize) -> Duration {
    let rank = sorted.len().saturating_mul(percent).saturating_add(99) / 100;
    sorted[rank.saturating_sub(1)]
}

fn measure(sink_delay: Duration) -> Report {
    let (mut runner, mut session, proxy, generation) = setup(sink_delay, INGRESS_CAPACITY, None);
    let ingress_before = proxy.metrics();
    let output_before = session.backend().metrics();
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut phases = PhaseTotals::default();

    for index in 0..SAMPLE_COUNT {
        let flushes_before = session.backend().metrics().flushes;
        let message = Message::ActivityItem {
            generation,
            text: format!("Slow sink item {index}"),
        };
        let input_started = Instant::now();
        send(&proxy, message);
        let update_started = Instant::now();
        let process = runner.process_pending();
        let update = update_started.elapsed();
        assert_eq!(process.updates, 1);

        let rendered = runner
            .render_terminal_timed(&mut session)
            .expect("slow-sink sample must render");
        assert_eq!(rendered.outcome, TerminalRenderOutcome::Applied);
        let timings = rendered.timings.expect("sample render must be timed");
        let output = session.backend().metrics();
        assert_eq!(output.flushes, flushes_before.saturating_add(1));
        let completed = output
            .last_write_completed
            .expect("successful flush must record write completion");
        samples.push(completed.duration_since(input_started));
        phases.add(update, timings);
    }

    let ingress_after = proxy.metrics();
    let output = difference(session.backend().metrics(), output_before);
    assert_eq!(
        ingress_after
            .accepted
            .saturating_sub(ingress_before.accepted),
        SAMPLE_COUNT as u64
    );
    assert_eq!(
        ingress_after
            .dequeued
            .saturating_sub(ingress_before.dequeued),
        SAMPLE_COUNT as u64
    );
    assert_eq!(output.patches, SAMPLE_COUNT);
    assert_eq!(output.flushes, SAMPLE_COUNT);
    assert_eq!(output.full_repaints, 0);
    assert_eq!(runner.application().activity_item_count(), 32);
    assert_eq!(
        runner.application().activity_item(31),
        Some("Slow sink item 63")
    );

    samples.sort_unstable();
    let total_input_to_write = samples
        .iter()
        .copied()
        .fold(Duration::ZERO, |total, sample| total.saturating_add(sample));
    let total_queue = ingress_after
        .total_queue_latency
        .checked_sub(ingress_before.total_queue_latency)
        .expect("ingress latency must be monotonic");
    Report {
        sink_delay,
        average_input_to_write: average(total_input_to_write, SAMPLE_COUNT),
        p50_input_to_write: percentile(&samples, 50),
        p95_input_to_write: percentile(&samples, 95),
        max_input_to_write: *samples.last().expect("samples must be nonempty"),
        average_queue: average(total_queue, SAMPLE_COUNT),
        phases,
        output,
    }
}

fn assert_output_limits(report: &Report) {
    assert!(
        report.output.bytes <= MAX_ANSI_BYTES,
        "slow-sink ANSI output exceeded its tracked ceiling: {} > {MAX_ANSI_BYTES}",
        report.output.bytes
    );
    assert!(
        report.output.writer_calls <= MAX_WRITER_CALLS,
        "slow-sink serializer callbacks exceeded their tracked ceiling: {} > {MAX_WRITER_CALLS}",
        report.output.writer_calls
    );
}

fn assert_latency_limits(report: &Report) {
    let average_overhead = report
        .average_input_to_write
        .saturating_sub(report.sink_delay);
    let p95_overhead = report.p95_input_to_write.saturating_sub(report.sink_delay);
    let max_overhead = report.max_input_to_write.saturating_sub(report.sink_delay);
    assert!(
        average_overhead <= MAX_AVERAGE_OVERHEAD,
        "slow-sink average overhead exceeded its tracked ceiling: {average_overhead:?} > {MAX_AVERAGE_OVERHEAD:?}"
    );
    assert!(
        p95_overhead <= MAX_P95_OVERHEAD,
        "slow-sink p95 overhead exceeded its tracked ceiling: {p95_overhead:?} > {MAX_P95_OVERHEAD:?}"
    );
    assert!(
        max_overhead <= MAX_SINGLE_OVERHEAD,
        "slow-sink maximum overhead exceeded its tracked ceiling: {max_overhead:?} > {MAX_SINGLE_OVERHEAD:?}"
    );
}

#[test]
fn production_output_stays_within_regression_limits() {
    assert_output_limits(&measure(Duration::ZERO));
}

#[test]
fn bounded_ingress_fills_while_backend_write_is_blocked() {
    const CAPACITY: usize = 2;
    let (gate, entered, release) = BlockingGate::new();
    let (mut runner, mut session, proxy, generation) =
        setup(Duration::ZERO, CAPACITY, Some(Arc::clone(&gate)));

    send(
        &proxy,
        Message::ActivityItem {
            generation,
            text: "Write trigger".to_owned(),
        },
    );
    assert_eq!(runner.process_pending().updates, 1);
    gate.arm();

    let producer_proxy = proxy.clone();
    let producer = thread::Builder::new()
        .name("focus-queue-blocked-write-ingress".to_owned())
        .spawn(move || {
            entered
                .recv_timeout(BLOCK_TIMEOUT)
                .expect("backend write must reach the blocking flush");
            send(
                &producer_proxy,
                Message::ActivityItem {
                    generation,
                    text: "Queued one".to_owned(),
                },
            );
            send(
                &producer_proxy,
                Message::ActivityItem {
                    generation,
                    text: "Queued two".to_owned(),
                },
            );
            let rejected = Message::ActivityItem {
                generation,
                text: "Recovered after full".to_owned(),
            };
            let error = match producer_proxy.send(rejected) {
                Ok(()) => panic!("third blocked-write message must observe Full"),
                Err(error) => error,
            };
            release
                .send(())
                .expect("blocked backend write must still be waiting");
            error
        })
        .expect("blocked-write producer must spawn");

    assert_eq!(
        runner
            .render_terminal(&mut session)
            .expect("blocked write must finish after release"),
        TerminalRenderOutcome::Applied
    );
    let error = producer
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
    assert_eq!(error.kind(), EventProxySendErrorKind::Full);
    let metrics = proxy.metrics();
    assert_eq!(metrics.capacity, CAPACITY);
    assert_eq!(metrics.depth, CAPACITY);
    assert_eq!(metrics.high_water_mark, CAPACITY);
    assert_eq!(metrics.rejected, 1);

    assert_eq!(runner.process_pending().updates, CAPACITY);
    send(&proxy, error.into_inner());
    assert_eq!(runner.process_pending().updates, 1);
    assert_eq!(
        runner
            .render_terminal(&mut session)
            .expect("recovered blocked-write ingress must render"),
        TerminalRenderOutcome::Applied
    );
    assert_eq!(proxy.metrics().depth, 0);
    assert_eq!(runner.application().activity_item_count(), 4);
    assert_eq!(runner.application().activity_item(0), Some("Write trigger"));
    assert_eq!(runner.application().activity_item(1), Some("Queued one"));
    assert_eq!(runner.application().activity_item(2), Some("Queued two"));
    assert_eq!(
        runner.application().activity_item(3),
        Some("Recovered after full")
    );
    let metrics = proxy.metrics();
    assert_eq!(metrics.accepted, 4);
    assert_eq!(metrics.dequeued, 4);
    assert_eq!(metrics.rejected, 1);
}

#[test]
#[ignore = "wall-clock slow-sink metric probe"]
fn slow_sink_metrics() {
    println!(
        "| Sink delay | Samples | Input-to-write avg | p50 | p95 | Max | Queue avg | Update avg | Render avg | View avg | Reconcile avg | Layout avg | Paint avg | Diff avg | Instrumented serialize/write avg | ANSI bytes | Writer calls | Flushes | Full repaints |"
    );
    println!(
        "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    for delay in [
        Duration::ZERO,
        Duration::from_millis(1),
        Duration::from_millis(5),
    ] {
        let report = measure(delay);
        assert_output_limits(&report);
        assert_latency_limits(&report);
        println!(
            "| {:.3} ms | {} | {:.3} ms | {:.3} ms | {:.3} ms | {:.3} ms | {:.3} us | {:.3} us | {:.3} ms | {:.3} us | {:.3} us | {:.3} us | {:.3} us | {:.3} us | {:.3} ms | {} | {} | {} | {} |",
            report.sink_delay.as_secs_f64() * 1_000.0,
            SAMPLE_COUNT,
            report.average_input_to_write.as_secs_f64() * 1_000.0,
            report.p50_input_to_write.as_secs_f64() * 1_000.0,
            report.p95_input_to_write.as_secs_f64() * 1_000.0,
            report.max_input_to_write.as_secs_f64() * 1_000.0,
            report.average_queue.as_secs_f64() * 1_000_000.0,
            average(report.phases.update, SAMPLE_COUNT).as_secs_f64() * 1_000_000.0,
            average(report.phases.render, SAMPLE_COUNT).as_secs_f64() * 1_000.0,
            average(report.phases.view, SAMPLE_COUNT).as_secs_f64() * 1_000_000.0,
            average(report.phases.staging, SAMPLE_COUNT).as_secs_f64() * 1_000_000.0,
            average(report.phases.layout, SAMPLE_COUNT).as_secs_f64() * 1_000_000.0,
            average(report.phases.paint, SAMPLE_COUNT).as_secs_f64() * 1_000_000.0,
            average(report.phases.diff, SAMPLE_COUNT).as_secs_f64() * 1_000_000.0,
            average(report.phases.terminal, SAMPLE_COUNT).as_secs_f64() * 1_000.0,
            report.output.bytes,
            report.output.writer_calls,
            report.output.flushes,
            report.output.full_repaints,
        );
    }
}
