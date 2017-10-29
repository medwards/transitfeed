use serde;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use zip::ZipArchive;

use transit::{Agency, Stop, Route, Trip, StopTime, Calendar, CalendarDate, FareAttribute, FareRule, Shape, Frequency, Transfer, FeedInfo};
use gtfs::GTFSIterator;
use gtfs::Error;

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
    pub shapes: Option<Vec<Shape>>,
    pub frequencies: Option<Vec<Frequency>>,
    pub transfers: Option<Vec<Transfer>>,
    pub feedinfo: Option<FeedInfo>,

    stop_map: HashMap<String, usize>,
    route_map: HashMap<String, usize>,
    trip_map: HashMap<String, usize>,
}

impl TransitFeed {
    pub fn from_zip(zipfile: &str, output: &str) -> Result<Self, Error> {
        let output_path = Path::new(output);
        let mut zip = zip::ZipArchive::new(File::open(zipfile).unwrap()).unwrap();
        extract_zip(&mut zip, output_path);
        Self::from_path(output)
    }

    pub fn from_path(folder: &str) -> Result<Self, Error> {
        let agencies = try!(load_feed_file(folder, "agency.txt"));
        let stops = try!(load_feed_file(folder, "stops.txt"));
        let routes = try!(load_feed_file(folder, "routes.txt"));
        let trips = try!(load_feed_file(folder, "trips.txt"));
        let stoptimes = try!(load_feed_file(folder, "stop_times.txt"));
        let calendars = try!(load_feed_file(folder, "calendar.txt"));

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
            calendar_dates: match load_feed_file(folder, "calendar_dates.txt") {
                Ok(records) => Some(records),
                Err(e) => { println!("SKIPPING optional file - {}", e); None }
            },
            fare_attributes: match load_feed_file(folder, "fare_attributes.txt") {
                Ok(records) => Some(records),
                Err(e) => { println!("SKIPPING optional file - {}", e); None }
            },
            fare_rules: match load_feed_file(folder, "fare_rules.txt") {
                Ok(records) => Some(records),
                Err(e) => { println!("SKIPPING optional file - {}", e); None }
            },
            shapes: match load_feed_file(folder, "shapes.txt") {
                Ok(records) => Some(records),
                Err(e) => { println!("SKIPPING optional file - {}", e); None }
            },
            frequencies: match load_feed_file(folder, "frequencies.txt") {
                Ok(records) => Some(records),
                Err(e) => { println!("SKIPPING optional file - {}", e); None}
            },
            transfers: match load_feed_file(folder, "calendar_dates.txt") {
                Ok(records) => Some(records),
                Err(e) => { println!("SKIPPING optional file - {}", e); None }
            },
            feedinfo: match load_feed_file(folder, "feed_info.txt") {
                Ok(mut records) => {
                    if records.len() != 1 {
                        println!("Unexpected number of entries in feed_info.txt");
                    }
                    records.pop()
                },
                Err(e) => { println!("SKIPPING optional file - {}", e); None}
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

    fn find_record<'a, T>(record_id: &str, map: &HashMap<String, usize>, records: &'a Vec<T>) -> Option<&'a T> {
        map.get(record_id).map(|index| &records[*index])
    }
}

// TODO: Need to log stuff here
fn load_feed_file<T>(folder: &str, file: &str) -> Result<Vec<T>, Error>
    where for<'de> T: serde::Deserialize<'de>
{
    let iter = GTFSIterator::from_path(Path::new(folder).join(file).to_str().unwrap())?;
    Ok(iter.filter_map(|r| match r {
        Ok(r) => Some(r),
        Err(e) => { println!("SKIPPING - {}", e); None }
    }).collect())
}

fn make_map<T, F: (Fn(&T) -> String)>(records: &Vec<T>, key_fn: F) -> HashMap<String, usize> {
    records.iter().enumerate()
        .map(|(index, record)| (key_fn(record), index))
        .collect()
}

// move this elsewhere
use std;
use std::io;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use zip;

fn extract_zip<T: io::Read + io::Seek>(archive: &mut ZipArchive<T>, output: &Path) {
    for i in 0..archive.len()
    {
        let mut file = archive.by_index(i).unwrap();
        let outpath = output.join(sanitize_filename(file.name()));
        println!("{}", outpath.display());

        {
            let comment = file.comment();
            if comment.len() > 0 { println!("  File comment: {}", comment); }
        }

		// shouldn't need this for GTFS data?
        //create_directory(outpath.parent().unwrap_or(std::path::Path::new("")), None);

        let perms = convert_permissions(file.unix_mode());

        // also suspicious but why not?
        if (&*file.name()).ends_with("/") {
            create_directory(&outpath, perms);

        }
        else {
            write_file(&mut file, &outpath, perms);
        }
    }
}

#[cfg(unix)]
fn convert_permissions(mode: Option<u32>) -> Option<fs::Permissions>
{
    match mode {
        Some(mode) => Some(fs::Permissions::from_mode(mode)),
        None => None,
    }
}
#[cfg(not(unix))]
fn convert_permissions(_mode: Option<u32>) -> Option<fs::Permissions>
{
    None
}

fn write_file(file: &mut zip::read::ZipFile, outpath: &std::path::Path, perms: Option<fs::Permissions>)
{
    let mut outfile = fs::File::create(&outpath).unwrap();
    io::copy(file, &mut outfile).unwrap();
    if let Some(perms) = perms {
        fs::set_permissions(outpath, perms).unwrap();
    }
}

fn create_directory(outpath: &std::path::Path, perms: Option<fs::Permissions>)
{
    fs::create_dir_all(&outpath).unwrap();
    if let Some(perms) = perms {
        fs::set_permissions(outpath, perms).unwrap();
    }
}

fn sanitize_filename(filename: &str) -> std::path::PathBuf
{
    let no_null_filename = match filename.find('\0') {
        Some(index) => &filename[0..index],
        None => filename,
    };

    std::path::Path::new(no_null_filename)
        .components()
        .filter(|component| match *component {
            std::path::Component::Normal(..) => true,
            _ => false
        })
        .fold(std::path::PathBuf::new(), |mut path, ref cur| {
            path.push(cur.as_os_str());
            path
        })
}
