pub(crate) mod chrono_string_seconds {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize, Serializer, de};

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let value = <&str>::deserialize(deserializer)?;
        let format = if value.contains('.') { "%s.%f" } else { "%s" };

        DateTime::parse_from_str(value, format)
            .map_err(de::Error::custom)
            .map(|dt| dt.with_timezone(&Utc))
    }

    pub(crate) fn serialize<S>(date_time: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted = date_time.format("%s").to_string();
        formatted.serialize(serializer)
    }
}

pub(crate) mod chrono_iso8601 {
    use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
    use serde::{Deserialize, Serialize, Serializer, de};

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let value = <&str>::deserialize(deserializer)?;
        if let Ok(dt_fixed) = DateTime::parse_from_rfc3339(value) {
            return Ok(dt_fixed.with_timezone(&Utc));
        }

        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            let dt = date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(Utc.from_utc_datetime(&dt));
        }

        if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
            return Ok(Utc.from_utc_datetime(&dt));
        }

        Err(de::Error::custom(format!(
            "failed to parse datetime: {value:?}"
        )))
    }

    pub(crate) fn serialize<S>(date_time: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        date_time.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct DateTimeStruct {
        #[serde(with = "super::chrono_string_seconds")]
        timestamp: DateTime<Utc>,
    }

    #[test]
    fn chrono_string_seconds_valid() {
        let json = r#"{"timestamp": "1695024538"}"#;
        let result = serde_json::from_str::<DateTimeStruct>(json);
        assert_eq!(result.unwrap().timestamp.timestamp(), 1_695_024_538);

        let json = r#"{"timestamp": "1695024538.111111"}"#;
        let result = serde_json::from_str::<DateTimeStruct>(json);
        assert_eq!(result.unwrap().timestamp.timestamp(), 1_695_024_538);
    }

    #[test]
    fn chrono_string_seconds_invalid() {
        let json = r#"{"timestamp": 1695024538}"#;
        let result = serde_json::from_str::<DateTimeStruct>(json);
        assert!(result.is_err());

        let json = r#"{"timestamp": "timestamp"}"#;
        let result = serde_json::from_str::<DateTimeStruct>(json);
        assert!(result.is_err());
    }

    #[test]
    fn serialize() {
        let object = DateTimeStruct {
            timestamp: DateTime::from_timestamp(61, 0).unwrap().to_utc(),
        };
        let result = serde_json::to_string(&object).unwrap();
        let json = r#"{"timestamp":"61"}"#;
        assert_eq!(result, json);
    }
}
