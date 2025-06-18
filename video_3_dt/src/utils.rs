use super::Result;
use polars::{
    frame::DataFrame,
    prelude::{Column, DataType},
};
use std::collections::HashMap;

pub fn label_encode(df: &DataFrame) -> Result<DataFrame> {
    let names = df.get_column_names();
    let mut df_encoded = DataFrame::empty();
    for name in names {
        let mut mapping = HashMap::new();
        let series = df.column(name)?;

        if let DataType::String = series.dtype() {
            let unique = series.unique()?;

            for (index, value) in unique.str()?.into_iter().enumerate() {
                mapping.insert(value, index as u32);
            }

            let c_encoded = df
                .column(name)?
                .str()?
                .into_iter()
                .map(|value| {
                    mapping
                        .get(&value)
                        .copied()
                        .ok_or_else(|| Box::from(format!("Missing label: {value:?}")))
                })
                .collect::<Result<Vec<u32>>>()?;

            df_encoded.with_column(Column::new(name.clone(), c_encoded))?;
        } else {
            df_encoded.with_column(series.clone())?;
        }
    }

    Ok(df_encoded)
}
