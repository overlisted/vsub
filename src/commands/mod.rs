pub mod file;
pub mod regex;

use crate::session::CommandInterface;
use std::io;

pub trait Command {
    fn run(&mut self, interface: CommandInterface) -> io::Result<()>;
}
