use std::collections::HashMap;
use std::iter::Iterator;
use std::vec::IntoIter;
use transit::StopTime;

#[derive(Debug, PartialEq)]
pub struct Run<'a> {
    pub trip: String,
    // TODO: StopTime is very verbose
    // redundant info removed from StopTime (trip id, sequence) (maybe make a trait?)
    pub sequence: Vec<&'a StopTime>,
}

pub enum Runs<'a, I: Iterator<Item = &'a StopTime>> {
    Unsorted(IntoIter<Run<'a>>),
    Sorted(std::iter::Peekable<I>),
}

impl<'a, I: Iterator<Item = &'a StopTime>> Runs<'a, I> {
    pub fn from_unsorted(stop_times: I) -> Self {
        let mut run_groups = HashMap::<String, Run>::new();
        // group StopTime by trip
        for stop_time in stop_times {
            let run = run_groups.entry(stop_time.trip_id.clone()).or_insert(Run {
                trip: stop_time.trip_id.clone(),
                sequence: vec![],
            });
            run.sequence.push(stop_time);
        }

        // sort StopTimes within each Run
        for run in run_groups.values_mut() {
            run.sequence
                .sort_by(|a, b| a.stop_sequence.cmp(&b.stop_sequence));
        }

        let runs: Vec<_> = run_groups.drain().map(|e| e.1).collect();
        Self::Unsorted(runs.into_iter())
    }

    pub fn from_sorted(stop_times: I) -> Self {
        Self::Sorted(stop_times.peekable())
    }
}

impl<'a, I: Iterator<Item = &'a StopTime>> Iterator for Runs<'a, I> {
    type Item = Run<'a>;
    fn next(&mut self) -> Option<Run<'a>> {
        match self {
            Self::Unsorted(iter) => iter.next(),
            Self::Sorted(iter) => {
                let mut sequence: Vec<&'a StopTime> = Vec::new();
                loop {
                    let preview_value = iter.peek();
                    let current_trip_id = sequence.last().map_or(None, |s| Some(&s.trip_id));
                    match preview_value {
                        // Add to the sequence if trip_ids match
                        // Otherwise return the run
                        Some(value) => {
                            if &value.trip_id == current_trip_id.unwrap_or(&value.trip_id) {
                                sequence.push(iter.next().unwrap());
                            } else {
                                return Some(Run {
                                    trip: current_trip_id.unwrap().clone(),
                                    sequence: sequence,
                                });
                            }
                        }
                        // check if sequence is empty, if so return None
                        // otherwise return the run
                        None => {
                            if sequence.is_empty() {
                                let _ = iter.next();
                                return None;
                            } else {
                                return Some(Run {
                                    trip: current_trip_id.unwrap().clone(),
                                    sequence: sequence,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use transit::{StopServiceType, TimeOffset, Timepoint};

    #[test]
    fn test_form_runs_from_sorted_sequences() {
        let times = vec![
            stop_time("A", 1, None, None),
            stop_time("A", 2, None, None),
            stop_time("A", 3, None, None),
            stop_time("B", 10, None, None),
            stop_time("B", 20, None, None),
            stop_time("B", 30, None, None),
        ];
        let mut runs = Runs::from_sorted(times.iter());
        assert_eq!(
            Some(Run {
                trip: String::from("A"),
                sequence: vec![&times[0], &times[1], &times[2]],
            }),
            runs.next()
        );
        assert_eq!(
            Some(Run {
                trip: String::from("B"),
                sequence: vec![&times[3], &times[4], &times[5]],
            }),
            runs.next()
        );
        assert_eq!(None, runs.next());
    }

    #[test]
    fn test_form_bad_runs_from_unsorted_sequences() {
        let times = vec![
            stop_time("A", 1, None, None),
            stop_time("A", 2, None, None),
            stop_time("B", 10, None, None),
            stop_time("B", 20, None, None),
            stop_time("A", 3, None, None),
            stop_time("B", 30, None, None),
        ];
        let mut runs = Runs::from_unsorted(times.iter());
        assert_eq!(
            Some(Run {
                trip: String::from("A"),
                sequence: vec![&times[0], &times[1]],
            }),
            runs.next()
        );
        assert_eq!(
            Some(Run {
                trip: String::from("B"),
                sequence: vec![&times[2], &times[3]],
            }),
            runs.next()
        );
        assert_eq!(
            Some(Run {
                trip: String::from("A"),
                sequence: vec![&times[4]],
            }),
            runs.next()
        );
        assert_eq!(
            Some(Run {
                trip: String::from("B"),
                sequence: vec![&times[5]],
            }),
            runs.next()
        );
        assert_eq!(None, runs.next());
    }

    #[test]
    fn test_form_single_run() {
        let times = vec![stop_time("A", 1, None, None)];
        let mut runs = Runs::from_sorted(times.iter());
        assert_eq!(
            Some(Run {
                trip: String::from("A"),
                sequence: vec![&stop_time("A", 1, None, None)],
            }),
            runs.next()
        );
        assert_eq!(None, runs.next());
    }

    #[test]
    fn test_no_runs_from_empty_iter() {
        let mut runs = Runs::from_sorted(std::iter::empty());
        assert_eq!(None, runs.next());
        assert_eq!(None, runs.next());
    }

    #[test]
    fn test_form_runs_from_unsorted_sequences() {
        let times = vec![
            stop_time("A", 3, None, None),
            stop_time("A", 2, None, None),
            stop_time("B", 1, None, None),
            stop_time("B", 2, None, None),
            stop_time("A", 1, None, None),
            stop_time("B", 3, None, None),
        ];
        let runs = Runs::from_unsorted(times.iter());
        let mut result = runs.collect::<Vec<Run>>();
        result.sort_by(|a, b| a.sequence[0].trip_id.cmp(&b.sequence[0].trip_id));

        let expected = vec![
            Run {
                trip: String::from("A"),
                sequence: vec![&times[4], &times[1], &times[0]],
            },
            Run {
                trip: String::from("B"),
                sequence: vec![&times[2], &times[3], &times[5]],
            },
        ];
        assert_eq!(expected, result);
    }

    fn stop_time(
        trip: &str,
        sequence: u64,
        arrival: Option<[u32; 3]>,
        departure: Option<[u32; 3]>,
    ) -> StopTime {
        return StopTime {
            trip_id: String::from(trip),
            departure_time: match departure {
                None => TimeOffset::from_hms(0, 0, 0),
                Some(hms) => TimeOffset::from_hms(hms[0], hms[1], hms[2]),
            },
            arrival_time: match arrival {
                None => TimeOffset::from_hms(0, 0, 0),
                Some(hms) => TimeOffset::from_hms(hms[0], hms[1], hms[2]),
            },
            stop_id: format!("{}.{}", trip, sequence),
            stop_sequence: sequence,
            stop_headsign: None,
            pickup_type: StopServiceType::RegularlyScheduled,
            dropoff_type: StopServiceType::RegularlyScheduled,
            timepoint: Timepoint::Exact,
            shape_dist_traveled: None,
        };
    }
}
