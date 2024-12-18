use std::fs;
use std::collections::HashSet;
use std::sync::LazyLock;

static MOSTCOMMONWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let contents = fs::read_to_string("src/utils/filter/mostcommonwords.txt")
    .expect("Should have been able to read the file");

    return contents.split("\n").map(|s| {return String::from(s.to_lowercase().as_str().trim())}).collect();   
});

static ALLWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let contents = fs::read_to_string("src/utils/filter/words_alpha.txt")
    .expect("Should have been able to read the file");

    return contents.split("\n").map(|s| {return String::from(s.to_lowercase().as_str().trim())}).collect();
});

pub fn get_most_common_words() -> &'static HashSet<String> {
    return &*MOSTCOMMONWORDS
}

pub fn get_all_word_hashset() -> &'static HashSet<String> {

    return &*ALLWORDS;
}