// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![doc(html_root_url = "https://docs.rs/proxy-wasm-experimental/0.0.8")]

pub mod error;
pub mod hostcalls;
pub mod traits;
pub mod types;

mod allocator;
mod bytestring;
mod dispatcher;
mod logger;

pub fn set_log_level(level: types::LogLevel) {
    logger::set_log_level(level);
}

pub fn set_root_context<F>(callback: F)
where
    F: FnMut(u32) -> Box<dyn traits::RootContext> + 'static,
{
    dispatcher::set_root_context(Box::new(callback));
}

pub fn set_stream_context<F>(callback: F)
where
    F: FnMut(u32, u32) -> Box<dyn traits::StreamContext> + 'static,
{
    dispatcher::set_stream_context(Box::new(callback));
}

pub fn set_http_context<F>(callback: F)
where
    F: FnMut(u32, u32) -> Box<dyn traits::HttpContext> + 'static,
{
    dispatcher::set_http_context(Box::new(callback));
}

#[no_mangle]
pub extern "C" fn proxy_abi_version_0_2_0() {}
