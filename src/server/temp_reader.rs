use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io::{AsyncRead, ReadBuf};

/// Reads a temporary file and deletes it when the download is finished or cancelled.
pub struct TempFileReader {
    file: File,
    path: PathBuf,
}

impl TempFileReader {
    pub fn new(file: File, path: PathBuf) -> Self {
        Self { file, path }
    }
}

impl AsyncRead for TempFileReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.file).poll_read(context, buffer)
    }
}

impl Drop for TempFileReader {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
