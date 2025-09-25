use anyhow::{Context, anyhow};
use boon::{Compiler, Format, Schemas};
use chrono::DateTime;
use schemars::Schema;
use serde_json::Value;
use std::error::Error;

pub(crate) fn validate_object(object: &Value, schema: Value) -> anyhow::Result<()> {
    let mut schemas = Schemas::new();

    let mut compiler = Compiler::new();
    compiler.register_format(Format {
        name: "rfc3339-datetime",
        func: validate_dt,
    });

    compiler.enable_format_assertions();
    compiler
        .add_resource("gbox", schema)
        .map_err(|err| anyhow!("{err:?}"))?;

    let sch_index = compiler
        .compile("gbox", &mut schemas)
        .map_err(|err| anyhow!("{err:?}"))?;

    schemas
        .validate(object, sch_index)
        .map_err(|err| anyhow!("{err}"))?;

    Ok(())
}

pub(crate) fn schema_datetime_format(schema: &mut Schema) {
    if let Some(obj) = schema.as_object_mut() {
        obj.insert("format".to_string(), "rfc3339-datetime".into());
    }
}

fn validate_dt(v: &Value) -> Result<(), Box<dyn Error>> {
    let Value::String(s) = v else {
        return Ok(());
    };

    let fmt = "%Y-%m-%dT%H:%M:%S%#z";

    _ = DateTime::parse_from_str(s, fmt)
        .with_context(|| format!("datetime expected to be in a format of: {fmt}"))?;

    Ok(())
}
