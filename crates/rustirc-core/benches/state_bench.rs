use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rustirc_core::{ChannelState, ChannelUser, StateManager, User};
use std::collections::HashMap;

fn benchmark_state_creation(c: &mut Criterion) {
    c.bench_function("create state manager", |b| {
        b.iter(|| StateManager::new(black_box("test_server".to_string())))
    });
}

fn benchmark_user_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("user_management");

    for user_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_users", user_count),
            user_count,
            |b, &user_count| {
                b.iter_with_setup(
                    || StateManager::new("test_server".to_string()),
                    |mut state| {
                        for i in 0..user_count {
                            let user = User {
                                nickname: format!("user{}", i),
                                username: Some(format!("username{}", i)),
                                hostname: Some(format!("host{}.example.com", i)),
                                realname: Some(format!("Real User {}", i)),
                                is_away: false,
                                away_message: None,
                                server: Some("server.example.com".to_string()),
                                hopcount: Some(0),
                                account: None,
                                operator_level: None,
                            };
                            state.add_user(black_box(user));
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

fn benchmark_channel_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_management");

    for channel_count in [10, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_channels", channel_count),
            channel_count,
            |b, &channel_count| {
                b.iter_with_setup(
                    || StateManager::new("test_server".to_string()),
                    |mut state| {
                        for i in 0..channel_count {
                            let channel = ChannelState {
                                name: format!("#channel{}", i),
                                topic: Some(format!("Topic for channel {}", i)),
                                topic_set_by: Some("admin".to_string()),
                                topic_set_at: None,
                                modes: HashMap::new(),
                                users: HashMap::new(),
                                user_count: 0,
                                ban_list: Vec::new(),
                                except_list: Vec::new(),
                                invite_list: Vec::new(),
                                created_at: None,
                            };
                            state.add_channel(black_box(channel));
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

fn benchmark_user_channel_operations(c: &mut Criterion) {
    c.bench_function("user join/part operations", |b| {
        b.iter_with_setup(
            || {
                let mut state = StateManager::new("test_server".to_string());

                // Pre-populate with users
                for i in 0..100 {
                    let user = User {
                        nickname: format!("user{}", i),
                        username: Some(format!("username{}", i)),
                        hostname: Some(format!("host{}.example.com", i)),
                        realname: Some(format!("Real User {}", i)),
                        is_away: false,
                        away_message: None,
                        server: Some("server.example.com".to_string()),
                        hopcount: Some(0),
                        account: None,
                        operator_level: None,
                    };
                    state.add_user(user);
                }

                // Pre-populate with channels
                for i in 0..10 {
                    let channel = ChannelState {
                        name: format!("#channel{}", i),
                        topic: Some(format!("Topic for channel {}", i)),
                        topic_set_by: Some("admin".to_string()),
                        topic_set_at: None,
                        modes: HashMap::new(),
                        users: HashMap::new(),
                        user_count: 0,
                        ban_list: Vec::new(),
                        except_list: Vec::new(),
                        invite_list: Vec::new(),
                        created_at: None,
                    };
                    state.add_channel(channel);
                }

                state
            },
            |mut state| {
                // Simulate users joining and parting channels
                for i in 0..50 {
                    let user_nick = format!("user{}", i % 100);
                    let channel_name = format!("#channel{}", i % 10);

                    let channel_user = ChannelUser {
                        nickname: user_nick.clone(),
                        prefix: "".to_string(),
                        modes: Vec::new(),
                        joined_at: None,
                    };

                    // Join
                    state.add_user_to_channel(
                        black_box(&channel_name),
                        black_box(channel_user.clone()),
                    );

                    // Part (every other user)
                    if i % 2 == 0 {
                        state.remove_user_from_channel(
                            black_box(&channel_name),
                            black_box(&user_nick),
                        );
                    }
                }
            },
        );
    });
}

fn benchmark_state_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_queries");

    // Setup a large state for querying
    let mut state = StateManager::new("test_server".to_string());

    // Add many users
    for i in 0..1000 {
        let user = User {
            nickname: format!("user{}", i),
            username: Some(format!("username{}", i)),
            hostname: Some(format!("host{}.example.com", i)),
            realname: Some(format!("Real User {}", i)),
            is_away: i % 10 == 0, // 10% away
            away_message: None,
            server: Some("server.example.com".to_string()),
            hopcount: Some(0),
            account: None,
            operator_level: None,
        };
        state.add_user(user);
    }

    // Add many channels with users
    for i in 0..100 {
        let channel = ChannelState {
            name: format!("#channel{}", i),
            topic: Some(format!("Topic for channel {}", i)),
            topic_set_by: Some("admin".to_string()),
            topic_set_at: None,
            modes: HashMap::new(),
            users: HashMap::new(),
            user_count: 0,
            ban_list: Vec::new(),
            except_list: Vec::new(),
            invite_list: Vec::new(),
            created_at: None,
        };
        state.add_channel(channel);

        // Add users to channels
        for j in 0..50 {
            let user_index = (i * 10 + j) % 1000;
            let channel_user = ChannelUser {
                nickname: format!("user{}", user_index),
                prefix: "".to_string(),
                modes: Vec::new(),
                joined_at: None,
            };
            state.add_user_to_channel(&format!("#channel{}", i), channel_user);
        }
    }

    group.bench_function("find_user", |b| {
        b.iter(|| state.get_user(black_box("user500")))
    });

    group.bench_function("find_channel", |b| {
        b.iter(|| state.get_channel(black_box("#channel50")))
    });

    group.bench_function("get_user_channels", |b| {
        b.iter(|| state.get_user_channels(black_box("user100")))
    });

    group.bench_function("count_away_users", |b| {
        b.iter(|| state.get_all_users().iter().filter(|u| u.is_away).count())
    });

    group.finish();
}

fn benchmark_concurrent_access(c: &mut Criterion) {
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    c.bench_function("concurrent state access", |b| {
        let rt = Runtime::new().unwrap();

        b.iter(|| {
            rt.block_on(async {
                let state = Arc::new(tokio::sync::RwLock::new(StateManager::new(
                    "test_server".to_string(),
                )));

                let mut handles = Vec::new();

                // Spawn multiple tasks doing concurrent operations
                for i in 0..10 {
                    let state_clone = Arc::clone(&state);
                    let handle = tokio::spawn(async move {
                        for j in 0..10 {
                            let user = User {
                                nickname: format!("user{}_{}", i, j),
                                username: Some(format!("username{}_{}", i, j)),
                                hostname: Some(format!("host{}.example.com", i)),
                                realname: Some(format!("Real User {} {}", i, j)),
                                is_away: false,
                                away_message: None,
                                server: Some("server.example.com".to_string()),
                                hopcount: Some(0),
                                account: None,
                                operator_level: None,
                            };

                            {
                                let mut state_guard = state_clone.write().await;
                                state_guard.add_user(user);
                            }

                            // Some read operations
                            {
                                let state_guard = state_clone.read().await;
                                let _ = state_guard.get_user(&format!("user{}_{}", i, j));
                            }
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

criterion_group!(
    benches,
    benchmark_state_creation,
    benchmark_user_management,
    benchmark_channel_management,
    benchmark_user_channel_operations,
    benchmark_state_queries,
    benchmark_concurrent_access
);
criterion_main!(benches);
