use {
    crate::{
        http::{Body, Version},
        StatusCode,
    },
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
    pub fn new(status: StatusCode) -> Self {
        Self {
            version: Version::Http10,
            status,
            headers: BTreeMap::new(),
            body: Body::Empty,
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;

        self
    }

    pub fn header<V>(mut self, key: &'static str, value: V) -> Self
    where
        V: ToString,
    {
        self.headers.insert(key, value.to_string());

        self
    }

    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<Body>,
    {
        self.body = body.into();

        self
    }

    pub(crate) fn into_stream(self, stream: &mut TcpStream) -> io::Result<()> {
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
            Body::Bytes(bytes) => {
                write!(stream, "Content-Length: {}\r\n", bytes.len())?;

                write!(stream, "\r\n")?;

                stream.write_all(&bytes)?
            }
        }

        Ok(())
    }
}
