 
#[cfg(not(target_family="wasm"))]
mod native;

#[cfg(not(target_family="wasm"))]
pub use native::*;


#[cfg(target_family="wasm")]
#[cfg(web_sys_unstable_apis)]
mod web;

#[cfg(target_family="wasm")]
#[cfg(web_sys_unstable_apis)]
pub use web::*;