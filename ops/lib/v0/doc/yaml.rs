use crate::v0::doc::OpsDoc;
use crate::v0::error::Error;
use crate::v0::format::Format;
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::task::Task;
use serde_yaml::Value;

pub struct Yaml;

/// YAML resolves `.inf`, `-.inf`, and `.nan` to non-finite floats, which JSON —
/// and so the model — cannot carry. The value is kept as the string YAML spells
/// it with, so a scalar the model has no shape for is not silently dropped when
/// the two are bridged.
fn spell_out_non_finite_numbers(value: &mut Value) {
    match value {
        Value::Number(number) if number.as_f64().is_some_and(|value| !value.is_finite()) => {
            let spelling = match number.as_f64() {
                Some(value) if value.is_nan() => ".nan",
                Some(value) if value.is_sign_positive() => ".inf",
                _ => "-.inf",
            };
            *value = Value::String(spelling.to_string());
        }
        Value::Sequence(items) => items.iter_mut().for_each(spell_out_non_finite_numbers),
        Value::Mapping(map) => map.values_mut().for_each(spell_out_non_finite_numbers),
        Value::Tagged(tagged) => spell_out_non_finite_numbers(&mut tagged.value),
        _ => {}
    }
}

impl OpsDoc for Yaml {
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error> {
        let mut value: Value =
            serde_yaml::from_str(input).map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?;
        spell_out_non_finite_numbers(&mut value);
        match value {
            Value::Sequence(items) => items
                .into_iter()
                .map(|item| {
                    let json = serde_json::to_value(item)
                        .map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?;
                    serde_json::from_value::<RawTask>(json)
                        .map_err(|e| Error::syntax(Format::Yaml, e.to_string()))
                })
                .collect(),
            other => {
                let json = serde_json::to_value(other)
                    .map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?;
                serde_json::from_value::<RawTask>(json)
                    .map(|raw| vec![raw])
                    .map_err(|e| Error::syntax(Format::Yaml, e.to_string()))
            }
        }
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        let value = if let [task] = tasks {
            serde_yaml::to_value(task).map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?
        } else {
            serde_yaml::to_value(tasks).map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?
        };
        serde_yaml::to_string(&value).map_err(|e| Error::syntax(Format::Yaml, e.to_string()))
    }
}
