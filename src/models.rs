use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelBase {
    Unknown,
    B181,
    B157,
    B155,
    B163,
    B171,
    B162,
    B164,
    B168,
    B172,
    B174,
}

impl ModelBase {
    pub fn from_code(code: &str) -> Self {
        match code {
            "B181" => Self::B181,
            "B157" => Self::B157,
            "B155" => Self::B155,
            "B163" => Self::B163,
            "B171" => Self::B171,
            "B162" => Self::B162,
            "B164" => Self::B164,
            "B168" => Self::B168,
            "B172" => Self::B172,
            "B174" => Self::B174,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN",
            Self::B181 => "B181",
            Self::B157 => "B157",
            Self::B155 => "B155",
            Self::B163 => "B163",
            Self::B171 => "B171",
            Self::B162 => "B162",
            Self::B164 => "B164",
            Self::B168 => "B168",
            Self::B172 => "B172",
            Self::B174 => "B174",
        }
    }

    pub fn supports_case_led(self) -> bool {
        matches!(self, Self::B181)
    }

    pub fn supports_personalized_anc(self) -> bool {
        matches!(self, Self::B155)
    }

    pub fn supports_enhanced_bass(self) -> bool {
        matches!(self, Self::B171 | Self::B172 | Self::B168 | Self::B162)
    }

    pub fn supports_in_ear_detection(self) -> bool {
        !matches!(self, Self::B174)
    }

    pub fn supports_custom_eq(self) -> bool {
        !matches!(self, Self::B181)
    }

    pub fn supports_listening_modes(self) -> bool {
        matches!(self, Self::B168 | Self::B172)
    }
}

impl fmt::Display for ModelBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ModelBase {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ModelBase::from_code(s))
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub base: ModelBase,
    pub anc_capable: bool,
}

pub static MODEL_LIST: &[ModelInfo] = &[
    ModelInfo {
        id: "ear_1_white",
        name: "Nothing Ear (1)",
        base: ModelBase::B181,
        anc_capable: true,
    },
    ModelInfo {
        id: "ear_1_black",
        name: "Nothing Ear (1)",
        base: ModelBase::B181,
        anc_capable: true,
    },
    ModelInfo {
        id: "ear_stick",
        name: "Nothing Ear (stick)",
        base: ModelBase::B157,
        anc_capable: false,
    },
    ModelInfo {
        id: "ear_2_white",
        name: "Nothing Ear (2)",
        base: ModelBase::B155,
        anc_capable: true,
    },
    ModelInfo {
        id: "ear_2_black",
        name: "Nothing Ear (2)",
        base: ModelBase::B155,
        anc_capable: true,
    },
    ModelInfo {
        id: "corsola_orange",
        name: "CMF Buds Pro",
        base: ModelBase::B163,
        anc_capable: true,
    },
    ModelInfo {
        id: "corsola_black",
        name: "CMF Buds Pro",
        base: ModelBase::B163,
        anc_capable: true,
    },
    ModelInfo {
        id: "corsola_white",
        name: "CMF Buds Pro",
        base: ModelBase::B163,
        anc_capable: true,
    },
    ModelInfo {
        id: "entei_black",
        name: "Nothing Ear",
        base: ModelBase::B171,
        anc_capable: true,
    },
    ModelInfo {
        id: "entei_white",
        name: "Nothing Ear",
        base: ModelBase::B171,
        anc_capable: true,
    },
    ModelInfo {
        id: "cleffa_black",
        name: "Nothing Ear (a)",
        base: ModelBase::B162,
        anc_capable: true,
    },
    ModelInfo {
        id: "cleffa_white",
        name: "Nothing Ear (a)",
        base: ModelBase::B162,
        anc_capable: true,
    },
    ModelInfo {
        id: "cleffa_yellow",
        name: "Nothing Ear (a)",
        base: ModelBase::B162,
        anc_capable: true,
    },
    ModelInfo {
        id: "crobat_orange",
        name: "CMF Neckband Pro",
        base: ModelBase::B164,
        anc_capable: true,
    },
    ModelInfo {
        id: "crobat_white",
        name: "CMF Neckband Pro",
        base: ModelBase::B164,
        anc_capable: true,
    },
    ModelInfo {
        id: "crobat_black",
        name: "CMF Neckband Pro",
        base: ModelBase::B164,
        anc_capable: true,
    },
    ModelInfo {
        id: "donphan_black",
        name: "CMF Buds",
        base: ModelBase::B168,
        anc_capable: true,
    },
    ModelInfo {
        id: "donphan_white",
        name: "CMF Buds",
        base: ModelBase::B168,
        anc_capable: true,
    },
    ModelInfo {
        id: "donphan_orange",
        name: "CMF Buds",
        base: ModelBase::B168,
        anc_capable: true,
    },
    ModelInfo {
        id: "espeon_black",
        name: "CMF Buds Pro 2",
        base: ModelBase::B172,
        anc_capable: true,
    },
    ModelInfo {
        id: "espeon_white",
        name: "CMF Buds Pro 2",
        base: ModelBase::B172,
        anc_capable: true,
    },
    ModelInfo {
        id: "espeon_orange",
        name: "CMF Buds Pro 2",
        base: ModelBase::B172,
        anc_capable: true,
    },
    ModelInfo {
        id: "espeon_blue",
        name: "CMF Buds Pro 2",
        base: ModelBase::B172,
        anc_capable: true,
    },
    ModelInfo {
        id: "flaaffy_white",
        name: "Nothing Ear (open)",
        base: ModelBase::B174,
        anc_capable: false,
    },
];

const SKU_TO_MODEL_PAIRS: &[(&str, &str)] = &[
    ("01", "ear_1_white"),
    ("02", "ear_1_black"),
    ("03", "ear_1_white"),
    ("04", "ear_1_black"),
    ("06", "ear_1_black"),
    ("07", "ear_1_white"),
    ("08", "ear_1_black"),
    ("10", "ear_1_black"),
    ("14", "ear_stick"),
    ("15", "ear_stick"),
    ("16", "ear_stick"),
    ("17", "ear_2_white"),
    ("18", "ear_2_white"),
    ("19", "ear_2_white"),
    ("27", "ear_2_black"),
    ("28", "ear_2_black"),
    ("29", "ear_2_black"),
    ("30", "corsola_black"),
    ("31", "corsola_black"),
    ("32", "corsola_white"),
    ("33", "corsola_white"),
    ("34", "corsola_orange"),
    ("35", "corsola_orange"),
    ("48", "crobat_orange"),
    ("49", "crobat_white"),
    ("50", "crobat_black"),
    ("51", "crobat_black"),
    ("52", "crobat_white"),
    ("53", "crobat_orange"),
    ("54", "donphan_black"),
    ("55", "donphan_black"),
    ("56", "donphan_white"),
    ("57", "donphan_white"),
    ("58", "donphan_orange"),
    ("59", "donphan_orange"),
    ("61", "entei_black"),
    ("62", "entei_white"),
    ("63", "cleffa_black"),
    ("64", "cleffa_white"),
    ("65", "cleffa_yellow"),
    ("66", "cleffa_black"),
    ("67", "cleffa_white"),
    ("68", "cleffa_yellow"),
    ("69", "entei_black"),
    ("70", "entei_white"),
    ("71", "cleffa_black"),
    ("72", "cleffa_white"),
    ("73", "cleffa_yellow"),
    ("74", "entei_black"),
    ("75", "entei_white"),
    ("76", "espeon_black"),
    ("77", "espeon_white"),
    ("78", "espeon_orange"),
    ("79", "espeon_blue"),
    ("80", "espeon_blue"),
    ("81", "espeon_orange"),
    ("82", "espeon_white"),
    ("83", "espeon_black"),
    ("11200005", "flaaffy_white"),
];

pub static MODEL_BY_ID: Lazy<HashMap<&'static str, &'static ModelInfo>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for info in MODEL_LIST {
        map.insert(info.id, info);
    }
    map
});

pub static SKU_TO_MODEL: Lazy<HashMap<&'static str, &'static ModelInfo>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for (sku, model_id) in SKU_TO_MODEL_PAIRS {
        if let Some(info) = MODEL_BY_ID.get(model_id) {
            map.insert(*sku, *info);
        }
    }
    map
});

pub fn model_from_id(id: &str) -> Option<&'static ModelInfo> {
    MODEL_BY_ID.get(id).copied()
}

pub fn model_from_sku(sku: &str) -> Option<&'static ModelInfo> {
    SKU_TO_MODEL.get(sku).copied()
}
