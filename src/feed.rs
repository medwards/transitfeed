extern crate tempfile;

use self::tempfile::{Builder, TempDir};
use csv;
use serde;
use std;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use zip;

use archive::extract_zip;
use gtfs::Error;
use gtfs::GTFSIterator;
use transit::{
    Agency, Calendar, CalendarDate, FareAttribute, FareRule, FeedInfo, Frequency, Route,
    ShapePoint, Stop, StopTime, Transfer, Trip,
};

pub use csv::{Terminator, Trim};

#[derive(Debug)]
pub struct FeedReader<P>
where
    P: FeedProvider,
{
    provider: P,
    builder: csv::ReaderBuilder,
}

pub trait FeedProvider {
    fn path(&self) -> &str;
}

pub struct LocalFeedProvider {
    path: String,
}

impl LocalFeedProvider {
    fn new(path: &str) -> LocalFeedProvider {
        LocalFeedProvider {
            path: path.to_string(),
        }
    }
}

impl FeedProvider for LocalFeedProvider {
    fn path(&self) -> &str {
        return &self.path;
    }
}

#[derive(Debug)]
pub struct ZipFeedProvider {
    dir: TempDir,
}

impl ZipFeedProvider {
    fn new(zipfile: &str) -> Result<ZipFeedProvider, Error> {
        let dir = Builder::new()
            .prefix("transitfeed")
            .tempdir()
            .map_err(|e| Error::Feed(format!("{}", e)))?;
        let mut zip =
            zip::ZipArchive::new(File::open(zipfile).map_err(|e| Error::Feed(format!("{}", e)))?)
                .map_err(|e| Error::Feed(format!("{}", e)))?;
        extract_zip(&mut zip, dir.path()).map_err(|e| Error::Feed(format!("{}", e)))?;
        Ok(ZipFeedProvider { dir: dir })
    }
}

impl FeedProvider for ZipFeedProvider {
    fn path(&self) -> &str {
        self.dir.path().to_str().unwrap()
    }
}

impl FeedReader<LocalFeedProvider> {
    pub fn new(path: &str) -> Self {
        FeedReader::from_provider(LocalFeedProvider::new(path))
    }
}

impl FeedReader<ZipFeedProvider> {
    pub fn from_zip(zipfile: &str) -> Result<Self, Error> {
        Ok(FeedReader::from_provider(ZipFeedProvider::new(zipfile)?))
    }
}

impl<P: FeedProvider> FeedReader<P> {
    pub fn from_provider(provider: P) -> Self {
        FeedReader {
            provider: provider,
            builder: csv::ReaderBuilder::new(),
        }
    }

    pub fn builder(&mut self) -> &mut csv::ReaderBuilder {
        &mut self.builder
    }

    pub fn agencies(&self) -> Result<GTFSIterator<File, Agency>, Error> {
        self.make_iterator("agency.txt")
    }

    pub fn stops(&self) -> Result<GTFSIterator<File, Stop>, Error> {
        self.make_iterator("stops.txt")
    }

    pub fn routes(&self) -> Result<GTFSIterator<File, Route>, Error> {
        self.make_iterator("routes.txt")
    }

    pub fn trips(&self) -> Result<GTFSIterator<File, Trip>, Error> {
        self.make_iterator("trips.txt")
    }

    pub fn stop_times(&self) -> Result<GTFSIterator<File, StopTime>, Error> {
        self.make_iterator("stop_times.txt")
    }

    pub fn calendars(&self) -> Result<GTFSIterator<File, Calendar>, Error> {
        self.make_iterator("calendar.txt")
    }

    pub fn calendar_dates(&self) -> Result<GTFSIterator<File, CalendarDate>, Error> {
        self.make_iterator("calendar_dates.txt")
    }

    pub fn fare_attributes(&self) -> Result<GTFSIterator<File, FareAttribute>, Error> {
        self.make_iterator("fare_attributes.txt")
    }

    pub fn fare_rules(&self) -> Result<GTFSIterator<File, FareRule>, Error> {
        self.make_iterator("fare_rules.txt")
    }

    pub fn shapes(&self) -> Result<GTFSIterator<File, ShapePoint>, Error> {
        self.make_iterator("shapes.txt")
    }

    pub fn frequencies(&self) -> Result<GTFSIterator<File, Frequency>, Error> {
        self.make_iterator("frequencies.txt")
    }

    pub fn transfers(&self) -> Result<GTFSIterator<File, Transfer>, Error> {
        self.make_iterator("transfers.txt")
    }

    pub fn feed_info(&self) -> Result<GTFSIterator<File, FeedInfo>, Error> {
        self.make_iterator("feed_info.txt")
    }

    fn make_iterator<T>(&self, filename: &str) -> Result<GTFSIterator<File, T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = match Path::new(&self.provider.path()).join(filename).to_str() {
            Some(path_str) => path_str.to_string(),
            None => {
                return Err(Error::Feed(format!(
                    "failed to construct path from {} and {}",
                    self.provider.path(),
                    filename
                )))
            }
        };
        let reader = match self.builder.from_path(&path) {
            Ok(reader) => reader,
            Err(e) => return Err(Error::Csv(path, e)),
        };
        Ok(GTFSIterator::new(reader, &path)?)
    }
}

/// Container for all transit records
pub struct TransitFeed {
    pub agencies: Vec<Agency>,
    pub stops: Vec<Stop>,
    pub routes: Vec<Route>,
    pub trips: Vec<Trip>,
    pub stoptimes: Vec<StopTime>,
    pub calendars: Vec<Calendar>,
    pub calendar_dates: Option<Vec<CalendarDate>>,
    pub fare_attributes: Option<Vec<FareAttribute>>,
    pub fare_rules: Option<Vec<FareRule>>,
    pub shapes: Option<Vec<ShapePoint>>,
    pub frequencies: Option<Vec<Frequency>>,
    pub transfers: Option<Vec<Transfer>>,
    pub feedinfo: Option<FeedInfo>,

    stop_map: HashMap<String, usize>,
    route_map: HashMap<String, usize>,
    trip_map: HashMap<String, usize>,
}

impl TransitFeed {
    pub fn from_reader<P: FeedProvider>(reader: &FeedReader<P>) -> Result<TransitFeed, Error> {
        let agencies = load_feed_file(try!(reader.agencies()));
        let stops = load_feed_file(try!(reader.stops()));
        let routes = load_feed_file(try!(reader.routes()));
        let trips = load_feed_file(try!(reader.trips()));
        let stoptimes = load_feed_file(try!(reader.stop_times()));
        let calendars = load_feed_file(try!(reader.calendars()));

        let stop_map = make_map(&stops, |stop: &Stop| stop.stop_id.clone());
        let route_map = make_map(&routes, |route: &Route| route.route_id.clone());
        let trip_map = make_map(&trips, |trip: &Trip| trip.trip_id.clone());

        Ok(TransitFeed {
            agencies: agencies,
            stops: stops,
            stop_map: stop_map,
            routes: routes,
            route_map: route_map,
            trips: trips,
            trip_map: trip_map,
            stoptimes: stoptimes,
            calendars: calendars,
            calendar_dates: load_optional_feed_file(reader.calendar_dates()),
            fare_attributes: load_optional_feed_file(reader.fare_attributes()),
            fare_rules: load_optional_feed_file(reader.fare_rules()),
            shapes: load_optional_feed_file(reader.shapes()),
            frequencies: load_optional_feed_file(reader.frequencies()),
            transfers: load_optional_feed_file(reader.transfers()),
            feedinfo: match load_optional_feed_file(reader.feed_info()) {
                Some(mut records) => {
                    if records.len() != 1 {
                        println!("Unexpected number of entries in feed_info.txt");
                    }
                    records.pop()
                }
                None => None,
            },
        })
    }

    pub fn find_stop(&self, id: &str) -> Option<&Stop> {
        TransitFeed::find_record(id, &self.stop_map, &self.stops)
    }

    pub fn find_route(&self, id: &str) -> Option<&Route> {
        TransitFeed::find_record(id, &self.route_map, &self.routes)
    }

    pub fn find_trip(&self, id: &str) -> Option<&Trip> {
        TransitFeed::find_record(id, &self.trip_map, &self.trips)
    }

    fn find_record<'a, T>(
        record_id: &str,
        map: &HashMap<String, usize>,
        records: &'a Vec<T>,
    ) -> Option<&'a T> {
        map.get(record_id).map(|index| &records[*index])
    }
}

// TODO: Need to log stuff here
fn load_feed_file<R, T>(iter: GTFSIterator<R, T>) -> Vec<T>
where
    R: std::io::Read,
    for<'de> T: serde::Deserialize<'de>,
{
    iter.filter_map(|r| match r {
        Ok(r) => Some(r),
        Err(e) => {
            println!("SKIPPING - {}", e);
            None
        }
    })
    .collect()
}

fn load_optional_feed_file<R, T>(result: Result<GTFSIterator<R, T>, Error>) -> Option<Vec<T>>
where
    R: std::io::Read,
    for<'de> T: serde::Deserialize<'de>,
{
    match result {
        Ok(iter) => Some(load_feed_file(iter)),
        Err(e) => {
            println!("SKIPPING optional file - {}", e);
            None
        }
    }
}

fn make_map<T, F: Fn(&T) -> String>(records: &Vec<T>, key_fn: F) -> HashMap<String, usize> {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| (key_fn(record), index))
        .collect()
}
