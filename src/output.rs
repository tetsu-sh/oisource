use crate::article::Article;
use csv::Writer;

pub fn write_csv(records: &Vec<Article>) {
    let mut wtr = Writer::from_path("test.csv").unwrap();
    for record in records.iter() {
        wtr.serialize(record);
    }
}
