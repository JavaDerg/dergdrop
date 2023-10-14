use std::pin::Pin;
use std::task::{Context, Poll, ready};
use bytes::Bytes;
use futures::Stream;
use pin_project::pin_project;
use tokio::fs::File;
use tokio::io::{AsyncRead, ReadBuf};

#[pin_project]
pub struct FileStream {
    #[pin]
    file: File,
    buffer: Box<[u8]>,
}

impl FileStream {
    pub fn new(file: File) -> Self {
        Self {
            file,
            buffer: Box::new([0; 32 * 1024]),
        }
    }
}

impl Stream for FileStream {
    type Item = Result<Bytes, axum::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let mut buffer = ReadBuf::new(&mut this.buffer[..]);

        match ready!(this.file.as_mut().poll_read(cx, &mut buffer)) {
            Ok(_) if buffer.filled().is_empty() => return Poll::Ready(None),
            Err(err) => return Poll::Ready(Some(Err(axum::Error::new(err)))),
            _ => (),
        }

        let bytes = Bytes::copy_from_slice(buffer.filled());
        Poll::Ready(Some(Ok(bytes)))
    }
}
