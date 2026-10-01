//! HTTP protocol settings module

/// HTTP/1.1 connection settings builder
pub struct Http1SettingsBuilder {
    write_v: bool,
    title_case_headers: bool,
    preserve_header_case: bool,
    max_headers: usize,
    read_buf_exact_size: u32,
    max_buf_size: u32,
}

impl Http1SettingsBuilder {
    /// Set whether HTTP/1 connections should try to use vectored writes,
    /// or always flatten into a single buffer.
    pub fn write_v(mut self, write_v: bool) -> Self {
        self.write_v = write_v;
        self
    }

    /// Set whether HTTP/1 connections will write header names as title case at
    /// the socket level.
    pub fn title_case_headers(mut self, title_case_headers: bool) -> Self {
        self.title_case_headers = title_case_headers;
        self
    }

    /// Set whether to support preserving original header cases.
    pub fn preserve_header_case(mut self, preserve_header_case: bool) -> Self {
        self.preserve_header_case = preserve_header_case;
        self
    }

    /// Set the maximum number of headers.
    pub fn max_headers(mut self, max_headers: usize) -> Self {
        self.max_headers = max_headers;
        self
    }

    /// Sets the exact size of the read buffer to *always* use.
    pub fn read_buf_exact_size(mut self, read_buf_exact_size: u32) -> Self {
        self.read_buf_exact_size = read_buf_exact_size;
        self
    }

    /// Set the maximum buffer size for the connection.
    pub fn max_buf_size(mut self, max_buf_size: u32) -> Self {
        self.max_buf_size = max_buf_size;
        self
    }

    /// build HTTP/1.1 settings
    pub fn build(self) -> Http1Settings {
        Http1Settings {
            write_v: self.write_v,
            title_case_headers: self.title_case_headers,
            preserve_header_case: self.preserve_header_case,
            max_headers: self.max_headers,
            read_buf_exact_size: self.read_buf_exact_size,
            max_buf_size: self.max_buf_size,
        }
    }
}

/// HTTP/1.1 connection settings
pub struct Http1Settings {
    write_v: bool,
    title_case_headers: bool,
    preserve_header_case: bool,
    max_headers: usize,
    read_buf_exact_size: u32,
    max_buf_size: u32,
}

impl Http1Settings {
    /// Builder for HTTP/1.1 settings
    pub fn builder() -> Http1SettingsBuilder {
        Http1SettingsBuilder {
            write_v: false,
            title_case_headers: false,
            preserve_header_case: false,
            max_headers: 100,
            read_buf_exact_size: 21,
            max_buf_size: 12,
        }
    }

    /// Indicates whether HTTP/1 connections should try to use vectored writes,
    /// or always flatten into a single buffer.
    pub fn write_v(&self) -> bool {
        self.write_v
    }

    /// Indicates whether HTTP/1 connections will write header names as title case at
    /// the socket level.
    pub fn title_case_headers(&self) -> bool {
        self.title_case_headers
    }

    /// Indicates whether to support preserving original header cases.
    pub fn preserve_header_case(&self) -> bool {
        self.preserve_header_case
    }

    /// Returns the maximum number of headers.
    pub fn max_headers(&self) -> usize {
        self.max_headers
    }

    /// Returns the exact size of the read buffer to *always* use.
    pub fn read_buf_exact_size(&self) -> u32 {
        self.read_buf_exact_size
    }

    /// Returns the maximum buffer size for the connection.
    pub fn max_buf_size(&self) -> u32 {
        self.max_buf_size
    }
}

impl Default for Http1Settings {
    fn default() -> Self {
        Http1Settings {
            write_v: false,
            title_case_headers: false,
            preserve_header_case: false,
            max_headers: 100,
            read_buf_exact_size: 8192,
            max_buf_size: 100,
        }
    }
}

/// HTTP/2 connection settings builder
pub struct Http2SettingsBuilder {}

impl Http2SettingsBuilder {}

/// HTTP/2 connection settings
pub struct Http2Settings {}

impl Http2Settings {}

impl Default for Http2Settings {
    fn default() -> Self {
        Http2Settings {}
    }
}

/// HTTP/3 connection settings builder
pub struct Http3SettingsBuilder {}

impl Http3SettingsBuilder {}

/// HTTP/3 connection settings
pub struct Http3Settings {}

impl Http3Settings {}

impl Default for Http3Settings {
    fn default() -> Self {
        Http3Settings {}
    }
}

/// Protocol settings builder allow us to customize each HTTP
/// protocol implementation in a less caotic way
pub struct ProtocolSettingsBuilder {
    h1: Option<Http1Settings>,
    h2: Option<Http2Settings>,
    h3: Option<Http3Settings>,
}

impl ProtocolSettingsBuilder {
    /// Set custom HTTP/1.1 settings
    pub fn h1(mut self, h1: Http1Settings) -> Self {
        self.h1 = Some(h1);
        self
    }

    /// Set custom HTTP/2 settings
    pub fn h2(mut self, h2: Http2Settings) -> Self {
        self.h2 = Some(h2);
        self
    }

    /// Set custom HTTP/3 settings
    pub fn h3(mut self, h3: Http3Settings) -> Self {
        self.h3 = Some(h3);
        self
    }

    /// Build protocol settings
    pub fn build(self) -> ProtocolSettings {
        ProtocolSettings { h1: self.h1, h2: self.h2, h3: self.h3 }
    }
}

#[derive(Default)]
/// Protocol settings for various HTTP versions.
pub struct ProtocolSettings {
    h1: Option<Http1Settings>,
    h2: Option<Http2Settings>,
    h3: Option<Http3Settings>,
}

impl ProtocolSettings {
    /// Create a new protocol settings builder
    pub fn builder(self) -> ProtocolSettingsBuilder {
        ProtocolSettingsBuilder { h1: None, h2: None, h3: None }
    }

    /// Return HTTP/1.1 settings
    pub fn h1(&self) -> &Option<Http1Settings> {
        &self.h1
    }

    /// Return HTTP/2 settings
    pub fn h2(&self) -> &Option<Http2Settings> {
        &self.h2
    }

    /// Return HTTP/3 settings
    pub fn h3(&self) -> &Option<Http3Settings> {
        &self.h3
    }
}
