// Copyright 2021 iliana destroyer of worlds <iliana@buttslol.net>
// SPDX-License-Identifier: MIT-0

#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use bytes::{Buf, Bytes};
use chrono::{DateTime, FixedOffset, Utc};
use minreq::{Method, Request};
use serde::{Deserialize, Serialize};

fn main() -> ! {
    minlambda::run(handler)
}

fn handler(event: Event) -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::from_env()?;

    let date = event.time.with_timezone(&tz_offset(event.time.timestamp()));
    println!("{:?}", date);

    let moon = esbat::daily_lunar_phase(date.date()).as_emoji().to_string();
    println!("{}", moon);

    let moon_emoji = esbat::Phase::iter()
        .map(|p| p.as_emoji())
        .collect::<Vec<_>>();

    let old_display_name = env
        .masto(Method::Get, "accounts/verify_credentials")
        .send()?
        .json::<Profile>()?
        .display_name;
    println!("old display name: {}", old_display_name);
    let display_name = old_display_name.replace(moon_emoji.as_slice(), &moon);
    println!("new display name: {}", display_name);
    if display_name != old_display_name {
        let body = serde_urlencoded::to_string(Profile { display_name })?.into_bytes();
        println!(
            "accounts/update_credentials: {}",
            env.masto(Method::Patch, "accounts/update_credentials")
                .with_body(body)
                .with_header("content-type", "application/x-www-form-urlencoded")
                .send_lazy()?
                .status_code
        );
    }

    Ok(())
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Deserialize, Clone, Copy)]
struct Event {
    time: DateTime<Utc>,
}

struct Env {
    masto_base: String,
    masto_access_token: String,
}

impl Env {
    fn from_env() -> Result<Env, std::env::VarError> {
        Ok(Env {
            masto_base: std::env::var("MASTO_BASE")?,
            masto_access_token: std::env::var("MASTO_ACCESS_TOKEN")?,
        })
    }

    fn masto(&self, method: Method, path: &str) -> Request {
        Request::new(
            method,
            format!("https://{}/api/v1/{}", self.masto_base, path),
        )
        .with_header(
            "authorization",
            format!("Bearer {}", self.masto_access_token),
        )
    }
}

#[derive(Deserialize, Serialize)]
struct Profile {
    display_name: String,
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

fn tz_offset(timestamp: i64) -> FixedOffset {
    let mut tzif = Bytes::from_static(include_bytes!("/usr/share/zoneinfo/America/Los_Angeles"));

    // skip past 32-bit header
    {
        assert_eq!(tzif.get_u32(), u32::from_be_bytes(*b"TZif"));
        assert_eq!(tzif.get_u8(), b'2');
        tzif.advance(15);
        let isutccnt = tzif.get_u32() as usize;
        let isstdcnt = tzif.get_u32() as usize;
        let leapcnt = tzif.get_u32() as usize;
        let timecnt = tzif.get_u32() as usize;
        let typecnt = tzif.get_u32() as usize;
        let charcnt = tzif.get_u32() as usize;
        tzif.advance(
            timecnt * 4 + timecnt + typecnt * 6 + charcnt + leapcnt * 8 + isstdcnt + isutccnt,
        );
    }

    assert_eq!(tzif.get_u32(), u32::from_be_bytes(*b"TZif"));
    assert_eq!(tzif.get_u8(), b'2');
    tzif.advance(15);
    tzif.advance(12); // isutccnt, isstdcnt, leapcnt
    let timecnt = tzif.get_u32() as usize;
    tzif.advance(8); // typecnt, charcnt

    let n = match (0..timecnt)
        .map(|_| tzif.get_i64())
        .collect::<Vec<_>>()
        .binary_search(&timestamp)
    {
        Ok(n) => n,
        Err(n) => n - 1,
    };

    tzif.advance(n);
    let transition_time_index = tzif.get_u8() as usize;
    tzif.advance(timecnt - n - 1);

    tzif.advance(6 * transition_time_index);
    FixedOffset::east(tzif.get_i32())
}

#[cfg(test)]
#[test]
fn test() {
    assert_eq!(tz_offset(1583661599).local_minus_utc(), -28800);
    assert_eq!(tz_offset(1583661600).local_minus_utc(), -25200);
    assert_eq!(tz_offset(1583661601).local_minus_utc(), -25200);
}
