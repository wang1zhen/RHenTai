use eh_client::client::RateLimiter;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn test_rate_limiter_spacing_after_delayed_wake() {
    let limiter = Arc::new(RateLimiter::new(350));

    // 1. Acquire #1
    let p1 = limiter.acquire().await.unwrap();
    let _start1 = tokio::time::Instant::now();
    drop(p1);

    // 2. Spawn acquire #2 (it sleeps until +350 ms), then yield
    let lim2 = limiter.clone();
    let handle2 = tokio::spawn(async move {
        let p2 = lim2.acquire().await.unwrap();
        let start2 = tokio::time::Instant::now();
        drop(p2);
        start2
    });
    tokio::task::yield_now().await;

    // 3. Advance time to simulate a late wake (600 ms > 350 ms)
    tokio::time::advance(Duration::from_millis(600)).await;

    // 4. Await #2 and record start2
    let start2 = handle2.await.unwrap();

    // 5. Acquire #3 and record start3
    let p3 = limiter.acquire().await.unwrap();
    let start3 = tokio::time::Instant::now();
    drop(p3);

    // 6. Assert start3 - start2 >= 350 ms
    let gap = start3 - start2;
    assert!(
        gap >= Duration::from_millis(350),
        "Request starts must never be closer than interval: actual gap was {gap:?}"
    );
}
