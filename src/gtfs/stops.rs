use transit::{Stop, WheelchairBoarding, LocationType};
use std::collections::HashMap;
use std::iter::Zip;
use std::slice::Iter;
use quick_csv::columns::Columns;
use gtfs::error::ParseError;

use gtfs::parse::{parse_float, parse_location_type, parse_wheelchair_boarding};

pub fn parse_row(row: Zip<Iter<String>, Columns>) -> Result<Stop, ParseError>
{
    let mut stop_id = String::new();
    let mut stop_code = None;
    let mut stop_name = String::new();
    let mut stop_desc = None;
    let mut stop_lat = 0.0;
    let mut stop_lon = 0.0;
    let mut zone_id = None;
    let mut stop_url = None;
    let mut location_type = LocationType::Stop;
    let mut parent_station = None;
    let mut stop_timezone = None;
    let mut wheelchair_boarding = WheelchairBoarding::NoInformation;
    let mut extended_fields = None;

    for (header, column) in row {
        match &header[..] {
            "stop_id" => { stop_id = String::from(column); },
            "stop_code" => { stop_code = Some(String::from(column)); },
            "stop_name" => { stop_name = String::from(column); },
            "stop_desc" => { stop_desc = Some(String::from(column)); },
            "stop_lat" => { stop_lat = parse_try!(parse_float(column)); },
            "stop_lon" => { stop_lon = parse_try!(parse_float(column)); },
            "zone_id" => { zone_id = Some(String::from(column)); },
            "stop_url" => { stop_url = Some(String::from(column)); },
            "location_type" => { location_type = parse_try!(parse_location_type(column)); },
            "parent_station" => { parent_station = Some(String::from(column)); },
            "stop_timezone" => { stop_timezone = Some(String::from(column)); },
            "wheelchair_boarding" => { wheelchair_boarding = parse_try!(parse_wheelchair_boarding(column)); },
            field => {
                extended_fields = Some(extended_fields.map_or(HashMap::new(), |m| m));
                extended_fields.as_mut().unwrap().insert(String::from(field), String::from(column));
            },
        }
    }
    Ok(Stop {
        stop_id: stop_id,
        stop_code: stop_code,
        stop_name: stop_name,
        stop_desc: stop_desc,
        stop_lat: stop_lat,
        stop_lon: stop_lon,
        zone_id: zone_id,
        stop_url: stop_url,
        location_type: location_type,
        parent_station: parent_station,
        stop_timezone: stop_timezone,
        wheelchair_boarding: wheelchair_boarding,
        extended_fields: extended_fields,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::Map;

    #[test]
    fn test_parses_basic_row() {
        let headers = vec!("stop_id", "stop_name", "stop_lat", "stop_lon")
            .iter().map(|x| x.to_string())
            .collect::<Vec<String>>();
        let counts = vec!(1 as usize, 5 as usize, 7 as usize, 9 as usize);
        let columns = Columns::new("1,foo,1,1", &counts);
        let expected = Stop {
            stop_id: "1".to_string(),
            stop_name: "foo".to_string(),
            stop_lat: 1.0,
            stop_lon: 1.0,
            stop_code: None,
            stop_desc: None,
            zone_id: None,
            stop_url: None,
            location_type: LocationType::Stop,
            parent_station: None,
            stop_timezone: None,
            wheelchair_boarding: WheelchairBoarding::NoInformation,
            extended_fields: None,
        };
        let result = parse_row(headers.as_slice().iter().zip(columns));
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_parses_extended_columns() {
        let headers = vec!("stop_id", "stop_name", "stop_lat", "stop_lon", "platform_code")
            .iter().map(|x| x.to_string())
            .collect::<Vec<String>>();
        let counts = vec!(1 as usize, 5 as usize, 7 as usize, 9 as usize, 11 as usize);
        let columns = Columns::new("1,foo,1,1,A", &counts);
        let mut expected_extended = HashMap::new();
        expected_extended.insert(String::from("platform_code"), String::from("A"));
        let expected = Stop {
            stop_id: "1".to_string(),
            stop_name: "foo".to_string(),
            stop_lat: 1.0,
            stop_lon: 1.0,
            stop_code: None,
            stop_desc: None,
            zone_id: None,
            stop_url: None,
            location_type: LocationType::Stop,
            parent_station: None,
            stop_timezone: None,
            wheelchair_boarding: WheelchairBoarding::NoInformation,
            extended_fields: Some(expected_extended),
        };
        let result = parse_row(headers.as_slice().iter().zip(columns));
        assert_eq!(expected, result.unwrap());
    }
}
