use std::fs;
use std::collections::HashSet;

pub fn get_most_common_words() -> HashSet<String> {
    let contents = fs::read_to_string("src/utils/filter/mostcommonwords.txt")
    .expect("Should have been able to read the file");

    let hash: HashSet<String> = contents.split("\n").map(|s| {return String::from(s.to_lowercase().as_str().trim())}).collect();

    return hash;
}

pub fn get_all_word_hashset() -> HashSet<String> {
    let contents = fs::read_to_string("src/utils/filter/words_alpha.txt")
    .expect("Should have been able to read the file");

    let hash: HashSet<String> = contents.split("\n").map(|s| {return String::from(s.to_lowercase().as_str().trim())}).collect();
    return hash;
}