use criterion::{criterion_group, criterion_main, Criterion};
use rustirc_protocol::Parser;
use std::hint::black_box;

fn benchmark_simple_message(c: &mut Criterion) {
    c.bench_function("parse simple message", |b| {
        b.iter(|| Parser::parse_message(black_box("PING :server.example.com")))
    });
}

fn benchmark_complex_message(c: &mut Criterion) {
    c.bench_function("parse complex message", |b| {
        b.iter(|| {
            Parser::parse_message(black_box(
                ":nick!user@host.example.com PRIVMSG #channel :Hello, world!",
            ))
        })
    });
}

fn benchmark_ircv3_message(c: &mut Criterion) {
    c.bench_function("parse IRCv3 message with tags", |b| {
        b.iter(|| {
            Parser::parse_message(black_box("@time=2021-01-01T00:00:00.000Z;msgid=12345 :nick!user@host.example.com PRIVMSG #channel :Hello with tags!"))
        })
    });
}

fn benchmark_long_message(c: &mut Criterion) {
    let long_message = format!(
        ":nick!user@host.example.com PRIVMSG #channel :{}",
        "A".repeat(400)
    );

    c.bench_function("parse long message", |b| {
        b.iter(|| Parser::parse_message(black_box(&long_message)))
    });
}

fn benchmark_malformed_message(c: &mut Criterion) {
    c.bench_function("parse malformed message", |b| {
        b.iter(|| {
            let _ = Parser::parse_message(black_box("INVALID MESSAGE FORMAT"));
        })
    });
}

fn benchmark_batch_parsing(c: &mut Criterion) {
    let messages = vec![
        "PING :server1.example.com",
        ":nick1!user1@host1.com PRIVMSG #test :Message 1",
        ":nick2!user2@host2.com PRIVMSG #test :Message 2",
        "@time=2021-01-01T00:00:00.000Z :nick3!user3@host3.com PRIVMSG #test :Message 3",
        "JOIN #newchannel",
        "PART #oldchannel :Leaving",
        ":server.example.com 001 nick :Welcome to the network",
        ":server.example.com 353 nick = #channel :nick1 nick2 nick3",
        "QUIT :Client disconnecting",
        "NOTICE #channel :Server maintenance in 10 minutes",
    ];

    c.bench_function("parse message batch", |b| {
        b.iter(|| {
            for msg in &messages {
                let _ = Parser::parse_message(black_box(msg));
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_simple_message,
    benchmark_complex_message,
    benchmark_ircv3_message,
    benchmark_long_message,
    benchmark_malformed_message,
    benchmark_batch_parsing
);
criterion_main!(benches);
