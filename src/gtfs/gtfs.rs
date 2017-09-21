use gtfs::error::{ParseError, GtfsError};
use std::io::BufRead;
use std::iter::Zip;
use std::slice::Iter;
use quick_csv::Csv;
use quick_csv::columns::Columns;

pub struct GTFSIterator<'a, B, F, T, U, V>
    where B: BufRead,
          F: (Fn(Zip<U, V>) -> Result<T, ParseError>),
          U: Iterator<Item=String>,
          V: Iterator<Item=&'a str>
{
    csv: Csv<B>,
    filename: String,
    header: Vec<String>,
    line: usize,
    parser: F,
}

impl<'a, B, F, T, Iter, Columns> GTFSIterator<'a, B, F, T, Iter, Columns>
    where B: BufRead,
          F: (Fn(Zip<Iter, Columns>) -> Result<T, ParseError>),
          Iter: Iterator<Item=String>,
          Columns: Iterator<Item=&'a str>,
{
    pub fn new(csv: Csv<B>, filename: String, parser: F) -> Result<GTFSIterator<'a, B, F, T, Iter, Columns>, GtfsError> {
        let mut csv = csv.has_header(true);
        let header = csv.headers();
        if header.len() == 0 {
            Err(GtfsError::CsvHeader(filename))
        } else {
            Ok(GTFSIterator {
                csv: csv,
                parser: parser,
                header: header,
                filename: filename,
                line: 1,
            })
        }
    }
}

impl<'a, B, F, T, U, V> Iterator for GTFSIterator<'a, B, F, T, U, V>
    where B: BufRead,
          F: (Fn(Zip<U, V>) -> Result<T, ParseError>),
          U: Iterator<Item=String>,
          V: Iterator<Item=&'a str>
{
    type Item = Result<T, GtfsError>;

    fn next(&mut self) -> Option<Result<T, GtfsError>> {
        self.line += 1;
        match self.csv.next() {
            None => None,
            Some(res) => match res {
                Ok(row) => match row.columns() {
                    Ok(columns) =>  {
                        let result = match (self.parser)(self.header.iter().zip(columns)) {
                            Ok(x) => Some(Ok(x)),
                            Err(y) => Some(Err(GtfsError::LineError(y, self.filename.clone(), self.line))),
                        };
                        result
                    },
                    Err(err) => Some(Err(GtfsError::Csv(err, self.filename.clone(), self.line))),
                },
                Err(err) => Some(Err(GtfsError::Csv(err, self.filename.clone(), self.line))),
            }
        }
    }
}

#[test]
fn test_returns_parsed_entry() {
    let csv = Csv::from_string("foo,bar,baz
                                ,,");
    let mut iterator = GTFSIterator::new(csv, "example.txt".to_string(), |_| Ok(())).unwrap();
    let entry = iterator.next().unwrap().unwrap();
    assert_eq!((), entry);
}

#[test]
fn test_wraps_parse_failures() {
    let csv = Csv::from_string("foo,bar,baz
                                ,,");
    let mut iterator = GTFSIterator::new(csv, "example.txt".to_string(), |_| -> Result<(), ParseError> { Err(ParseError::ParseFloat("".to_string())) }).unwrap();
    let entry = iterator.next().unwrap().err().unwrap();
    assert_eq!("GtfsError: error reading line (example.txt:2) - Some(ParseFloat(\"\"))", format!("{}", entry));
}

#[test]
fn test_wraps_row_failures() {
    // Use column mismatch
    let csv = Csv::from_string("foo,bar,baz
                                ,");
    let mut iterator = GTFSIterator::new(csv, "example.txt".to_string(), |_| Ok(())).unwrap();
    let entry = iterator.next().unwrap().err().unwrap();
    assert_eq!("GtfsError: Current column count mismatch with previous rows (example.txt:2) - Some(ColumnMismatch(3, 2))", format!("{}", entry));
}
