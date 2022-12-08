#![allow(non_snake_case)]
use anyhow::Result;

// use crate::osx::register_fn;
// mod osx;

mod patch;

mod macho;
mod cache;

fn main() -> Result<()> {
    let c = cache::Cache::new()?;
    let res = c.create(None)?;
    println!("+ res: {:#x?}", res);
    Ok(())
}
