use std::fs::File;
use parquet::file::properties::WriterProperties;
use parquet::arrow::arrow_writer::ArrowWriter;
use arrow_csv::writer::Writer;

use arrow_array::{RecordBatch};


pub fn write_as_csv(batch: RecordBatch, file_name: &str) {
    let file = File::create(file_name).unwrap();

    let mut writer = Writer::new(file);
    let batches = vec![&batch, &batch];

    for batch in batches {
        writer.write(batch).unwrap();
    }
}

pub fn write_as_parquet(batch: RecordBatch, file_name: &str) {
    let file = File::create(file_name).unwrap();

    let mut writer = ArrowWriter::try_new(
        file, 
        batch.schema(),
        Some(WriterProperties::builder().build())
    ).unwrap();

    writer.write(&batch).unwrap();
    writer.close().unwrap();
}
