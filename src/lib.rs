//! bluepencil: an editor for rules files. It takes
//! a rules file and an observations file, the
//! evidence of what agents actually did, and produces
//! a proposal file for a human: delete the ignored,
//! rewrite the misread with the misreading quoted,
//! promote the buried and load bearing (BP-1 to
//! BP-17). Nothing is applied automatically, and the
//! tool never writes to the files it judges.
//!
//! Round 3 is the Rust port (BP-18 to BP-26). It
//! consumes no sibling crate: the JSON readers stand
//! in this crate (BP-19, BP-21), and sha2 is the only
//! external dependency, for the target digest.

pub mod cli;
pub mod diff;
pub mod emit;
pub mod failure;
pub mod helptext;
pub mod inventory;
pub mod json;
pub mod observations;
pub mod proposals;
pub mod pyrepr;
pub mod repr_table;
pub mod util;
pub mod value;

/// The command line entry, in the cli module.
pub fn run_cli() -> i32 {
    // A generous stack on a spawned thread, the
    // family precedent: parsed values nest as deep
    // as their files, and cloning or dropping a deep
    // value recurses with its depth.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(crate::cli::run_cli)
        .expect("the CLI thread starts");
    child.join().unwrap_or(1)
}
