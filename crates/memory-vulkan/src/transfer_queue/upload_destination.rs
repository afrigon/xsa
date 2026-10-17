use crate::transfer_queue::UploadTarget;

// Where an upload's bytes land. An owned destination is a whole resource the request keeps alive and its ticket hands
// back: an image is undefined before its first copy and ready to sample after its last.
pub(crate) enum UploadDestination {
    Owned(UploadTarget),
}
