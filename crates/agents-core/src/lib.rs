pub mod driftx;
pub mod cleanup;
pub mod explain;
pub mod fsutil;
pub mod loadag;
pub mod matwiz;
pub mod model;
pub mod outputs;
pub mod prompts;
pub mod resolv;
pub mod schemas;
pub mod shared;
pub mod skillpl;
pub mod stamps;
pub mod templ;

pub fn hello_core() -> &'static str {
    "agents-core"
}
