use super::App;
use super::task_status::TaskStatus;

// A command spanning several updates; polled after each one until it is done.
pub(super) trait CommandTask {
    fn poll(&mut self, app: &mut App) -> TaskStatus;

    // Whether the task progresses only as frames render, so updates render one even while the window draws none.
    fn needs_frames(&self) -> bool {
        false
    }
}
