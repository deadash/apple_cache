#![allow(non_snake_case)]
#![feature(exclusive_range_pattern)]
#![feature(slice_internals)]
use std::fs;
use anyhow::{Result, Context};

// use crate::osx::register_fn;
// mod osx;

mod patch;

mod macho;
mod cache;

#[cfg(feature = "emu")]
mod emu;

fn main() -> Result<()> {
    let mut template: serde_json::Value = serde_json::from_slice(&fs::read("cache.json")?)?;
    let mut c = cache::Cache::new()?;
    let (ctx, res) = c.create(None)?;
    let data = base64::encode(res);
    println!("+ res: {:x}, {}", ctx, data);
    // let mut headers = reqwest::header::HeaderMap::new();
    // headers.insert(reqwest::header::USER_AGENT, "AssetCache/243 CFNetwork/1111 Darwin/19.0.0 (x86_64)".parse()?);
    // headers.insert("X-Protocol-Version", "3".parse()?);
    // let client = reqwest::blocking::Client::builder()
    //     .default_headers(headers)
    //     .cookie_store(true)
    //     .build()?;
    // let resp = client.post("https://lcdn-registration.apple.com/lcdn/session")
    //     .body(data)
    //     .send()?;
    // // Got cookies
    // let lcdn = resp.cookies().find_map(|c|
    //     if c.name() == "LCDN-Session" {
    //         Some(c.value().to_owned())
    //     } else { None } ).context("cookies Not found.")?;
    // println!("+ LCDN: {}", lcdn);

    // let data = resp.text()?;
    // println!("+ Got {}", data);
    
    // let data = data.trim_matches('"');
    // let data = base64::decode(data)?;
    // println!("+ Obtain: {:?}", c.obtain(ctx, &data));

    // // json data
    // if let serde_json::Value::Object(ref mut map) = template {
    //     map.insert("session-token".to_owned(), lcdn.into());
    // }
    // let data = template.to_string();

    // // Register
    // let data = c.sign(ctx, data.as_bytes())?;
    // let data = base64::encode(data);
    // println!("+ Sign: {}", data);

    // let resp = client.post("https://lcdn-registration.apple.com/lcdn/register")
    //     .body(data)
    //     .send()?;
    // let text = resp.text()?;
    // println!("+ Register: {}", text);

    // // 获取心跳延迟
    

    Ok(())
}
