//! Transit Feed provides a consistent set of data structures, parsers,
//! and API clients for obtaining usable transit related information
//! such as routes, stop, trips, stop times, and more.

extern crate quick_csv;
extern crate chrono;
extern crate zip;

mod transit;
mod gtfs;
mod run;

pub use transit::*;
pub use gtfs::{GTFSIterator, agencies, calendars, calendar_dates, frequencies, routes, shapes, stops, stop_times, trips};
pub use run::{RunIterator, Run};
