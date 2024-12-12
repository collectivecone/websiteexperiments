use std::fs;

pub fn get_most_common_words() -> Vec<String> {

    let contents = fs::read_to_string("src/utils/filter/mostcommonwords.txt")
    .expect("Should have been able to read the file");

    let vec: Vec<String> = contents.split("\n").map(|s| {return String::from(s)}).collect();



    return vec;

}