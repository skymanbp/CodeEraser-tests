use super::*;

/// Porter 1980's own worked examples, one or two per step (the paper
/// lists them beside each rule), plus a full-length run. One text
/// block rather than a pair table: a tuple table of this length is
/// the byte shape of every other stance table in the suite.
const PAPER: &str = "\
caresses caress
ponies poni
cats cat
agreed agre
plastered plaster
motoring motor
conflated conflat
hopping hop
filing file
happy happi
sky sky
relational relat
rational ration
digitizer digit
hesitanci hesit
electrical electr
hopeful hope
adjustment adjust
adoption adopt
controll control
generalizations gener";

#[test]
fn porter_paper_examples_stem_as_published() {
    for line in PAPER.lines() {
        let (word, want) = line.split_once(' ').expect("word stem");
        assert_eq!(stem(word), want, "{word}");
    }
}

#[test]
fn short_and_non_ascii_words_are_left_alone() {
    assert_eq!(stem("is"), "is");
    assert_eq!(stem("café"), "café");
    assert_eq!(stem("x1"), "x1");
}
