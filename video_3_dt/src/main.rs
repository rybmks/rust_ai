use std::fs::File;
use std::io::Write;

use linfa::Dataset;
use linfa::traits::Fit;
use linfa_trees::DecisionTree;
use ndarray::{Axis, s};
use polars::prelude::*;
use video_3_dt::Result;
use video_3_dt::utils::label_encode;

const SUCCESS_VALUE: f64 = 1.0;

fn main() -> Result<()> {
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .with_parse_options(CsvParseOptions {
            null_values: Some(NullValues::AllColumnsSingle("N/A".into())),
            ..Default::default()
        })
        .try_into_reader_with_file_path(Some("./vgsales.csv".into()))?
        .finish()?
        .drop_nulls::<String>(None)?
        .drop("Name")?
        .drop("Rank")?; // Column is not useful for dt

    let mut df = label_encode(&df)?;

    // dbg!(df.shape());
    // for name in df.get_column_names() {
    //     let col = df.column(name)?;
    //     println!("{name}: unique = {:?}", col.unique()?);
    // }

    let values: Vec<i32> = df
        .column("Global_Sales")?
        .f64()?
        .into_iter()
        .map(|opt_val| {
            if let Some(v) = opt_val {
                if v > SUCCESS_VALUE { 1 } else { 0 }
            } else {
                0
            }
        })
        .collect();

    let mask = Int32Chunked::from_vec("is_success".into(), values).into_series();
    let df = df.with_column(mask)?.drop("Global_Sales")?;

    let feature_names = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let df = df.to_ndarray::<Float32Type>(IndexOrder::C)?;
    let num_features = df.len_of(Axis(1)) - 1;
    let features = df.slice(s![.., 0..num_features]).to_owned();
    let labels = df.column(num_features).to_owned();

    let linfa_dataset =
        Dataset::new(features, labels.mapv(|x| x as usize)).with_feature_names(feature_names);

    let model = DecisionTree::params()
        .split_quality(linfa_trees::SplitQuality::Gini)
        .fit(&linfa_dataset)?;

    let mut safe_tikz = model.export_to_tikz().with_legend().to_string();

    safe_tikz = safe_tikz.replace("_", "\\_"); // Fix legend render issue

    File::create("dt.tex")?.write_all(safe_tikz.as_bytes())?;

    Ok(())
}
