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

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Apparently, Rust toolchain doesn't handle well exported name `malloc`
// when this package is compiled to targets other than `wasm32-unknown-unknown`.
// Specifically, linking issues have been observed with targets `wasm32-wasi`
// and `x86_64-unknown-linux-gnu`, which is a blocker for unit testing.
// Therefore, export name `malloc` only in the context of target `wasm32-unknown-unknown`.
#[cfg_attr(all(target_arch = "wasm32", target_vendor = "unknown", target_os = "unknown"), export_name = "malloc")]
#[no_mangle]
pub extern "C" fn proxy_on_memory_allocate(size: usize) -> *mut u8 {
    let mut vec: Vec<u8> = Vec::with_capacity(size);
    unsafe {
        vec.set_len(size);
    }
    let slice = vec.into_boxed_slice();
    Box::into_raw(slice) as *mut u8
}
