#![allow(missing_docs)]
//! Opt-in live-thread ingress pressure and latency report.

use std::{
    num::NonZeroUsize,
    sync::{Arc, Barrier, Mutex, MutexGuard},
    thread,
    time::{Duration, Instant},
};

use arborui::{
    AppRunner, EventProxy, EventProxySendErrorKind, HeadlessRenderOutcome, Invalidation, Renderer,
    RuntimeOptions, Size, WidthPolicy,
};
use arborui_example_focus_queue::{ActivityCancellation, ActivityStatus, FocusQueue, Message};

const MESSAGE_COUNT: usize = 1_024;
const MAX_TURNS: usize = 100_000;
const RETRY_DELAY: Duration = Duration::from_micros(50);

struct Launch {
    generation: u64,
    cancellation: ActivityCancellation,
    proxy: EventProxy<Message>,
}

struct Report {
    capacity: usize,
    accepted: u64,
    rejected: u64,
    average_queue_latency: Duration,
    max_queue_latency: Duration,
    elapsed: Duration,
    turns: usize,
    frames: usize,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn send_with_retry(
    proxy: &EventProxy<Message>,
    mut message: Message,
    saturated: &Barrier,
    saturation_reported: &mut bool,
) -> u64 {
    let mut retries = 0u64;
    loop {
        match proxy.send(message) {
            Ok(()) => return retries,
            Err(error) if error.kind() == EventProxySendErrorKind::Full => {
                retries = retries.saturating_add(1);
                message = error.into_inner();
                if !*saturation_reported {
                    *saturation_reported = true;
                    saturated.wait();
                }
                thread::sleep(RETRY_DELAY);
            }
            Err(error) => panic!("live ingress closed before delivery: {error}"),
        }
    }
}

fn measure(capacity: usize) -> Report {
    let launch = Arc::new(Mutex::new(None));
    let launch_slot = Arc::clone(&launch);
    let application =
        FocusQueue::with_activity_launcher(60, move |generation, cancellation, proxy| {
            *lock(&launch_slot) = Some(Launch {
                generation,
                cancellation,
                proxy,
            });
            Ok(())
        });
    let size = Size::new(72, 18);
    let options = RuntimeOptions::new()
        .with_event_ingress_capacity(NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::MIN));
    let mut runner = AppRunner::new_with_options(
        application,
        size,
        Renderer::new(size, WidthPolicy::Unicode),
        options,
    );
    assert_eq!(
        runner.render_headless().expect("initial frame must render"),
        HeadlessRenderOutcome::Committed
    );
    runner.enqueue(Message::StartActivity);
    assert_eq!(runner.process_pending().updates, 1);
    assert_eq!(
        runner
            .render_headless()
            .expect("running activity frame must render"),
        HeadlessRenderOutcome::Committed
    );

    let Launch {
        generation,
        cancellation,
        proxy,
    } = lock(&launch)
        .take()
        .expect("activity launcher must expose its ingress proxy");
    let producer_start = Arc::new(Barrier::new(2));
    let thread_start = Arc::clone(&producer_start);
    let saturated = Arc::new(Barrier::new(2));
    let producer_saturated = Arc::clone(&saturated);
    let producer_proxy = proxy.clone();
    let producer = thread::Builder::new()
        .name(format!("focus-queue-ingress-{capacity}"))
        .spawn(move || {
            thread_start.wait();
            let mut saturation_reported = false;
            let mut retries = 0u64;
            for index in 0..MESSAGE_COUNT {
                retries = retries.saturating_add(send_with_retry(
                    &producer_proxy,
                    Message::ActivityItem {
                        generation,
                        text: format!("Live ingress item {index}"),
                    },
                    &producer_saturated,
                    &mut saturation_reported,
                ));
            }
            retries.saturating_add(send_with_retry(
                &producer_proxy,
                Message::ActivityFinished { generation },
                &producer_saturated,
                &mut saturation_reported,
            ))
        })
        .expect("live ingress producer must spawn");

    let started = Instant::now();
    producer_start.wait();
    saturated.wait();
    let mut turns = 0usize;
    let mut frames = 0usize;
    while runner.application().activity_status() != ActivityStatus::Completed
        || proxy.metrics().depth != 0
    {
        assert!(turns < MAX_TURNS, "live ingress did not settle");
        let process = runner.wait_for_work(Duration::from_millis(10));
        turns = turns.saturating_add(1);
        if process.invalidation != Invalidation::None {
            let outcome = runner
                .render_headless()
                .expect("live ingress frame must render");
            if outcome == HeadlessRenderOutcome::Committed {
                frames = frames.saturating_add(1);
            }
        }
    }
    let elapsed = started.elapsed();
    let retries = producer
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
    let metrics = proxy.metrics();

    assert!(cancellation.is_cancelled());
    assert_eq!(runner.application().activity_item_count(), 32);
    assert_eq!(metrics.capacity, capacity);
    assert_eq!(metrics.depth, 0);
    assert_eq!(metrics.high_water_mark, capacity);
    assert_eq!(metrics.accepted, (MESSAGE_COUNT + 1) as u64);
    assert_eq!(metrics.dequeued, metrics.accepted);
    assert_eq!(metrics.rejected, retries);
    assert!(!metrics.closed);

    let average_queue_latency = metrics
        .total_queue_latency
        .checked_div(u32::try_from(metrics.dequeued).expect("sample count must fit u32"))
        .expect("live ingress must dequeue messages");
    Report {
        capacity,
        accepted: metrics.accepted,
        rejected: metrics.rejected,
        average_queue_latency,
        max_queue_latency: metrics.max_queue_latency,
        elapsed,
        turns,
        frames,
    }
}

#[test]
#[ignore = "live scheduling metric probe"]
fn live_ingress_metrics() {
    println!(
        "| Capacity | Accepted | Full rejections | Average queue latency | Max queue latency | Complete burst | Drain turns | Frames |"
    );
    println!("| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    for capacity in [1, 8, 64] {
        let report = measure(capacity);
        println!(
            "| {} | {} | {} | {:.3} us | {:.3} us | {:.3} ms | {} | {} |",
            report.capacity,
            report.accepted,
            report.rejected,
            report.average_queue_latency.as_secs_f64() * 1_000_000.0,
            report.max_queue_latency.as_secs_f64() * 1_000_000.0,
            report.elapsed.as_secs_f64() * 1_000.0,
            report.turns,
            report.frames,
        );
    }
}
