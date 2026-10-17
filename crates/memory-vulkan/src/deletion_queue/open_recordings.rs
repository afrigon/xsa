use crate::RecordingId;

/// The recordings open on any timeline, and the entries held until every recording open when they were queued is
/// submitted or abandoned: an entry cannot know which recording uses it, nor what value that recording will signal.
pub(crate) struct OpenRecordings<T> {
    open: Vec<RecordingId>,
    next: u64,
    held: Vec<Hold<T>>,
}

struct Hold<T> {
    awaited: Vec<RecordingId>,
    entries: Vec<T>,
}

impl<T> OpenRecordings<T> {
    pub fn open(&mut self) -> RecordingId {
        let id = RecordingId { index: self.next };
        self.next += 1;
        self.open.push(id);

        id
    }

    /// Holds `entry` until the recordings open now close, or hands it back when none is open.
    pub fn hold(&mut self, entry: T) -> Option<T> {
        if self.open.is_empty() {
            return Some(entry);
        }

        if let Some(last) = self.held.last_mut()
            && last.awaited == self.open
        {
            last.entries.push(entry);
            return None;
        }

        self.held.push(Hold {
            awaited: self.open.clone(),
            entries: vec![entry],
        });

        None
    }

    /// Closes `id` and returns the entries that no longer wait for any recording, in the order they were held.
    pub fn close(&mut self, id: RecordingId) -> Vec<T> {
        self.open.retain(|&open| open != id);
        let mut released = Vec::new();

        for hold in &mut self.held {
            hold.awaited.retain(|&awaited| awaited != id);
        }

        while let Some(first) = self.held.first()
            && first.awaited.is_empty()
        {
            released.extend(self.held.remove(0).entries);
        }

        released
    }
}

impl<T> Default for OpenRecordings<T> {
    fn default() -> Self {
        OpenRecordings {
            open: Vec::new(),
            next: 0,
            held: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_held_without_an_open_recording() {
        let mut recordings = OpenRecordings::default();

        assert_eq!(recordings.hold(1), Some(1));
    }

    #[test]
    fn an_entry_waits_for_its_recording() {
        let mut recordings = OpenRecordings::default();
        let recording = recordings.open();

        assert_eq!(recordings.hold(1), None);
        assert_eq!(recordings.close(recording), vec![1]);
    }

    #[test]
    fn an_entry_waits_for_every_recording_open_when_it_was_held() {
        let mut recordings = OpenRecordings::default();
        let first = recordings.open();
        let second = recordings.open();
        recordings.hold(1);

        assert_eq!(recordings.close(second), Vec::<u32>::new());
        assert_eq!(recordings.close(first), vec![1]);
    }

    #[test]
    fn a_recording_opened_later_does_not_delay_an_entry() {
        let mut recordings = OpenRecordings::default();
        let first = recordings.open();
        recordings.hold(1);
        let second = recordings.open();
        recordings.hold(2);

        assert_eq!(recordings.close(first), vec![1]);
        assert_eq!(recordings.close(second), vec![2]);
    }

    #[test]
    fn entries_are_released_in_the_order_they_were_held() {
        let mut recordings = OpenRecordings::default();
        let first = recordings.open();
        recordings.hold(1);
        let second = recordings.open();
        recordings.hold(2);
        recordings.hold(3);

        assert_eq!(recordings.close(second), Vec::<u32>::new());
        assert_eq!(recordings.close(first), vec![1, 2, 3]);
    }
}
