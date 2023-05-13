use std::sync::Arc;
use std::collections::LinkedList;
use arrow_array::{Int64Array, StringArray, ArrayRef,RecordBatch};
use arrow::datatypes::{Field,Schema,DataType};

use serde::{Deserialize, Serialize};


#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct KafkaConsumerMessage {
    id: i64,
    val: String
}

pub fn get_record_batch(list: Vec<KafkaConsumerMessage>) -> RecordBatch {

    let mut id_vec: Vec<i64> = Vec::new();
    let mut vals_vec: Vec<String> = Vec::new();

    for v in list {
        id_vec.push(v.id);
        vals_vec.push(v.val);
    }
    
    let ids = Int64Array::from(id_vec);
    let vals = StringArray::from(vals_vec);
    
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("val", DataType::Utf8, false)
    ]);

    RecordBatch::try_new(Arc::new(schema), vec![
        Arc::new(ids) as ArrayRef,
        Arc::new(vals) as ArrayRef,
    ]).unwrap()
}
