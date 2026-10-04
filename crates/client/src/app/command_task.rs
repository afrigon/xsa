use super::App;
use super::task_status::TaskStatus;

// A command spanning several frames; polled once per frame, after drawing, until it is done.
pub(super) trait CommandTask {
    fn poll(&mut self, app: &mut App) -> TaskStatus;
}
