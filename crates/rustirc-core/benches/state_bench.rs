//! Benchmarks for state management operations
//!
//! Tests the StateManager with its event-sourcing based async API.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rustirc_core::{ChannelState, ChannelUser, ServerState, StateManager, User};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn benchmark_state_creation(c: &mut Criterion) {
    c.bench_function("create state manager", |b| {
        b.iter(|| black_box(StateManager::new()))
    });
}

fn benchmark_server_state_creation(c: &mut Criterion) {
    c.bench_function("create server state", |b| {
        b.iter(|| {
            black_box(ServerState::new(
                "conn-1".to_string(),
                "irc.example.com".to_string(),
                6697,
                true,
            ))
        })
    });
}

fn benchmark_channel_state_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_state");

    // Create channel state
    group.bench_function("create", |b| {
        b.iter(|| black_box(ChannelState::new("#rust".to_string())))
    });

    // Add user to channel
    group.bench_function("add_user", |b| {
        b.iter_with_setup(
            || ChannelState::new("#rust".to_string()),
            |mut channel| {
                let user = User {
                    nickname: "testuser".to_string(),
                    username: Some("user".to_string()),
                    hostname: Some("host.example.com".to_string()),
                    realname: Some("Test User".to_string()),
                    server: None,
                    away: false,
                    away_message: None,
                    idle_time: None,
                    signon_time: None,
                    oper: false,
                    account: None,
                };
                channel.add_user(black_box("testuser".to_string()), black_box(user));
            },
        )
    });

    // Remove user from channel
    group.bench_function("remove_user", |b| {
        b.iter_with_setup(
            || {
                let mut channel = ChannelState::new("#rust".to_string());
                let user = User {
                    nickname: "testuser".to_string(),
                    username: Some("user".to_string()),
                    hostname: Some("host.example.com".to_string()),
                    realname: Some("Test User".to_string()),
                    server: None,
                    away: false,
                    away_message: None,
                    idle_time: None,
                    signon_time: None,
                    oper: false,
                    account: None,
                };
                channel.add_user("testuser".to_string(), user);
                channel
            },
            |mut channel| {
                channel.remove_user(black_box("testuser"));
            },
        )
    });

    group.finish();
}

fn benchmark_channel_user_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_user_scaling");

    for user_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_users", user_count),
            user_count,
            |b, &user_count| {
                b.iter_with_setup(
                    || ChannelState::new("#largechannel".to_string()),
                    |mut channel| {
                        for i in 0..user_count {
                            let user = User {
                                nickname: format!("user{}", i),
                                username: Some(format!("username{}", i)),
                                hostname: Some(format!("host{}.example.com", i)),
                                realname: Some(format!("User {}", i)),
                                server: None,
                                away: i % 10 == 0, // 10% away
                                away_message: None,
                                idle_time: None,
                                signon_time: None,
                                oper: i % 100 == 0, // 1% operators
                                account: None,
                            };
                            channel.add_user(black_box(format!("user{}", i)), black_box(user));
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

fn benchmark_user_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("user_count");

    for user_count in [10, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("count", user_count),
            user_count,
            |b, &user_count| {
                let mut channel = ChannelState::new("#counting".to_string());
                for i in 0..user_count {
                    let user = User {
                        nickname: format!("user{}", i),
                        username: Some(format!("username{}", i)),
                        hostname: Some(format!("host{}.example.com", i)),
                        realname: Some(format!("User {}", i)),
                        server: None,
                        away: false,
                        away_message: None,
                        idle_time: None,
                        signon_time: None,
                        oper: false,
                        account: None,
                    };
                    channel.add_user(format!("user{}", i), user);
                }

                b.iter(|| black_box(channel.user_count()))
            },
        );
    }

    group.finish();
}

fn benchmark_async_state_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("async get_state", |b| {
        let state_manager = StateManager::new();

        b.iter(|| {
            rt.block_on(async {
                black_box(state_manager.get_state().await);
            });
        });
    });
}

fn benchmark_concurrent_state_access(c: &mut Criterion) {
    use std::sync::Arc;

    c.bench_function("concurrent state access", |b| {
        let rt = Runtime::new().unwrap();

        b.iter(|| {
            rt.block_on(async {
                let state_manager = Arc::new(StateManager::new());

                let mut handles = Vec::new();

                // Spawn multiple tasks doing concurrent read operations
                for _ in 0..10 {
                    let sm = Arc::clone(&state_manager);
                    let handle = tokio::spawn(async move {
                        for _ in 0..5 {
                            let _ = black_box(sm.get_state().await);
                        }
                    });
                    handles.push(handle);
                }

                // Wait for all tasks to complete
                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });
    });
}

fn benchmark_channel_user_struct(c: &mut Criterion) {
    c.bench_function("create channel_user", |b| {
        b.iter(|| {
            black_box(ChannelUser {
                nick: "testuser".to_string(),
                modes: vec!['o', 'v'],
                join_time: 1234567890,
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_state_creation,
    benchmark_server_state_creation,
    benchmark_channel_state_operations,
    benchmark_channel_user_scaling,
    benchmark_user_count,
    benchmark_async_state_access,
    benchmark_concurrent_state_access,
    benchmark_channel_user_struct,
);
criterion_main!(benches);
