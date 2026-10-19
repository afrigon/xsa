use crate::transfer_queue::UploadTarget;

// Where an upload's bytes land. An owned destination is a whole resource the request keeps alive and its ticket hands
// back: an image is undefined before its first copy and in its target layout after its last. A region is part of an
// image that stays in `GENERAL` layout, where frames may sample the rest of it while it is written.
pub(crate) enum UploadDestination {
    Owned(UploadTarget),
    Region(UploadTarget),
}
