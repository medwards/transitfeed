extern crate transitfeed;
use transitfeed::run::*;
use transitfeed::{GTFSIterator, StopTime};

#[test]
fn test_read_sorted_runs() {
    let iter = GTFSIterator::from_path("./examples/good_feed/stop_times.txt")
        .unwrap()
        .filter_map(|s: Result<StopTime, _>| s.ok());
    let stop_times: Vec<_> = iter.collect();
    let mut runs = Runs::from_sorted(stop_times.iter());
    let run = runs.next().expect("no run was found");
    assert_eq!("STBA".to_string(), run.trip);
    // just check run length for now since stoptime is big
    assert_eq!(2, run.sequence.len());
    assert_eq!("CITY1", &runs.next().expect("no run was found").trip);
    assert_eq!("CITY2", &runs.next().expect("no run was found").trip);
    assert_eq!("AB1", &runs.next().expect("no run was found").trip);
    assert_eq!("AB2", &runs.next().expect("no run was found").trip);
    assert_eq!("BFC1", &runs.next().expect("no run was found").trip);
    assert_eq!("BFC2", &runs.next().expect("no run was found").trip);
    assert_eq!("AAMV1", &runs.next().expect("no run was found").trip);
    assert_eq!("AAMV2", &runs.next().expect("no run was found").trip);
    assert_eq!("AAMV3", &runs.next().expect("no run was found").trip);
    assert_eq!("AAMV4", &runs.next().expect("no run was found").trip);
    assert_eq!(None, runs.next());
}

#[test]
fn test_read_unsorted_runs() {
    let iter = GTFSIterator::from_path("./examples/good_feed/stop_times.txt")
        .unwrap()
        .filter_map(|s: Result<StopTime, _>| s.ok());
    let stop_times: Vec<_> = iter.collect();
    let runs = Runs::from_unsorted(stop_times.iter());
    // sorting results in indeterminate run order
    let trips: Vec<_> = runs.map(|r| r.trip).collect();
    assert!(trips.as_slice().contains(&"STBA".to_string()));
    /*
    assert!(trips.as_slice().contains("CITY1"));
    assert!(trips.as_slice().contains("CITY2"));
    assert!(trips.as_slice().contains("AB1"));
    assert!(trips.as_slice().contains("AB2"));
    assert!(trips.as_slice().contains("BFC1"));
    assert!(trips.as_slice().contains("BFC2"));
    assert!(trips.as_slice().contains("AAMV1"));
    assert!(trips.as_slice().contains("AAMV2"));
    assert!(trips.as_slice().contains("AAMV3"));
    assert!(trips.as_slice().contains("AAMV4"));
    */
}
