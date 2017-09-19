use std::collections::HashMap;
use std::vec::IntoIter;
use std::iter::Iterator;
use transit::StopTime;
use gtfs::Error;

#[derive(Debug, PartialEq)]
pub struct Run
{
    pub trip: String,
    // TODO: StopTime is very verbose
    // redundant info removed from StopTime (trip id, sequence) (maybe make a trait?)
    pub sequence: Vec<StopTime>
}

pub struct RunIterator
{
    runs: IntoIter<Run>
}

impl RunIterator
{
    // TODO: return value isn't generic forcing users to specify types, very ugly
    /// Creates a RunIterator from an Iterator of StopTimes
    ///
    /// The StopTimes Iterator will be consumed so that they can be grouped and sorted
    pub fn new<U: Iterator<Item=Result<StopTime, Error>>>(stop_times: U) -> RunIterator {
        let mut run_groups = HashMap::<String, Run>::new();
        // group StopTimes by trip
        for stop_time in stop_times.filter(|r: &Result<StopTime, Error>| r.is_ok()).map(|r| r.unwrap()) {
            let run = run_groups.entry(stop_time.trip_id.clone()).or_insert(Run { trip: stop_time.trip_id.clone(), sequence: vec!() });
            run.sequence.push(stop_time);
        }

        // sort StopTimes within each Run
        for run in run_groups.values_mut() {
            run.sequence.sort_by(|a, b| a.stop_sequence.cmp(&b.stop_sequence));
        }

        let runs = run_groups.drain().map(|e| e.1).collect::<Vec<Run>>();
        RunIterator {
            runs: runs.into_iter(),
        }
    }

    // TODO: Write a version that trusts that the stop_times iterator is already sorted
    //       And panics if it does not (keep a list of completed trip_ids)
}

impl Iterator for RunIterator
{
    type Item = Run;
    fn next(&mut self) -> Option<Run> {
        self.runs.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use transit::{TimeOffset, PickupType, DropoffType, Timepoint};

    #[test]
    fn test_form_runs_from_unsorted_sequences() {
        let times = vec!(Result::Ok(stop_time("A", 3, None, None)),
                         Result::Ok(stop_time("A", 2, None, None)),
                         Result::Ok(stop_time("B", 1, None, None)),
                         Result::Ok(stop_time("B", 2, None, None)),
                         Result::Ok(stop_time("A", 1, None, None)),
                         Result::Ok(stop_time("B", 3, None, None)));
        let runs = RunIterator::new(times.into_iter());
        let mut result = runs.collect::<Vec<Run>>();
        result.sort_by(|a, b| a.sequence[0].trip_id.cmp(&b.sequence[0].trip_id));

        let expected = vec!(Run { trip: String::from("A"),
                                  sequence: vec!(stop_time("A", 1, None, None),
                                                 stop_time("A", 2, None, None),
                                                 stop_time("A", 3, None, None)) },
                            Run { trip: String::from("B"),
                                  sequence: vec!(stop_time("B", 1, None, None),
                                                 stop_time("B", 2, None, None),
                                                 stop_time("B", 3, None, None)) });
        assert_eq!(expected, result);
    }

    fn stop_time(trip: &str, sequence: u64, arrival: Option<[u32; 3]>, departure: Option<[u32; 3]>) -> StopTime {
        return StopTime {
            trip_id: String::from(trip),
            departure_time: match departure {
                None => TimeOffset::from_hms(0, 0, 0),
                Some(hms) => TimeOffset::from_hms(hms[0], hms[1], hms[2])
            },
            arrival_time: match arrival {
                None => TimeOffset::from_hms(0, 0, 0),
                Some(hms) => TimeOffset::from_hms(hms[0], hms[1], hms[2])
            },
            stop_id: format!("{}.{}", trip, sequence),
            stop_sequence: sequence,
            stop_headsign: None,
            pickup_type: PickupType::RegularlyScheduled,
            dropoff_type: DropoffType::RegularlyScheduled,
            timepoint: Timepoint::Exact,
            shape_dist_traveled: None,
        }
    }
}
