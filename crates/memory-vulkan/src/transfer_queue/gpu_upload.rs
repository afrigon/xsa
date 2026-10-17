use crate::{MemoryError, TransferStatus, UploadTicket};

/// A resource being uploaded, held by whoever owns it: pending until its ticket completes, then ready to use.
pub enum GpuUpload<T> {
    Pending(UploadTicket<T>),
    Ready(T),
}

impl<T> GpuUpload<T> {
    /// The resource, once its upload completed.
    ///
    /// # Panics
    ///
    /// If polled again after it returned an error.
    pub fn poll(&mut self) -> Result<Option<&T>, MemoryError> {
        if let GpuUpload::Pending(ticket) = self {
            match ticket.poll() {
                TransferStatus::Pending => return Ok(None),
                TransferStatus::Complete(resource) => *self = GpuUpload::Ready(resource),
                TransferStatus::Failed(error) => return Err(error),
            }
        }

        Ok(self.get())
    }

    /// Blocks until the upload completes.
    ///
    /// # Panics
    ///
    /// If called again after it returned an error.
    pub fn wait(&mut self) -> Result<&T, MemoryError> {
        if let GpuUpload::Pending(ticket) = self {
            *self = GpuUpload::Ready(ticket.wait_settled()?);
        }

        Ok(self.get().expect("a waited upload is ready"))
    }

    pub fn get(&self) -> Option<&T> {
        match self {
            GpuUpload::Pending(_) => None,
            GpuUpload::Ready(resource) => Some(resource),
        }
    }
}

impl<T> From<UploadTicket<T>> for GpuUpload<T> {
    fn from(ticket: UploadTicket<T>) -> GpuUpload<T> {
        GpuUpload::Pending(ticket)
    }
}
