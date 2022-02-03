use crate::session::CommandInterface;
use std::io;

pub struct Write;

impl super::Command for Write {
    fn run(&mut self, mut interface: CommandInterface) -> io::Result<()> {
        let n = interface.save_buffer()?;
        interface.finalize(true, format!("{} bytes written", n));

        Ok(())
    }
}
