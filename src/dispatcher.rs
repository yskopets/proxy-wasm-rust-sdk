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

use crate::hostcalls;
use crate::traits::*;
use crate::types::*;
use hashbrown::HashMap;
use std::cell::{Cell, RefCell};

thread_local! {
static DISPATCHER: Dispatcher = Dispatcher::new();
}

type NewRootContextFn = dyn FnMut(u32) -> Box<dyn RootContext>;
type NewStreamContextFn = dyn FnMut(u32, u32) -> Box<dyn StreamContext>;
type NewHttpContextFn = dyn FnMut(u32, u32) -> Box<dyn HttpContext>;

pub(crate) fn set_root_context(callback: Box<NewRootContextFn>) {
    DISPATCHER.with(|dispatcher| dispatcher.set_root_context(callback));
}

pub(crate) fn set_stream_context(callback: Box<NewStreamContextFn>) {
    DISPATCHER.with(|dispatcher| dispatcher.set_stream_context(callback));
}

pub(crate) fn set_http_context(callback: Box<NewHttpContextFn>) {
    DISPATCHER.with(|dispatcher| dispatcher.set_http_context(callback));
}

pub(crate) fn register_callout(token_id: u32) {
    DISPATCHER.with(|dispatcher| dispatcher.register_callout(token_id));
}

struct NoopRoot;

impl Context for NoopRoot {}
impl RootContext for NoopRoot {}

struct Dispatcher {
    new_root: RefCell<Option<Box<NewRootContextFn>>>,
    roots: RefCell<HashMap<u32, Box<dyn RootContext>>>,
    new_stream: RefCell<Option<Box<NewStreamContextFn>>>,
    streams: RefCell<HashMap<u32, Box<dyn StreamContext>>>,
    new_http_stream: RefCell<Option<Box<NewHttpContextFn>>>,
    http_streams: RefCell<HashMap<u32, Box<dyn HttpContext>>>,
    active_id: Cell<u32>,
    callouts: RefCell<HashMap<u32, u32>>,
}

impl Dispatcher {
    fn new() -> Dispatcher {
        Dispatcher {
            new_root: RefCell::new(None),
            roots: RefCell::new(HashMap::new()),
            new_stream: RefCell::new(None),
            streams: RefCell::new(HashMap::new()),
            new_http_stream: RefCell::new(None),
            http_streams: RefCell::new(HashMap::new()),
            active_id: Cell::new(0),
            callouts: RefCell::new(HashMap::new()),
        }
    }

    fn set_root_context(&self, callback: Box<NewRootContextFn>) {
        self.new_root.replace(Some(callback));
    }

    fn set_stream_context(&self, callback: Box<NewStreamContextFn>) {
        self.new_stream.replace(Some(callback));
    }

    fn set_http_context(&self, callback: Box<NewHttpContextFn>) {
        self.new_http_stream.replace(Some(callback));
    }

    fn register_callout(&self, token_id: u32) {
        if self
            .callouts
            .borrow_mut()
            .insert(token_id, self.active_id.get())
            .is_some()
        {
            panic!("duplicate token_id")
        }
    }

    fn create_root_context(&self, context_id: u32) {
        let new_context = match *self.new_root.borrow_mut() {
            Some(ref mut f) => f(context_id),
            None => Box::new(NoopRoot),
        };
        if self
            .roots
            .borrow_mut()
            .insert(context_id, new_context)
            .is_some()
        {
            panic!("duplicate context_id")
        }
    }

    fn create_stream_context(&self, context_id: u32, root_context_id: u32) {
        let new_context = match self.roots.borrow().get(&root_context_id) {
            Some(root_context) => match *self.new_stream.borrow_mut() {
                Some(ref mut f) => f(context_id, root_context_id),
                None => match root_context.create_stream_context(context_id) {
                    Some(stream_context) => stream_context,
                    None => panic!("create_stream_context returned None"),
                },
            },
            None => panic!("invalid root_context_id"),
        };
        if self
            .streams
            .borrow_mut()
            .insert(context_id, new_context)
            .is_some()
        {
            panic!("duplicate context_id")
        }
    }

    fn create_http_context(&self, context_id: u32, root_context_id: u32) {
        let new_context = match self.roots.borrow().get(&root_context_id) {
            Some(root_context) => match *self.new_http_stream.borrow_mut() {
                Some(ref mut f) => f(context_id, root_context_id),
                None => match root_context.create_http_context(context_id) {
                    Some(stream_context) => stream_context,
                    None => panic!("create_http_context returned None"),
                },
            },
            None => panic!("invalid root_context_id"),
        };
        if self
            .http_streams
            .borrow_mut()
            .insert(context_id, new_context)
            .is_some()
        {
            panic!("duplicate context_id")
        }
    }

    fn on_create_context(&self, context_id: u32, root_context_id: u32) {
        if root_context_id == 0 {
            self.create_root_context(context_id);
        } else if self.new_http_stream.borrow().is_some() {
            self.create_http_context(context_id, root_context_id);
        } else if self.new_stream.borrow().is_some() {
            self.create_stream_context(context_id, root_context_id);
        } else if let Some(root_context) = self.roots.borrow().get(&root_context_id) {
            match root_context.get_type() {
                Some(ContextType::HttpContext) => {
                    self.create_http_context(context_id, root_context_id)
                }
                Some(ContextType::StreamContext) => {
                    self.create_stream_context(context_id, root_context_id)
                }
                None => panic!("missing ContextType on root_context"),
            }
        } else {
            panic!("invalid root_context_id and missing constructors");
        }
    }

    fn on_done(&self, context_id: u32) -> bool {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            http_stream.on_done()
        } else if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            stream.on_done()
        } else if let Some(root) = self.roots.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            root.on_done()
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_log(&self, context_id: u32) {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            http_stream.on_log()
        } else if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            stream.on_log()
        } else if let Some(root) = self.roots.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            root.on_log()
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_delete(&self, context_id: u32) {
        if !(self.http_streams.borrow_mut().remove(&context_id).is_some()
            || self.streams.borrow_mut().remove(&context_id).is_some()
            || self.roots.borrow_mut().remove(&context_id).is_some())
        {
            panic!("invalid context_id")
        }
    }

    fn on_vm_start(&self, context_id: u32, vm_configuration_size: usize) -> bool {
        if let Some(root) = self.roots.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            root.on_vm_start(vm_configuration_size)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_configure(&self, context_id: u32, plugin_configuration_size: usize) -> bool {
        if let Some(root) = self.roots.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            root.on_configure(plugin_configuration_size)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_tick(&self, context_id: u32) {
        if let Some(root) = self.roots.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            root.on_tick()
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_queue_ready(&self, context_id: u32, queue_id: u32) {
        if let Some(root) = self.roots.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            root.on_queue_ready(queue_id)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_new_connection(&self, context_id: u32) -> Action {
        if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            stream.on_new_connection()
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_downstream_data(&self, context_id: u32, data_size: usize, end_of_stream: bool) -> Action {
        if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            stream.on_downstream_data(data_size, end_of_stream)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_downstream_close(&self, context_id: u32, peer_type: PeerType) {
        if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            stream.on_downstream_close(peer_type)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_upstream_data(&self, context_id: u32, data_size: usize, end_of_stream: bool) -> Action {
        if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            stream.on_upstream_data(data_size, end_of_stream)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_upstream_close(&self, context_id: u32, peer_type: PeerType) {
        if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            stream.on_upstream_close(peer_type)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_http_request_headers(
        &self,
        context_id: u32,
        num_headers: usize,
        end_of_stream: bool,
    ) -> Action {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            http_stream.on_http_request_headers(num_headers, end_of_stream)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_http_request_body(
        &self,
        context_id: u32,
        body_size: usize,
        end_of_stream: bool,
    ) -> Action {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            http_stream.on_http_request_body(body_size, end_of_stream)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_http_request_trailers(&self, context_id: u32, num_trailers: usize) -> Action {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            http_stream.on_http_request_trailers(num_trailers)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_http_response_headers(
        &self,
        context_id: u32,
        num_headers: usize,
        end_of_stream: bool,
    ) -> Action {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            http_stream.on_http_response_headers(num_headers, end_of_stream)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_http_response_body(
        &self,
        context_id: u32,
        body_size: usize,
        end_of_stream: bool,
    ) -> Action {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            http_stream.on_http_response_body(body_size, end_of_stream)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_http_response_trailers(&self, context_id: u32, num_trailers: usize) -> Action {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            http_stream.on_http_response_trailers(num_trailers)
        } else {
            panic!("invalid context_id")
        }
    }

    fn on_http_call_response(
        &self,
        token_id: u32,
        num_headers: usize,
        body_size: usize,
        num_trailers: usize,
    ) {
        let context_id = self
            .callouts
            .borrow_mut()
            .remove(&token_id)
            .expect("invalid token_id");

        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            hostcalls::set_effective_context(context_id).unwrap();
            http_stream.on_http_call_response(token_id, num_headers, body_size, num_trailers)
        } else if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            hostcalls::set_effective_context(context_id).unwrap();
            stream.on_http_call_response(token_id, num_headers, body_size, num_trailers)
        } else if let Some(root) = self.roots.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            hostcalls::set_effective_context(context_id).unwrap();
            root.on_http_call_response(token_id, num_headers, body_size, num_trailers)
        }
    }
}

#[no_mangle]
pub extern "C" fn proxy_on_context_create(context_id: u32, root_context_id: u32) {
    DISPATCHER.with(|dispatcher| dispatcher.on_create_context(context_id, root_context_id))
}

#[no_mangle]
pub extern "C" fn proxy_on_done(context_id: u32) -> bool {
    DISPATCHER.with(|dispatcher| dispatcher.on_done(context_id))
}

#[no_mangle]
pub extern "C" fn proxy_on_log(context_id: u32) {
    DISPATCHER.with(|dispatcher| dispatcher.on_log(context_id))
}

#[no_mangle]
pub extern "C" fn proxy_on_delete(context_id: u32) {
    DISPATCHER.with(|dispatcher| dispatcher.on_delete(context_id))
}

#[no_mangle]
pub extern "C" fn proxy_on_vm_start(context_id: u32, vm_configuration_size: usize) -> bool {
    DISPATCHER.with(|dispatcher| dispatcher.on_vm_start(context_id, vm_configuration_size))
}

#[no_mangle]
pub extern "C" fn proxy_on_configure(context_id: u32, plugin_configuration_size: usize) -> bool {
    DISPATCHER.with(|dispatcher| dispatcher.on_configure(context_id, plugin_configuration_size))
}

#[no_mangle]
pub extern "C" fn proxy_on_tick(context_id: u32) {
    DISPATCHER.with(|dispatcher| dispatcher.on_tick(context_id))
}

#[no_mangle]
pub extern "C" fn proxy_on_queue_ready(context_id: u32, queue_id: u32) {
    DISPATCHER.with(|dispatcher| dispatcher.on_queue_ready(context_id, queue_id))
}

#[no_mangle]
pub extern "C" fn proxy_on_new_connection(context_id: u32) -> Action {
    DISPATCHER.with(|dispatcher| dispatcher.on_new_connection(context_id))
}

#[no_mangle]
pub extern "C" fn proxy_on_downstream_data(
    context_id: u32,
    data_size: usize,
    end_of_stream: bool,
) -> Action {
    DISPATCHER
        .with(|dispatcher| dispatcher.on_downstream_data(context_id, data_size, end_of_stream))
}

#[no_mangle]
pub extern "C" fn proxy_on_downstream_connection_close(context_id: u32, peer_type: PeerType) {
    DISPATCHER.with(|dispatcher| dispatcher.on_downstream_close(context_id, peer_type))
}

#[no_mangle]
pub extern "C" fn proxy_on_upstream_data(
    context_id: u32,
    data_size: usize,
    end_of_stream: bool,
) -> Action {
    DISPATCHER.with(|dispatcher| dispatcher.on_upstream_data(context_id, data_size, end_of_stream))
}

#[no_mangle]
pub extern "C" fn proxy_on_upstream_connection_close(context_id: u32, peer_type: PeerType) {
    DISPATCHER.with(|dispatcher| dispatcher.on_upstream_close(context_id, peer_type))
}

#[no_mangle]
pub extern "C" fn proxy_on_request_headers(
    context_id: u32,
    num_headers: usize,
    end_of_stream: bool,
) -> Action {
    DISPATCHER.with(|dispatcher| {
        dispatcher.on_http_request_headers(context_id, num_headers, end_of_stream)
    })
}

#[no_mangle]
pub extern "C" fn proxy_on_request_body(
    context_id: u32,
    body_size: usize,
    end_of_stream: bool,
) -> Action {
    DISPATCHER
        .with(|dispatcher| dispatcher.on_http_request_body(context_id, body_size, end_of_stream))
}

#[no_mangle]
pub extern "C" fn proxy_on_request_trailers(context_id: u32, num_trailers: usize) -> Action {
    DISPATCHER.with(|dispatcher| dispatcher.on_http_request_trailers(context_id, num_trailers))
}

#[no_mangle]
pub extern "C" fn proxy_on_response_headers(
    context_id: u32,
    num_headers: usize,
    end_of_stream: bool,
) -> Action {
    DISPATCHER.with(|dispatcher| {
        dispatcher.on_http_response_headers(context_id, num_headers, end_of_stream)
    })
}

#[no_mangle]
pub extern "C" fn proxy_on_response_body(
    context_id: u32,
    body_size: usize,
    end_of_stream: bool,
) -> Action {
    DISPATCHER
        .with(|dispatcher| dispatcher.on_http_response_body(context_id, body_size, end_of_stream))
}

#[no_mangle]
pub extern "C" fn proxy_on_response_trailers(context_id: u32, num_trailers: usize) -> Action {
    DISPATCHER.with(|dispatcher| dispatcher.on_http_response_trailers(context_id, num_trailers))
}

#[no_mangle]
pub extern "C" fn proxy_on_http_call_response(
    _context_id: u32,
    token_id: u32,
    num_headers: usize,
    body_size: usize,
    num_trailers: usize,
) {
    DISPATCHER.with(|dispatcher| {
        dispatcher.on_http_call_response(token_id, num_headers, body_size, num_trailers)
    })
}
