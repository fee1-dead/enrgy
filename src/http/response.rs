use {
    crate::http::{Body, StatusCode, Version},
    std::{
        collections::BTreeMap,
        io::{self, Write as _},
        net::TcpStream,
    },
};

pub struct HttpResponse {
    version: Version,
    status: StatusCode,
    headers: BTreeMap<&'static str, String>,
    body: Body,
}

impl HttpResponse {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(status: StatusCode) -> HttpResponseBuilder {
        HttpResponseBuilder {
            inner: Self {
                version: Version::Http10,
                status,
                headers: BTreeMap::new(),
                body: Body::Empty,
            },
        }
    }

    pub fn ok() -> HttpResponseBuilder {
        Self::new(StatusCode::OK)
    }

    pub fn not_found() -> HttpResponseBuilder {
        Self::new(StatusCode::NOT_FOUND)
    }

    pub fn internal_server_error() -> HttpResponseBuilder {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn bad_request() -> HttpResponseBuilder {
        Self::new(StatusCode::BAD_REQUEST)
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub(crate) fn into_stream(self, compress: bool, stream: &mut TcpStream) -> io::Result<()> {
        write!(
            stream,
            "{} {} {}\r\n",
            self.version,
            self.status.0,
            self.status.phrase()
        )?;

        for (key, value) in self.headers {
            write!(stream, "{}: {}\r\n", key, value)?;
        }

        match self.body {
            Body::None | Body::Empty => {
                write!(stream, "Content-Length: 0\r\n")?;
            }
            Body::Bytes(bytes) if compress => {
                write!(stream, "Content-Encoding: gzip\r\n")?;

                let compressed = {
                    let mut buf = Vec::with_capacity(bytes.len());

                    {
                        let mut encoder = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default());

                        encoder.write_all(&bytes)?;
                    }

                    buf
                };

                write!(stream, "Content-Length: {}\r\n", compressed.len())?;

                write!(stream, "\r\n")?;

                stream.write_all(&compressed)?
            }
            Body::Bytes(bytes) => {
                write!(stream, "Content-Length: {}\r\n", bytes.len())?;

                write!(stream, "\r\n")?;

                stream.write_all(&bytes)?
            }
        }

        Ok(())
    }
}

pub struct HttpResponseBuilder {
    inner: HttpResponse,
}

impl HttpResponseBuilder {
    pub fn status(mut self, status: StatusCode) -> Self {
        self.inner.status = status;

        self
    }

    pub fn header<V>(mut self, key: &'static str, value: V) -> Self
    where
        V: ToString,
    {
        self.inner.headers.insert(key, value.to_string());

        self
    }

    pub fn body<B>(mut self, body: B) -> HttpResponse
    where
        B: Into<Body>,
    {
        self.inner.body = body.into();

        self.inner
    }

    pub fn finish(self) -> HttpResponse {
        self.inner
    }
}
