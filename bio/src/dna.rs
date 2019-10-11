use std::collections::HashMap;

pub fn complement(n: char) -> char {
    match n {
        'a' => 't',
        'c' => 'g',
        'g' => 'c',
        't' => 'a',
        'n' => 'n',
        'A' => 'T',
        'C' => 'G',
        'G' => 'C',
        'T' => 'A',
        _ => 'N', // fixme -- should warn
        // _ => panic!("Unexpected nucleotide character `{}'", n),
    }
}

pub fn reverse_complement(s: &str) -> String {
    s.chars()
        .rev()
        .map(complement)
        .collect()
}

pub fn frame(s: &str, phase: usize) -> Vec<&str> {
    let l = s.len()-phase;
    let r = l % 3 ;
    (phase..s.len()-r).step_by(3).map(|i| &s[i..i+3]).collect()
}

pub fn translate(codons: &[&str]) -> String {
    codons
        .iter()
        .map(|c| *(TRANSLATION_TABLE.get(*c).unwrap_or(&"*")))
        .collect()
}

lazy_static! {
pub static ref TRANSLATION_TABLE: HashMap<String, &'static str> = {
    let mut tab = HashMap::new();

    tab.insert("ttt".to_string(), "F");
    tab.insert("ttc".to_string(), "F");
    tab.insert("tta".to_string(), "L");
    tab.insert("ttg".to_string(), "L");

    tab.insert("ctt".to_string(), "L");
    tab.insert("ctc".to_string(), "L");
    tab.insert("cta".to_string(), "L");
    tab.insert("ctg".to_string(), "L");

    tab.insert("att".to_string(), "I");
    tab.insert("atc".to_string(), "I");
    tab.insert("ata".to_string(), "I");
    tab.insert("atg".to_string(), "M");
    
    tab.insert("gtt".to_string(), "V");
    tab.insert("gtc".to_string(), "V");
    tab.insert("gta".to_string(), "V");
    tab.insert("gtg".to_string(), "V");


    tab.insert("tct".to_string(), "S");
    tab.insert("tcc".to_string(), "S");
    tab.insert("tca".to_string(), "S");
    tab.insert("tcg".to_string(), "S");

    tab.insert("cct".to_string(), "P");
    tab.insert("ccc".to_string(), "P");
    tab.insert("cca".to_string(), "P");
    tab.insert("ccg".to_string(), "P");

    tab.insert("act".to_string(), "T");
    tab.insert("acc".to_string(), "T");
    tab.insert("aca".to_string(), "T");
    tab.insert("acg".to_string(), "T");
    
    tab.insert("gct".to_string(), "A");
    tab.insert("gcc".to_string(), "A");
    tab.insert("gca".to_string(), "A");
    tab.insert("gcg".to_string(), "A");


    tab.insert("tat".to_string(), "Y");
    tab.insert("tac".to_string(), "Y");
    tab.insert("taa".to_string(), "*");
    tab.insert("tag".to_string(), "*");

    tab.insert("cat".to_string(), "H");
    tab.insert("cac".to_string(), "H");
    tab.insert("caa".to_string(), "Q");
    tab.insert("cag".to_string(), "Q");

    tab.insert("aat".to_string(), "N");
    tab.insert("aac".to_string(), "N");
    tab.insert("aaa".to_string(), "K");
    tab.insert("aag".to_string(), "K");
    
    tab.insert("gat".to_string(), "D");
    tab.insert("gac".to_string(), "D");
    tab.insert("gaa".to_string(), "E");
    tab.insert("gag".to_string(), "E");


    tab.insert("tgt".to_string(), "C");
    tab.insert("tgc".to_string(), "C");
    tab.insert("tga".to_string(), "*");
    tab.insert("tgg".to_string(), "W");

    tab.insert("cgt".to_string(), "R");
    tab.insert("cgc".to_string(), "R");
    tab.insert("cga".to_string(), "R");
    tab.insert("cgg".to_string(), "R");

    tab.insert("agt".to_string(), "S");
    tab.insert("agc".to_string(), "S");
    tab.insert("aga".to_string(), "R");
    tab.insert("agg".to_string(), "R");
    
    tab.insert("ggt".to_string(), "G");
    tab.insert("ggc".to_string(), "G");
    tab.insert("gga".to_string(), "G");
    tab.insert("ggg".to_string(), "G");

    for (k, v) in tab.clone().iter() {
        tab.insert(String::from(k), v);
    }

    tab
};
}
