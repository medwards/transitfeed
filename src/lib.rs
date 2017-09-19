//! Transit Feed provides a consistent set of data structures, parsers,
//! and API clients for obtaining usable transit related information
//! such as routes, stop, trips, stop times, and more.

extern crate chrono;
extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate zip;

mod archive;
pub mod feed;
mod gtfs;
pub mod run;
mod transit;

pub use feed::{FeedReader, Terminator, TransitFeed, Trim};
pub use gtfs::{Error, GTFSIterator};
pub use run::{Run, RunIterator};
pub use transit::*;
