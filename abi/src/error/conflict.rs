use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashMap;
use std::{convert::Infallible, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(conflict) = s.parse() {
            Ok(ReservationConflictInfo::Parsed(conflict))
        } else {
            Ok(ReservationConflictInfo::Unparsed(s.to_string()))
        }
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedInfo::from_str(s)?.try_into()
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();

    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            old: value.old.try_into()?,
        })
    }
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");
        let mut split = timespan_str.splitn(2, ',');
        let start = parse_datetime(split.next().ok_or(())?)?;
        let end = parse_datetime(split.next().ok_or(())?)?;
        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(
            r#"\((?P<k1>[a-zA-Z0-9_-]+)\s*,\s*(?P<k2>[a-zA-Z0-9_-]+)\)=\((?P<v1>[a-zA-Z0-9_-]+)\s*,\s*\[(?P<v2>[^\)\]]+)"#,
        )
        .unwrap();
        let mut maps = vec![];
        for cap in re.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(cap["k1"].to_string(), cap["v1"].to_string());
            map.insert(cap["k2"].to_string(), cap["v2"].to_string());
            maps.push(Some(map))
        }
        if maps.len() != 2 {
            return Err(());
        }

        Ok(ParsedInfo {
            new: maps[0].take().unwrap(),
            old: maps[1].take().unwrap(),
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ReservationConflictInfo;

    use super::{ParsedInfo, ReservationWindow};

    // TODO
    const ERR_MSG:&str = "键(resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"))与已存在的键(resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\"))冲突";
    #[test]
    fn parser_should_work() {
        let parsered = ParsedInfo::from_str(ERR_MSG);
        if let Ok(info) = parsered {
            assert_eq!(
                info.old.get("timespan").unwrap(),
                "\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\""
            );
        } else {
            panic!("parser not work");
        }
    }

    #[test]
    fn parser_info_to_reservation_window_should_work() {
        let parsered = ParsedInfo::from_str(ERR_MSG);
        if let Ok(info) = parsered {
            if let Ok(info) = ReservationWindow::try_from(info.old) {
                assert_eq!(info.start.to_rfc3339(), "2022-12-25T22:00:00+00:00");
            } else {
                panic!("parser_info_to_reservation_window not work");
            }
        } else {
            panic!("parser not work");
        }
    }

    #[test]
    fn parser_to_conflict_should_work() {
        let info: Result<ReservationConflictInfo, _> = ERR_MSG.parse();
        if let Ok(ReservationConflictInfo::Parsed(info)) = info {
            assert_eq!(info.old.start.to_rfc3339(), "2022-12-25T22:00:00+00:00")
        }
    }
}
