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

use crate::dispatcher;
use crate::types::*;
use std::ptr::{null, null_mut};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::{HostCallError, HostResponseError, Result};

/// Represents empty headers map.
pub const NO_HEADERS: &[(&[u8], &[u8])] = &[];

/// Represents empty body.
pub const NO_BODY: Option<&[u8]> = None;

/// Represents empty trailers map.
pub const NO_TRAILERS: &[(&[u8], &[u8])] = &[];

mod abi {
    pub const PROXY_LOG: &str = "proxy_log";
    pub const PROXY_GET_CURRENT_TIME_NANOSECONDS: &str = "proxy_get_current_time_nanoseconds";
    pub const PROXY_SET_TICK_PERIOD_MILLISECONDS: &str = "proxy_set_tick_period_milliseconds";
    pub const PROXY_GET_BUFFER_BYTES: &str = "proxy_get_buffer_bytes";
    pub const PROXY_SET_BUFFER_BYTES: &str = "proxy_set_buffer_bytes";
    pub const PROXY_GET_HEADER_MAP_PAIRS: &str = "proxy_get_header_map_pairs";
    pub const PROXY_SET_HEADER_MAP_PAIRS: &str = "proxy_set_header_map_pairs";
    pub const PROXY_GET_HEADER_MAP_VALUE: &str = "proxy_get_header_map_value";
    pub const PROXY_REPLACE_HEADER_MAP_VALUE: &str = "proxy_replace_header_map_value";
    pub const PROXY_REMOVE_HEADER_MAP_VALUE: &str = "proxy_remove_header_map_value";
    pub const PROXY_ADD_HEADER_MAP_VALUE: &str = "proxy_add_header_map_value";
    pub const PROXY_GET_PROPERTY: &str = "proxy_get_property";
    pub const PROXY_SET_PROPERTY: &str = "proxy_set_property";
    pub const PROXY_GET_SHARED_DATA: &str = "proxy_get_shared_data";
    pub const PROXY_SET_SHARED_DATA: &str = "proxy_set_shared_data";
    pub const PROXY_REGISTER_SHARED_QUEUE: &str = "proxy_register_shared_queue";
    pub const PROXY_RESOLVE_SHARED_QUEUE: &str = "proxy_resolve_shared_queue";
    pub const PROXY_DEQUEUE_SHARED_QUEUE: &str = "proxy_dequeue_shared_queue";
    pub const PROXY_ENQUEUE_SHARED_QUEUE: &str = "proxy_enqueue_shared_queue";
    pub const PROXY_CONTINUE_STREAM: &str = "proxy_continue_stream";
    pub const PROXY_CLOSE_STREAM: &str = "proxy_close_stream";
    pub const PROXY_SEND_LOCAL_RESPONSE: &str = "proxy_send_local_response";
    pub const PROXY_HTTP_CALL: &str = "proxy_http_call";
    pub const PROXY_SET_EFFECTIVE_CONTEXT: &str = "proxy_set_effective_context";
    pub const PROXY_DONE: &str = "proxy_done";
    pub const PROXY_DEFINE_METRIC: &str = "proxy_define_metric";
    pub const PROXY_GET_METRIC: &str = "proxy_get_metric";
    pub const PROXY_RECORD_METRIC: &str = "proxy_record_metric";
    pub const PROXY_INCREMENT_METRIC: &str = "proxy_increment_metric";
}

extern "C" {
    fn proxy_log(level: LogLevel, message_data: *const u8, message_size: usize) -> Status;
}

/// Logs a message at a given log level.
pub fn log(level: LogLevel, message: &str) -> Result<()> {
    unsafe {
        match proxy_log(level, message.as_ptr(), message.len()) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_LOG, status).into()),
        }
    }
}

extern "C" {
    fn proxy_get_current_time_nanoseconds(return_time: *mut u64) -> Status;
}

/// Returns current system time.
pub fn get_current_time() -> Result<SystemTime> {
    let mut return_time: u64 = 0;
    unsafe {
        match proxy_get_current_time_nanoseconds(&mut return_time) {
            Status::Ok => Ok(UNIX_EPOCH + Duration::from_nanos(return_time)),
            status => {
                Err(HostCallError::new(abi::PROXY_GET_CURRENT_TIME_NANOSECONDS, status).into())
            }
        }
    }
}

extern "C" {
    fn proxy_set_tick_period_milliseconds(period: u32) -> Status;
}

/// Sets the timer to a given period.
pub fn set_tick_period(period: Duration) -> Result<()> {
    unsafe {
        match proxy_set_tick_period_milliseconds(period.as_millis() as u32) {
            Status::Ok => Ok(()),
            status => {
                Err(HostCallError::new(abi::PROXY_SET_TICK_PERIOD_MILLISECONDS, status).into())
            }
        }
    }
}

extern "C" {
    fn proxy_get_buffer_bytes(
        buffer_type: BufferType,
        start: usize,
        max_size: usize,
        return_buffer_data: *mut *mut u8,
        return_buffer_size: *mut usize,
    ) -> Status;
}

/// Returns content from a given buffer.
pub fn get_buffer(
    buffer_type: BufferType,
    start: usize,
    max_size: usize,
) -> Result<Option<ByteString>> {
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match proxy_get_buffer_bytes(
            buffer_type,
            start,
            max_size,
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => {
                if !return_data.is_null() {
                    Ok(Vec::from_raw_parts(return_data, return_size, return_size))
                        .map(ByteString::from)
                        .map(Option::from)
                } else {
                    Ok(None)
                }
            }
            Status::NotFound => Ok(None),
            status => Err(HostCallError::new(abi::PROXY_GET_BUFFER_BYTES, status).into()),
        }
    }
}

extern "C" {
    fn proxy_set_buffer_bytes(
        buffer_type: BufferType,
        start: usize,
        size: usize,
        buffer_data: *const u8,
        buffer_size: usize,
    ) -> Status;
}

/// Mutates content in a given buffer.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
/// use proxy_wasm::types::BufferType;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// hostcalls::set_buffer(BufferType::HttpRequestBody, 0, usize::MAX, "replacement text")?;
/// # Ok(())
/// # }
pub fn set_buffer<B>(buffer_type: BufferType, start: usize, size: usize, value: B) -> Result<()>
where
    B: AsRef<[u8]>,
{
    unsafe {
        match proxy_set_buffer_bytes(
            buffer_type,
            start,
            size,
            value.as_ref().as_ptr(),
            value.as_ref().len(),
        ) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_SET_BUFFER_BYTES, status).into()),
        }
    }
}

extern "C" {
    fn proxy_get_header_map_pairs(
        map_type: MapType,
        return_map_data: *mut *mut u8,
        return_map_size: *mut usize,
    ) -> Status;
}

/// Returns all key-value pairs from a given map.
pub fn get_map(map_type: MapType) -> Result<Vec<(ByteString, ByteString)>> {
    unsafe {
        let mut return_data: *mut u8 = null_mut();
        let mut return_size: usize = 0;
        match proxy_get_header_map_pairs(map_type, &mut return_data, &mut return_size) {
            Status::Ok => {
                if !return_data.is_null() {
                    let serialized_map = Vec::from_raw_parts(return_data, return_size, return_size);
                    utils::deserialize_map(&serialized_map).map_err(|err| {
                        HostResponseError::new(abi::PROXY_GET_HEADER_MAP_PAIRS, err).into()
                    })
                } else {
                    Ok(Vec::new())
                }
            }
            status => Err(HostCallError::new(abi::PROXY_GET_HEADER_MAP_PAIRS, status).into()),
        }
    }
}

extern "C" {
    fn proxy_set_header_map_pairs(
        map_type: MapType,
        map_data: *const u8,
        map_size: usize,
    ) -> Status;
}

/// Sets all key-value pairs in a given map.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
/// use proxy_wasm::types::MapType;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// hostcalls::set_map(MapType::HttpRequestHeaders, &vec![
///     (":method", "GET"),
///     (":path", "/stuff"),
/// ])?;
/// # Ok(())
/// # }
/// ```
pub fn set_map<K, V>(map_type: MapType, map: &[(K, V)]) -> Result<()>
where
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    let serialized_map = utils::serialize_map(map);
    unsafe {
        match proxy_set_header_map_pairs(map_type, serialized_map.as_ptr(), serialized_map.len()) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_SET_HEADER_MAP_PAIRS, status).into()),
        }
    }
}

extern "C" {
    fn proxy_get_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> Status;
}

/// Returns value of a given key from a given map.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
/// use proxy_wasm::types::MapType;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// let value = hostcalls::get_map_value(MapType::HttpRequestHeaders, "authorization")?;
/// # Ok(())
/// # }
/// ```
pub fn get_map_value<K>(map_type: MapType, key: K) -> Result<Option<ByteString>>
where
    K: AsRef<[u8]>,
{
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match proxy_get_header_map_value(
            map_type,
            key.as_ref().as_ptr(),
            key.as_ref().len(),
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => {
                if !return_data.is_null() {
                    Ok(Vec::from_raw_parts(return_data, return_size, return_size))
                        .map(ByteString::from)
                        .map(Option::from)
                } else {
                    Ok(None)
                }
            }
            status => Err(HostCallError::new(abi::PROXY_GET_HEADER_MAP_VALUE, status).into()),
        }
    }
}

extern "C" {
    fn proxy_replace_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
        value_data: *const u8,
        value_size: usize,
    ) -> Status;
}

extern "C" {
    fn proxy_remove_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
    ) -> Status;
}

/// Sets / replaces / removes value of given key from a given map.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
/// use proxy_wasm::types::MapType;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// hostcalls::set_map_value(MapType::HttpRequestHeaders, "authorization", Some("Bearer ..."))?;
/// # Ok(())
/// # }
/// ```
pub fn set_map_value<K, V>(map_type: MapType, key: K, value: Option<V>) -> Result<()>
where
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    unsafe {
        if let Some(value) = value {
            match proxy_replace_header_map_value(
                map_type,
                key.as_ref().as_ptr(),
                key.as_ref().len(),
                value.as_ref().as_ptr(),
                value.as_ref().len(),
            ) {
                Status::Ok => Ok(()),
                status => {
                    Err(HostCallError::new(abi::PROXY_REPLACE_HEADER_MAP_VALUE, status).into())
                }
            }
        } else {
            match proxy_remove_header_map_value(map_type, key.as_ref().as_ptr(), key.as_ref().len())
            {
                Status::Ok => Ok(()),
                status => {
                    Err(HostCallError::new(abi::PROXY_REMOVE_HEADER_MAP_VALUE, status).into())
                }
            }
        }
    }
}

extern "C" {
    fn proxy_add_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
        value_data: *const u8,
        value_size: usize,
    ) -> Status;
}

/// Adds a key-value pair to a given map.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
/// use proxy_wasm::types::MapType;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// hostcalls::add_map_value(MapType::HttpRequestHeaders, "authorization", "Bearer ...")?;
/// # Ok(())
/// # }
/// ```
pub fn add_map_value<K, V>(map_type: MapType, key: K, value: V) -> Result<()>
where
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    unsafe {
        match proxy_add_header_map_value(
            map_type,
            key.as_ref().as_ptr(),
            key.as_ref().len(),
            value.as_ref().as_ptr(),
            value.as_ref().len(),
        ) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_ADD_HEADER_MAP_VALUE, status).into()),
        }
    }
}

extern "C" {
    fn proxy_get_property(
        path_data: *const u8,
        path_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> Status;
}

/// Returns value of a property in the current context.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// let value = hostcalls::get_property(&["request", "time"])?;
/// # Ok(())
/// # }
/// ```
pub fn get_property<P>(path: &[P]) -> Result<Option<ByteString>>
where
    P: AsRef<str>,
{
    let serialized_path = utils::serialize_property_path(path);
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match proxy_get_property(
            serialized_path.as_ptr(),
            serialized_path.len(),
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => {
                if !return_data.is_null() {
                    Ok(Vec::from_raw_parts(return_data, return_size, return_size))
                        .map(ByteString::from)
                        .map(Option::from)
                } else {
                    Ok(None)
                }
            }
            Status::NotFound => Ok(None),
            status => Err(HostCallError::new(abi::PROXY_GET_PROPERTY, status).into()),
        }
    }
}

extern "C" {
    fn proxy_set_property(
        path_data: *const u8,
        path_size: usize,
        value_data: *const u8,
        value_size: usize,
    ) -> Status;
}

/// Sets property to a given value in the current context.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// hostcalls::set_property(&["my_filter", "my_property"], Some("my value"))?;
/// # Ok(())
/// # }
/// ```
pub fn set_property<P, V>(path: &[P], value: Option<V>) -> Result<()>
where
    P: AsRef<str>,
    V: AsRef<[u8]>,
{
    let serialized_path = utils::serialize_property_path(path);
    let (value_ptr, value_len) = value.map_or((null(), 0), |value| {
        (value.as_ref().as_ptr(), value.as_ref().len())
    });
    unsafe {
        match proxy_set_property(
            serialized_path.as_ptr(),
            serialized_path.len(),
            value_ptr,
            value_len,
        ) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_SET_PROPERTY, status).into()),
        }
    }
}

extern "C" {
    fn proxy_get_shared_data(
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
        return_cas: *mut u32,
    ) -> Status;
}

/// Returns shared data by key.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// let (data, version) = hostcalls::get_shared_data("my_shared_key")?;
/// # Ok(())
/// # }
/// ```
pub fn get_shared_data<K>(key: K) -> Result<(Option<ByteString>, Option<u32>)>
where
    K: AsRef<str>,
{
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    let mut return_cas: u32 = 0;
    unsafe {
        match proxy_get_shared_data(
            key.as_ref().as_ptr(),
            key.as_ref().len(),
            &mut return_data,
            &mut return_size,
            &mut return_cas,
        ) {
            Status::Ok => {
                let cas = match return_cas {
                    0 => None,
                    cas => Some(cas),
                };
                if !return_data.is_null() {
                    Ok((
                        Some(Vec::from_raw_parts(return_data, return_size, return_size))
                            .map(ByteString::from),
                        cas,
                    ))
                } else {
                    Ok((None, cas))
                }
            }
            Status::NotFound => Ok((None, None)),
            status => Err(HostCallError::new(abi::PROXY_GET_SHARED_DATA, status).into()),
        }
    }
}

extern "C" {
    fn proxy_set_shared_data(
        key_data: *const u8,
        key_size: usize,
        value_data: *const u8,
        value_size: usize,
        cas: u32,
    ) -> Status;
}

/// Sets shared data by key.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// hostcalls::set_shared_data("my_shared_key", Some("my value"), None)?;
/// # Ok(())
/// # }
/// ```
pub fn set_shared_data<K, V>(key: K, value: Option<V>, cas: Option<u32>) -> Result<()>
where
    K: AsRef<str>,
    V: AsRef<[u8]>,
{
    let (value_ptr, value_len) = value.map_or((null(), 0), |value| {
        (value.as_ref().as_ptr(), value.as_ref().len())
    });
    unsafe {
        match proxy_set_shared_data(
            key.as_ref().as_ptr(),
            key.as_ref().len(),
            value_ptr,
            value_len,
            cas.unwrap_or(0),
        ) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_SET_SHARED_DATA, status).into()),
        }
    }
}

extern "C" {
    fn proxy_register_shared_queue(
        name_data: *const u8,
        name_size: usize,
        return_id: *mut u32,
    ) -> Status;
}

/// Registers a shared queue with a given name.
pub fn register_shared_queue(name: &str) -> Result<u32> {
    unsafe {
        let mut return_id: u32 = 0;
        match proxy_register_shared_queue(name.as_ptr(), name.len(), &mut return_id) {
            Status::Ok => Ok(return_id),
            status => Err(HostCallError::new(abi::PROXY_REGISTER_SHARED_QUEUE, status).into()),
        }
    }
}

extern "C" {
    fn proxy_resolve_shared_queue(
        vm_id_data: *const u8,
        vm_id_size: usize,
        name_data: *const u8,
        name_size: usize,
        return_id: *mut u32,
    ) -> Status;
}

/// Looks up for an existing shared queue with a given name.
pub fn resolve_shared_queue(vm_id: &str, name: &str) -> Result<Option<u32>> {
    let mut return_id: u32 = 0;
    unsafe {
        match proxy_resolve_shared_queue(
            vm_id.as_ptr(),
            vm_id.len(),
            name.as_ptr(),
            name.len(),
            &mut return_id,
        ) {
            Status::Ok => Ok(Some(return_id)),
            Status::NotFound => Ok(None),
            status => Err(HostCallError::new(abi::PROXY_RESOLVE_SHARED_QUEUE, status).into()),
        }
    }
}

extern "C" {
    fn proxy_dequeue_shared_queue(
        queue_id: u32,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> Status;
}

/// Returns data from the end of a given queue.
pub fn dequeue_shared_queue(queue_id: u32) -> Result<Option<ByteString>> {
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match proxy_dequeue_shared_queue(queue_id, &mut return_data, &mut return_size) {
            Status::Ok => {
                if !return_data.is_null() {
                    Ok(Vec::from_raw_parts(return_data, return_size, return_size))
                        .map(ByteString::from)
                        .map(Option::from)
                } else {
                    Ok(None)
                }
            }
            Status::Empty => Ok(None),
            status => Err(HostCallError::new(abi::PROXY_DEQUEUE_SHARED_QUEUE, status).into()),
        }
    }
}

extern "C" {
    fn proxy_enqueue_shared_queue(
        queue_id: u32,
        value_data: *const u8,
        value_size: usize,
    ) -> Status;
}

/// Adds a value to the front of a given queue.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// hostcalls::enqueue_shared_queue(1, Some("my value"))?;
/// # Ok(())
/// # }
/// ```
pub fn enqueue_shared_queue<V>(queue_id: u32, value: Option<V>) -> Result<()>
where
    V: AsRef<[u8]>,
{
    let (value_ptr, value_len) = value.map_or((null(), 0), |value| {
        (value.as_ref().as_ptr(), value.as_ref().len())
    });
    unsafe {
        match proxy_enqueue_shared_queue(queue_id, value_ptr, value_len) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_ENQUEUE_SHARED_QUEUE, status).into()),
        }
    }
}

extern "C" {
    fn proxy_continue_stream(stream: StreamType) -> Status;
}

/// Resumes processing of a given stream, i.e. HTTP request or HTTP response.
pub fn continue_stream(stream_type: StreamType) -> Result<()> {
    unsafe {
        match proxy_continue_stream(stream_type) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_CONTINUE_STREAM, status).into()),
        }
    }
}

extern "C" {
    fn proxy_close_stream(stream: StreamType) -> Status;
}

/// Terminates processing of a given stream, i.e. HTTP request or HTTP response.
pub fn close_stream(stream_type: StreamType) -> Result<()> {
    unsafe {
        match proxy_close_stream(stream_type) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_CLOSE_STREAM, status).into()),
        }
    }
}

extern "C" {
    fn proxy_send_local_response(
        status_code: u32,
        status_code_details_data: *const u8,
        status_code_details_size: usize,
        body_data: *const u8,
        body_size: usize,
        headers_data: *const u8,
        headers_size: usize,
        grpc_status: i32,
    ) -> Status;
}

/// Sends HTTP response without forwarding request to the upstream.
///
/// # Examples
///
/// ```no_run
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// hostcalls::send_http_response(
///     400,
///     hostcalls::NO_HEADERS,
///     hostcalls::NO_BODY,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn send_http_response<K, V, B>(
    status_code: u32,
    headers: &[(K, V)],
    body: Option<B>,
) -> Result<()>
where
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
    B: AsRef<[u8]>,
{
    let serialized_headers = utils::serialize_map(headers);
    let (body_ptr, body_len) = body.map_or((null(), 0), |body| {
        (body.as_ref().as_ptr(), body.as_ref().len())
    });
    unsafe {
        match proxy_send_local_response(
            status_code,
            null(),
            0,
            body_ptr,
            body_len,
            serialized_headers.as_ptr(),
            serialized_headers.len(),
            -1,
        ) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_SEND_LOCAL_RESPONSE, status).into()),
        }
    }
}

extern "C" {
    fn proxy_http_call(
        upstream_data: *const u8,
        upstream_size: usize,
        headers_data: *const u8,
        headers_size: usize,
        body_data: *const u8,
        body_size: usize,
        trailers_data: *const u8,
        trailers_size: usize,
        timeout: u32,
        return_token: *mut u32,
    ) -> Status;
}

/// Dispatches an HTTP call to a given upstream.
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
/// # use proxy_wasm_experimental as proxy_wasm;
/// use proxy_wasm::hostcalls;
///
/// # fn action() -> proxy_wasm::error::Result<()> {
/// let request_handle = hostcalls::dispatch_http_call(
///     "cluster_name",
///     &vec![
///         (":method", "POST"),
///         (":path", "/stuff"),
///     ],
///     Some("hi there!"),
///     hostcalls::NO_TRAILERS,
///     Duration::from_secs(10),
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn dispatch_http_call<K1, V1, K2, V2, B>(
    upstream: &str,
    headers: &[(K1, V1)],
    body: Option<B>,
    trailers: &[(K2, V2)],
    timeout: Duration,
) -> Result<u32>
where
    K1: AsRef<[u8]>,
    V1: AsRef<[u8]>,
    K2: AsRef<[u8]>,
    V2: AsRef<[u8]>,
    B: AsRef<[u8]>,
{
    let serialized_headers = utils::serialize_map(headers);
    let serialized_trailers = utils::serialize_map(trailers);
    let (body_ptr, body_len) = body.map_or((null(), 0), |body| {
        (body.as_ref().as_ptr(), body.as_ref().len())
    });
    let mut return_token: u32 = 0;
    unsafe {
        match proxy_http_call(
            upstream.as_ptr(),
            upstream.len(),
            serialized_headers.as_ptr(),
            serialized_headers.len(),
            body_ptr,
            body_len,
            serialized_trailers.as_ptr(),
            serialized_trailers.len(),
            timeout.as_millis() as u32,
            &mut return_token,
        ) {
            Status::Ok => {
                dispatcher::register_callout(return_token);
                Ok(return_token)
            }
            status => Err(HostCallError::new(abi::PROXY_HTTP_CALL, status).into()),
        }
    }
}

extern "C" {
    fn proxy_set_effective_context(context_id: u32) -> Status;
}

/// Changes the effective context.
pub fn set_effective_context(context_id: u32) -> Result<()> {
    unsafe {
        match proxy_set_effective_context(context_id) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_SET_EFFECTIVE_CONTEXT, status).into()),
        }
    }
}

extern "C" {
    fn proxy_done() -> Status;
}

/// Indicates to the host environment that Wasm VM side is done processing current context.
pub fn done() -> Result<()> {
    unsafe {
        match proxy_done() {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_DONE, status).into()),
        }
    }
}

extern "C" {
    fn proxy_define_metric(
        metric_type: MetricType,
        name_data: *const u8,
        name_size: usize,
        return_id: *mut u32,
    ) -> Status;
}

pub fn define_metric(metric_type: MetricType, name: &str) -> Result<u32> {
    let mut return_id: u32 = 0;
    unsafe {
        match proxy_define_metric(metric_type, name.as_ptr(), name.len(), &mut return_id) {
            Status::Ok => Ok(return_id),
            status => Err(HostCallError::new(abi::PROXY_DEFINE_METRIC, status).into()),
        }
    }
}

extern "C" {
    fn proxy_get_metric(metric_id: u32, return_value: *mut u64) -> Status;
}

pub fn get_metric(metric_id: u32) -> Result<u64> {
    let mut return_value: u64 = 0;
    unsafe {
        match proxy_get_metric(metric_id, &mut return_value) {
            Status::Ok => Ok(return_value),
            status => Err(HostCallError::new(abi::PROXY_GET_METRIC, status).into()),
        }
    }
}

extern "C" {
    fn proxy_record_metric(metric_id: u32, value: u64) -> Status;
}

pub fn record_metric(metric_id: u32, value: u64) -> Result<()> {
    unsafe {
        match proxy_record_metric(metric_id, value) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_RECORD_METRIC, status).into()),
        }
    }
}

extern "C" {
    fn proxy_increment_metric(metric_id: u32, offset: i64) -> Status;
}

pub fn increment_metric(metric_id: u32, offset: i64) -> Result<()> {
    unsafe {
        match proxy_increment_metric(metric_id, offset) {
            Status::Ok => Ok(()),
            status => Err(HostCallError::new(abi::PROXY_INCREMENT_METRIC, status).into()),
        }
    }
}

mod utils {
    use crate::error::Result;
    use crate::types::ByteString;
    use std::convert::TryFrom;

    pub(super) fn serialize_property_path<P>(path: &[P]) -> Vec<u8>
    where
        P: AsRef<str>,
    {
        if path.is_empty() {
            return Vec::new();
        }
        let mut size: usize = 0;
        for part in path {
            size += part.as_ref().len() + 1;
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(size);
        for part in path {
            bytes.extend_from_slice(part.as_ref().as_bytes());
            bytes.push(0);
        }
        bytes.pop();
        bytes
    }

    pub(super) fn serialize_map<K, V>(map: &[(K, V)]) -> Vec<u8>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let mut size: usize = 4;
        for (name, value) in map {
            size += name.as_ref().len() + value.as_ref().len() + 10;
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(size);
        bytes.extend_from_slice(&map.len().to_le_bytes());
        for (name, value) in map {
            bytes.extend_from_slice(&name.as_ref().len().to_le_bytes());
            bytes.extend_from_slice(&value.as_ref().len().to_le_bytes());
        }
        for (name, value) in map {
            bytes.extend_from_slice(name.as_ref());
            bytes.push(0);
            bytes.extend_from_slice(value.as_ref());
            bytes.push(0);
        }
        bytes
    }

    pub(super) fn deserialize_map(bytes: &[u8]) -> Result<Vec<(ByteString, ByteString)>> {
        let mut map = Vec::new();
        if bytes.is_empty() {
            return Ok(map);
        }
        let size = u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[0..4])?) as usize;
        let mut p = 4 + size * 8;
        for n in 0..size {
            let s = 4 + n * 8;
            let size = u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[s..s + 4])?) as usize;
            let key = bytes[p..p + size].to_vec();
            p += size + 1;
            let size = u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[s + 4..s + 8])?) as usize;
            let value = bytes[p..p + size].to_vec();
            p += size + 1;
            map.push((key.into(), value.into()));
        }
        Ok(map)
    }
}
