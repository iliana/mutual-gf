// Copyright 2020 iliana destroyer of worlds <iliana@buttslol.net>
// SPDX-License-Identifier: MIT-0

#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use bytes::{Buf, Bytes};
use chrono::{DateTime, Datelike, FixedOffset};
use minreq::{Method, Request};
use serde::{Deserialize, Serialize};

const EMOJI: [char; 8] = ['🌑', '🌒', '🌓', '🌔', '🌕', '🌖', '🌗', '🌘'];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::from_env()?;
    loop {
        let response = env.lambda(Method::Get, "next").send_lazy()?;
        let request_id = &response.headers["lambda-runtime-aws-request-id"];
        let date = DateTime::parse_from_rfc2822(&response.headers["date"])?;
        let date = date.with_timezone(&tz_offset(date.timestamp()));
        println!("{:?}", date);

        if !(2020..=2025).contains(&date.year()) {
            panic!("moon phase calendar only goes from 2020 to 2025");
        }
        let moon = phase_for_date(date.year(), date.month(), date.day()).to_string();
        println!("{}", moon);

        let old_display_name = env
            .masto(Method::Get, "accounts/verify_credentials")
            .send()?
            .json::<Profile>()?
            .display_name;
        println!("old display name: {}", old_display_name);
        let display_name = old_display_name.replace(&EMOJI[..], &moon);
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

        env.lambda(Method::Post, &format!("{}/response", request_id))
            .with_json(&())?
            .send_lazy()?;
    }
}

fn phase_for_date(year: i32, month: u32, day: u32) -> char {
    let phase = EMOJI.binary_search(&CALENDAR_START_PHASE).unwrap()
        + match CALENDAR.binary_search(&(year, month, day)) {
            Ok(n) => 2 * n,
            Err(n) => 2 * n + 7,
        };
    EMOJI[phase % 8]
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

struct Env {
    aws_lambda_runtime_api: String,
    masto_base: String,
    masto_access_token: String,
}

impl Env {
    fn from_env() -> Result<Env, std::env::VarError> {
        Ok(Env {
            aws_lambda_runtime_api: std::env::var("AWS_LAMBDA_RUNTIME_API")?,
            masto_base: std::env::var("MASTO_BASE")?,
            masto_access_token: std::env::var("MASTO_ACCESS_TOKEN")?,
        })
    }

    fn lambda(&self, method: Method, path: &str) -> Request {
        Request::new(
            method,
            format!(
                "http://{}/2018-06-01/runtime/invocation/{}",
                self.aws_lambda_runtime_api, path
            ),
        )
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

macro_rules! calendar {
    ( $( $year:expr => { $( $month:ident => [ $( $day:expr ),* ] ,)* } )* ) => {{
        #[repr(u8)]
        enum Month { Jan = 1, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec }
        [ $( $( $( ($year, Month::$month as u32, $day), )* )* )* ]
    }};
}

/// Moon phase transitions for Seattle, Washington, USA, 2020-2025
const CALENDAR_START_PHASE: char = '🌓';
const CALENDAR: &[(i32, u32, u32)] = &calendar! {
    2020 => {
        Jan => [2, 10, 17, 24],
        Feb => [1, 8, 15, 23],
        Mar => [2, 9, 16, 24],
        Apr => [1, 7, 14, 22, 30],
        May => [7, 14, 22, 29],
        Jun => [5, 12, 20, 28],
        Jul => [4, 12, 20, 27],
        Aug => [3, 11, 18, 25],
        Sep => [1, 10, 17, 23],
        Oct => [1, 9, 16, 23, 31],
        Nov => [8, 14, 21, 30],
        Dec => [7, 14, 21, 29],
    }
    2021 => {
        Jan => [6, 12, 20, 28],
        Feb => [4, 11, 19, 27],
        Mar => [5, 13, 21, 28],
        Apr => [4, 11, 19, 26],
        May => [3, 11, 19, 26],
        Jun => [2, 10, 17, 24],
        Jul => [1, 9, 17, 23, 31],
        Aug => [8, 15, 22, 30],
        Sep => [6, 13, 20, 28],
        Oct => [6, 12, 20, 28],
        Nov => [4, 11, 19, 27],
        Dec => [3, 10, 18, 26],
    }
    2022 => {
        Jan => [2, 9, 17, 25, 31],
        Feb => [8, 16, 23],
        Mar => [2, 10, 18, 24, 31],
        Apr => [8, 16, 23, 30],
        May => [8, 15, 22, 30],
        Jun => [7, 14, 20, 28],
        Jul => [6, 13, 20, 28],
        Aug => [5, 11, 18, 27],
        Sep => [3, 10, 17, 25],
        Oct => [2, 9, 17, 25, 31],
        Nov => [8, 16, 23, 30],
        Dec => [7, 16, 23, 29],
    }
    2023 => {
        Jan => [6, 14, 21, 28],
        Feb => [5, 13, 19, 27],
        Mar => [7, 14, 21, 28],
        Apr => [5, 13, 19, 27],
        May => [5, 12, 19, 27],
        Jun => [3, 10, 17, 26],
        Jul => [3, 9, 17, 25],
        Aug => [1, 8, 16, 24, 30],
        Sep => [6, 14, 22, 29],
        Oct => [6, 14, 21, 28],
        Nov => [5, 13, 20, 27],
        Dec => [4, 12, 19, 26],
    }
    2024 => {
        Jan => [3, 11, 17, 25],
        Feb => [2, 9, 16, 24],
        Mar => [3, 10, 16, 25],
        Apr => [1, 8, 15, 23],
        May => [1, 7, 15, 23, 30],
        Jun => [6, 13, 21, 28],
        Jul => [5, 13, 21, 27],
        Aug => [4, 12, 19, 26],
        Sep => [2, 10, 17, 24],
        Oct => [2, 10, 17, 24],
        Nov => [1, 8, 15, 22, 30],
        Dec => [8, 15, 22, 30],
    }
    2025 => {
        Jan => [6, 13, 21, 29],
        Feb => [5, 12, 20, 27],
        Mar => [6, 13, 22, 29],
        Apr => [4, 12, 20, 27],
        May => [4, 12, 20, 26],
        Jun => [2, 11, 18, 25],
        Jul => [2, 10, 17, 24],
        Aug => [1, 9, 15, 22, 30],
        Sep => [7, 14, 21, 29],
        Oct => [6, 13, 21, 29],
        Nov => [5, 11, 19, 27],
        Dec => [4, 11, 19, 27],
    }
};

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

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[cfg(test)]
#[test]
fn test() {
    fn is_sorted<T: Clone + Ord>(s: &[T]) -> bool {
        let mut sorted = s.to_vec();
        sorted.sort();
        s == sorted.as_slice()
    }

    assert!(is_sorted(&EMOJI));
    assert!(is_sorted(&CALENDAR));

    assert_eq!(phase_for_date(2020, 01, 01), '🌒');
    assert_eq!(phase_for_date(2020, 01, 02), '🌓');
    assert_eq!(phase_for_date(2020, 01, 03), '🌔');
    assert_eq!(phase_for_date(2020, 01, 09), '🌔');
    assert_eq!(phase_for_date(2020, 01, 10), '🌕');
    assert_eq!(phase_for_date(2020, 01, 11), '🌖');
    assert_eq!(phase_for_date(2020, 01, 16), '🌖');
    assert_eq!(phase_for_date(2020, 01, 17), '🌗');
    assert_eq!(phase_for_date(2020, 01, 18), '🌘');
    assert_eq!(phase_for_date(2020, 01, 23), '🌘');
    assert_eq!(phase_for_date(2020, 01, 24), '🌑');
    assert_eq!(phase_for_date(2020, 01, 25), '🌒');

    assert_eq!(phase_for_date(2020, 10, 31), '🌕');
    assert_eq!(phase_for_date(2021, 08, 08), '🌑');
    assert_eq!(phase_for_date(2025, 12, 31), '🌔');

    assert_eq!(tz_offset(1583661599).local_minus_utc(), -28800);
    assert_eq!(tz_offset(1583661600).local_minus_utc(), -25200);
    assert_eq!(tz_offset(1583661601).local_minus_utc(), -25200);
}
