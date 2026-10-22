use std::io::Write;

use anstream::StripStream;
use tracing_subscriber::fmt::MakeWriter;

// Events are always formatted with ANSI styles, so any message may style itself; this strips them when the log
// is not colored.
pub(crate) struct ColoredWriter<Writer> {
    writer: Writer,
    colored: bool,
}

impl<Writer> ColoredWriter<Writer> {
    pub fn new(writer: Writer, colored: bool) -> ColoredWriter<Writer> {
        ColoredWriter { writer, colored }
    }
}

impl<'writer, Writer> MakeWriter<'writer> for ColoredWriter<Writer>
where
    Writer: MakeWriter<'writer>,
    Writer::Writer: Send + 'static,
{
    type Writer = Box<dyn Write + 'writer>;

    fn make_writer(&'writer self) -> Self::Writer {
        let writer: Box<dyn Write + Send> = Box::new(self.writer.make_writer());

        if self.colored {
            writer
        } else {
            Box::new(StripStream::new(writer))
        }
    }
}
