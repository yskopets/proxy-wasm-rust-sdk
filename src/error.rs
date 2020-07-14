// Copyright 2020 Tetrate
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;

use crate::types::Status;

/// A boxed [`Error`].
///
/// [`Error`]: https://doc.rust-lang.org/std/fmt/struct.Error.html
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized [`Result`] type.
///
/// [`Result`]: https://doc.rust-lang.org/core/result/enum.Result.html
pub type Result<T> = core::result::Result<T, Error>;

/// An error to call a Host ABI function.
#[derive(Debug)]
pub struct HostCallError<'a> {
    function: &'a str,
    status: Status,
}

impl<'a> HostCallError<'a> {
    pub(crate) fn new(function: &'a str, status: Status) -> Self {
        HostCallError { function, status }
    }

    pub fn module(&self) -> &'a str {
        return "env"
    }

    pub fn function(&self) -> &'a str {
        return self.function
    }

    pub fn status(&self) -> Status {
        return self.status
    }
}

impl<'a> fmt::Display for HostCallError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "call to the host ABI function \"{}.{}\" has failed with status code {}",
            "env",
            self.function,
            self.status as u32,
        )
    }
}

impl<'a> std::error::Error for HostCallError<'a> {}

/// An error to parse the response from a Host ABI.
#[derive(Debug)]
pub struct HostResponseError<'a> {
    function: &'a str,
    error: Error,
}

impl<'a> HostResponseError<'a> {
    pub(crate) fn new(function: &'a str, error: Error) -> Self {
        HostResponseError { function, error }
    }

    pub fn module(&self) -> &'a str {
        return "env"
    }

    pub fn function(&self) -> &'a str {
        return self.function
    }
}

impl<'a> fmt::Display for HostResponseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "failed to parse response from the host ABI function \"{}.{}\": {}",
            "env",
            self.function,
            self.error,
        )
    }
}

impl<'a> std::error::Error for HostResponseError<'a> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.error)
    }
}
