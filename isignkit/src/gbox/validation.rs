use crate::error::GboxError;
use boon::{Compiler, Format, Schemas};
use chrono::DateTime;
use schemars::Schema;
use serde_json::Value;
use std::error::Error;

const RFC3339_CUSTOM_TAG: &str = "rfc3339-datetime";

pub(crate) fn validate_object(object: &Value, schema: Value) -> Result<(), GboxError> {
    let mut schemas = Schemas::new();

    let mut compiler = Compiler::new();
    compiler.register_format(Format {
        name: RFC3339_CUSTOM_TAG,
        func: validate_dt,
    });

    compiler.enable_format_assertions();
    compiler
        .add_resource("gbox", schema)
        .map_err(|err| GboxError::Validation(err.to_string()))?;

    let sch_index = compiler
        .compile("gbox", &mut schemas)
        .map_err(|err| GboxError::Validation(err.to_string()))?;

    schemas
        .validate(object, sch_index)
        .map_err(|err| GboxError::Validation(err.to_string()))?;

    Ok(())
}

pub(crate) fn schema_datetime_format(schema: &mut Schema) {
    if let Some(obj) = schema.as_object_mut() {
        obj.insert("format".to_string(), RFC3339_CUSTOM_TAG.into());
    }
}

fn validate_dt(v: &Value) -> Result<(), Box<dyn Error>> {
    let Value::String(s) = v else {
        return Ok(());
    };

    let fmt = "%Y-%m-%dT%H:%M:%S%#z";

    _ = DateTime::parse_from_str(s, fmt).map_err(|_| GboxError::InvalidDateTimeFormat {
        got: s.clone(),
        expected: fmt.to_string(),
    })?;

    Ok(())
}
