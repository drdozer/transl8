
use std::collections::HashMap;

pub fn complement(n: char) -> char {
    match n {
        'a' => 't',
        'c' => 'g',
        'g' => 'c',
        't' => 'a',
        'n' => 'n',
        _ => panic!("Unexpected nucleotide character `{}'", n),
    }
}

pub fn reverse_complement(s: &str) -> String {
    s.chars()
        .rev()
        .map(|c| complement(c))
        .collect()
}

pub fn frame(s: &str, phase: usize) -> Vec<&str> {
    let l = s.len()-phase;
    let r = l % 3 ;
    (phase..s.len()-r).step_by(3).map(|i| &s[i..i+3]).collect()
}

pub fn translate(codons: &Vec<&str>) -> String {
    codons
        .iter()
        .map(|c| *(TRANSLATION_TABLE.get(c).unwrap_or(&"*")))
        .collect()
}

lazy_static! {
pub static ref TRANSLATION_TABLE: HashMap<&'static str, &'static str> = {
    let mut tab = HashMap::new();

    tab.insert("ttt", "F");
    tab.insert("ttc", "F");
    tab.insert("tta", "L");
    tab.insert("ttg", "L");

    tab.insert("ctt", "L");
    tab.insert("ctc", "L");
    tab.insert("cta", "L");
    tab.insert("ctg", "L");

    tab.insert("att", "I");
    tab.insert("atc", "I");
    tab.insert("ata", "I");
    tab.insert("atg", "M");
    
    tab.insert("gtt", "V");
    tab.insert("gtc", "V");
    tab.insert("gta", "V");
    tab.insert("gtg", "V");


    tab.insert("tct", "S");
    tab.insert("tcc", "S");
    tab.insert("tca", "S");
    tab.insert("tcg", "S");

    tab.insert("cct", "P");
    tab.insert("ccc", "P");
    tab.insert("cca", "P");
    tab.insert("ccg", "P");

    tab.insert("act", "T");
    tab.insert("acc", "T");
    tab.insert("aca", "T");
    tab.insert("acg", "T");
    
    tab.insert("gct", "A");
    tab.insert("gcc", "A");
    tab.insert("gca", "A");
    tab.insert("gcg", "A");


    tab.insert("tat", "Y");
    tab.insert("tac", "Y");
    tab.insert("taa", "*");
    tab.insert("tag", "*");

    tab.insert("cat", "H");
    tab.insert("cac", "H");
    tab.insert("caa", "Q");
    tab.insert("cag", "Q");

    tab.insert("aat", "N");
    tab.insert("aac", "N");
    tab.insert("aaa", "K");
    tab.insert("aag", "K");
    
    tab.insert("gat", "D");
    tab.insert("gac", "D");
    tab.insert("gaa", "E");
    tab.insert("gag", "E");


    tab.insert("tgt", "C");
    tab.insert("tgc", "C");
    tab.insert("tga", "*");
    tab.insert("tgg", "W");

    tab.insert("cgt", "R");
    tab.insert("cgc", "R");
    tab.insert("cga", "R");
    tab.insert("cgg", "R");

    tab.insert("agt", "S");
    tab.insert("agc", "S");
    tab.insert("aga", "R");
    tab.insert("agg", "R");
    
    tab.insert("ggt", "G");
    tab.insert("ggc", "G");
    tab.insert("gga", "G");
    tab.insert("ggg", "G");

    tab
};
}
