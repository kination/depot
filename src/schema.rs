use std::sync::Arc;

use std::fs::File;
use parquet::file::properties::WriterProperties;
use parquet::arrow::arrow_writer::ArrowWriter;
use arrow_csv::writer::Writer;

use arrow_array::{Int64Array, StringArray, ArrayRef,RecordBatch};
use arrow::datatypes::{Field,Schema,DataType};

use serde::{Deserialize, Serialize};


#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct KafkaConsumerMessage {
    id: i64,
    val: String
}

pub fn get_record_batch(msg: KafkaConsumerMessage) -> RecordBatch {
    let ids = Int64Array::from(vec![msg.id]);
    let vals = StringArray::from(vec![msg.val]);

    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("val", DataType::Utf8, false)
    ]);

    RecordBatch::try_new(Arc::new(schema), vec![
        Arc::new(ids) as ArrayRef,
        Arc::new(vals) as ArrayRef,
    ]).unwrap()
}
