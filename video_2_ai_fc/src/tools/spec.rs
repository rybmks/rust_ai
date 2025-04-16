use crate::{Result, utils::XValue};
use schemars::{JsonSchema, schema_for};
use serde_json::{Value, json};

#[derive(Debug)]
pub struct ToolSpec {
    pub fn_name: String,
    pub fn_description: String,
    pub params: Value,
}

pub fn tool_spec<T: JsonSchema>() -> Result<ToolSpec> {
    let root_schema = schema_for!(T);
    let mut json_schema = serde_json::to_value(root_schema)?;

    let fn_name = json_schema.x_take("title")?;
    let fn_description = json_schema.x_take("description")?;
    let params = into_spec_params(json_schema)?;

    let tool_spec = ToolSpec {
        fn_name,
        fn_description,
        params,
    };

    Ok(tool_spec)
}

fn into_spec_params(mut json_schema: Value) -> Result<Value> {
    let required: Value = json_schema.x_take("required")?;
    let mut properties: Value = json_schema.x_take("properties")?;

    let json_schema = json_schema;

    let properties_obj = properties
        .as_object_mut()
        .ok_or("Properties is not object")?;

    for (_name, prop_value) in properties_obj {
        if let Some(Value::String(ref_def)) = prop_value.pointer_mut("/allOf/0/$ref") {
            let ref_def = ref_def.trim_start_matches('#');

            if let Some(Value::Object(refed_obj)) = json_schema.pointer(ref_def) {
                if let Some(prop_obj) = prop_value.as_object_mut() {
                    for (sub_name, sub_val) in refed_obj {
                        prop_obj.insert(sub_name.to_string(), sub_val.clone());
                    }

                    prop_obj.remove("allOf");
                }
            }
        }
    }

    Ok(json!({
        "type": "object",
        "properties": properties,
        "required": required
    }))
}
