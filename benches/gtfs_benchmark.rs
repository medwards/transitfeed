#![feature(test)]
extern crate transitfeed;
extern crate test;
extern crate quick_csv;

use std::fmt::{Debug, Display};
use std::fs;
use std::io::Read;
use test::Bencher;
use quick_csv::Csv;
use transitfeed::{GTFSIterator, agencies, calendars, calendar_dates, frequencies, routes, shapes, stops, stop_times, trips};

const AGENCY_DATA: &'static str = "./examples/bench/agency.txt";
const CALENDAR_DATA: &'static str = "./examples/bench/calendar.txt";
const CALENDAR_DATE_DATA: &'static str = "./examples/bench/calendar_dates.txt";
const ROUTE_DATA: &'static str = "./examples/bench/routes.txt";
const SHAPE_DATA: &'static str = "./examples/bench/shapes.txt";
const STOP_DATA: &'static str = "./examples/bench/stops.txt";
const STOP_TIMES_DATA: &'static str = "./examples/bench/stop_times.txt";
const TRIP_DATA: &'static str = "./examples/bench/trips.txt";
const FREQUENCY_DATA: &'static str = "./examples/bench/frequencies.txt";

fn or_die<T, E: Debug+Display>(r: Result<T, E>) -> T {
    r.or_else(|e: E| -> Result<T, E> { panic!(format!("{:?}", e)) }).unwrap()
}

fn file_to_mem(fp: &str) -> Vec<u8> {
    let mut f = or_die(fs::File::open(fp));
    let mut bs = vec![];
    or_die(f.read_to_end(&mut bs));
    bs
}

#[bench]
fn bench_agency_iterator(b: &mut Bencher) {
    let data = file_to_mem(AGENCY_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "agency.txt".to_string(), agencies::parse_row).unwrap();
        for agency in iterator {
            let _ = agency;
        }
    })
}

#[bench]
fn bench_calendar_iterator(b: &mut Bencher) {
    let data = file_to_mem(CALENDAR_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "calendar.txt".to_string(), calendars::parse_row).unwrap();
        for calendar in iterator {
            let _ = calendar;
        }
    })
}

#[bench]
fn bench_calendar_date_iterator(b: &mut Bencher) {
    let data = file_to_mem(CALENDAR_DATE_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "calendar_dates.txt".to_string(), calendar_dates::parse_row).unwrap();
        for calendar_date in iterator {
            let _ = calendar_date;
        }
    })
}

#[bench]
fn bench_frequency_iterator(b: &mut Bencher) {
    let data = file_to_mem(FREQUENCY_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "frequencies.txt".to_string(), frequencies::parse_row).unwrap();
        for freq in iterator {
            let _ = freq;
        }
    })
}

#[bench]
fn bench_route_iterator(b: &mut Bencher) {
    let data = file_to_mem(ROUTE_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "routes.txt".to_string(), routes::parse_row).unwrap();
        for route in iterator {
            let _ = route;
        }
    })
}

#[bench]
fn bench_shape_iterator(b: &mut Bencher) {
    let data = file_to_mem(SHAPE_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "shapes.txt".to_string(), shapes::parse_row).unwrap();
        for shape in iterator {
            let _ = shape;
        }
    })
}

#[bench]
fn bench_stop_iterator(b: &mut Bencher) {
    let data = file_to_mem(STOP_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "stops.txt".to_string(), stops::parse_row).unwrap();
        for stop in iterator {
            let _ = stop;
        }
    })
}

#[bench]
fn bench_stop_time_iterator(b: &mut Bencher) {
    let data = file_to_mem(STOP_TIMES_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "stop_times.txt".to_string(), stop_times::parse_row).unwrap();
        for stop_time in iterator {
            let _ = stop_time;
        }
    })
}

#[bench]
fn bench_trip_iterator(b: &mut Bencher) {
    let data = file_to_mem(TRIP_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let csv = Csv::from_reader(&*data);
        let iterator = GTFSIterator::new(csv, "trips.txt".to_string(), trips::parse_row).unwrap();
        for trip in iterator {
            let _ = trip;
        }
    })
}
