// What a request does once its last copy completes or its submission fails.
pub(crate) trait TransferCompletion: Send {
    // Its ticket was dropped, so nobody wants the result.
    fn is_cancelled(&self) -> bool;

    fn complete(self: Box<Self>);

    fn fail(self: Box<Self>, error: crate::MemoryError);
}
