#![warn(clippy::all, clippy::pedantic)]

mod interface;
mod tasks;

use crate::tasks::Task;
use crate::tasks::TaskManager;
use crate::tasks::TaskStatus;
use interface::Interface;

fn main() {
    let mut interface = Interface::new();
    interface.run();
}
