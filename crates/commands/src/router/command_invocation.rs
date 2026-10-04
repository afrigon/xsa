use super::CommandExecution;
use crate::completion::CommandCompletion;

pub enum CommandInvocation {
    Execute(CommandExecution),
    Complete(CommandCompletion),
}
