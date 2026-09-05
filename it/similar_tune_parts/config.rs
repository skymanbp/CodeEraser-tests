//! One text table: explicit experiments, including rejected alternatives.
use std::collections::BTreeMap;

pub const TABLE: &str = "\
baseline
no_s weights=3,1,2,1,0,1
no_sl weights=3,1,2,1,0,0
words weights=3,0,2,1,0,0
names weights=3,0,0,0,0,0
name_callee weights=3,0,2,0,0,0
no_name weights=0,1,2,1,1,1
no_callee weights=3,1,0,1,1,1
no_doc weights=3,1,2,0,1,1
no_shape weights=3,0,2,1,1,1
structure weights=0,0,0,0,1,0
name6 weights=6,1,2,1,1,1
name9 weights=9,1,2,1,1,1
callee4 weights=3,1,4,1,1,1
doc3 weights=3,1,2,3,1,1
binary_query qcap=1
cap2_query qcap=2
rare_query dfdiv=4
top8_query select=8
top16_query select=16
active_words weights=3,0,2,1,0,0 length=active
field model=field
field_words model=field weights=3,0,2,1,0,0
field_binary model=field qcap=1
positive_idf idf=positive
log_ndf idf=ndf
k0 k=0/1
khalf_b0 k=1/2 b=0/1
khalf_bhalf k=1/2 b=1/2
khalf_bthree k=1/2
khalf_b1 k=1/2 b=1/1
k1_b0 k=1/1 b=0/1
k1_bhalf k=1/1 b=1/2
k1_bthree k=1/1
k1_b1 k=1/1 b=1/1
kbase_b0 b=0/1
kbase_bquarter b=1/4
kbase_bhalf b=1/2
kbase_b1 b=1/1
k2_b0 k=2/1 b=0/1
k2_bhalf k=2/1 b=1/2
k2_bthree k=2/1
k2_b1 k=2/1 b=1/1
k3_b0 k=3/1 b=0/1
k3_bhalf k=3/1 b=1/2
k3_bthree k=3/1
k3_b1 k=3/1 b=1/1
lm20 model=lm mu=20 weights=3,0,2,1,0,0 length=active
lm100 model=lm mu=100 weights=3,0,2,1,0,0 length=active
lm500 model=lm mu=500 weights=3,0,2,1,0,0 length=active
weighted_jaccard model=jaccard qcap=1
word_jaccard model=jaccard qcap=1 weights=3,0,2,1,0,0
cosine_squared model=cosine
role_first order=role
name_first order=name
shape_first order=shape
ppmi_v1 assoc=v1
ppmi_m1 assoc=ppmi m=1
ppmi_m2 assoc=ppmi m=2
ppmi_min3 assoc=ppmi min=768
ppmi_min4 assoc=ppmi min=1024
ppmi_min6 assoc=ppmi min=1536
ppmi_quarter assoc=ppmi scale=4096
ppmi_mass4 assoc=ppmi mass=4
ppmi_mass8 assoc=ppmi mass=8
ppmi_terms2 assoc=ppmi gate=terms2
ppmi_terms5 assoc=ppmi gate=terms5
ppmi_low_word_mass assoc=ppmi gate=lowmass
ppmi_high_word_mass assoc=ppmi gate=highmass
ppmi_tie assoc=ppmi order=tie
ppmi_band20 assoc=ppmi order=band bands=20
ppmi_band100 assoc=ppmi order=band bands=100
ppmi_rerank5 assoc=ppmi order=stage stage=5
ppmi_rerank20 assoc=ppmi order=stage stage=20
translation_quarter model=translation mix=4 mu=100 weights=3,0,2,1,0,0 length=active
translation_half model=translation mix=2 mu=100 weights=3,0,2,1,0,0 length=active
translation500 model=translation mix=4 mu=500 weights=3,0,2,1,0,0 length=active
rocchio3 assoc=rocchio feedback=3 mass=4
rocchio5 assoc=rocchio feedback=5 mass=4
rm3_style3 assoc=rm3 feedback=3 mass=4
rm3_style5 assoc=rm3 feedback=5 mass=4
callee_second assoc=second mass=4
callee_second_tie assoc=second order=tie mass=4
word_binary_ppmi weights=3,0,2,1,0,0 qcap=1 assoc=ppmi mass=4
rule:spec n1,c1,&,n2,p,&,|
rule:name n1
rule:nc n1,c1,&
rule:shape_spec n1,c1,&,n2,|,p,&
rule:nc2 n1,c2,&
rule:cross_spec n1,c1,&,n2,p,&,|,f,!,&
rule:doc_union n1,c1,&,n2,p,&,|,d2,p,&,|
rule:n2c1 n2,c1,&
rule:n1c2_alt2 n1,c2,&,n2,p,&,|
rule:n1c1_alt3 n1,c1,&,n3,p,&,|
rule:n2c1_alt2 n2,c1,&,n2,p,&,|
rule:name_doc n1,d1,&
rule:name_callee_doc n1,c1,&,d1,&
rule:shape_callee c2,p,&
rule:name_half n1,c1,&,n2,p,&,|,h,&
rule:shape_or_doc n1,c1,&,n2,p,&,|,p,d1,|,&";

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub knobs: BTreeMap<String, String>,
}

impl Config {
    pub fn text<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.knobs.get(key).map_or(default, String::as_str)
    }

    pub fn int(&self, key: &str, default: i128) -> i128 {
        self.knobs
            .get(key)
            .map_or(default, |v| v.parse().expect(key))
    }

    pub fn ratio(&self, key: &str, default: &str) -> (i128, i128) {
        let (a, b) = self.text(key, default).split_once('/').expect("fraction");
        (
            a.parse().expect("numerator"),
            b.parse().expect("denominator"),
        )
    }

    pub fn weights(&self) -> [i128; 6] {
        self.text("weights", "3,1,2,1,1,1")
            .split(',')
            .map(|s| s.parse().expect("weight"))
            .collect::<Vec<_>>()
            .try_into()
            .expect("six channels")
    }
}

pub fn configs() -> Vec<Config> {
    TABLE
        .lines()
        .filter(|l| !l.starts_with("rule:"))
        .map(|l| {
            let mut words = l.split_whitespace();
            Config {
                name: words.next().expect("name").into(),
                knobs: words
                    .map(|w| {
                        let (k, v) = w.split_once('=').expect("key=value");
                        (k.into(), v.into())
                    })
                    .collect(),
            }
        })
        .collect()
}

pub fn rules() -> Vec<(&'static str, &'static str)> {
    TABLE
        .lines()
        .filter_map(|l| l.strip_prefix("rule:"))
        .map(|l| l.split_once(' ').expect("rule expression"))
        .collect()
}
