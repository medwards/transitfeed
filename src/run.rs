use std::collections::HashMap;
use std::vec::IntoIter;
use std::iter::Iterator;
use transit::StopTime;

#[derive(Debug, PartialEq)]
pub struct Run<'a> {
    pub trip: String,
    // TODO: StopTime is very verbose
    // redundant info removed from StopTime (trip id, sequence) (maybe make a trait?)
    pub sequence: Vec<&'a StopTime>,
}

pub struct RunIterator<'a> {
    runs: IntoIter<Run<'a>>,
}

impl<'a> RunIterator<'a> {
    // TODO: return value isn't generic forcing users to specify types, very ugly
    /// Creates a RunIterator from an Iterator of StopTimes
    ///
    /// The StopTimes Iterator will be consumed so that they can be grouped and sorted
    pub fn new<U: Iterator<Item = &'a StopTime>>(stop_times: U) -> RunIterator<'a> {
        let mut run_groups = HashMap::<String, Run>::new();
        // group StopTimes by trip
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

        let runs = run_groups.drain().map(|e| e.1).collect::<Vec<Run>>();
        RunIterator {
            runs: runs.into_iter(),
        }
    }

    // TODO: Write a version that trusts that the stop_times iterator is already sorted
    //       And panics if it does not (keep a list of completed trip_ids)
}

impl<'a> Iterator for RunIterator<'a> {
    type Item = Run<'a>;
    fn next(&mut self) -> Option<Run<'a>> {
        self.runs.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use transit::{StopServiceType, TimeOffset, Timepoint};

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
        let runs = RunIterator::new(times.iter());
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
