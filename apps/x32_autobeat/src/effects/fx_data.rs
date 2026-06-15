use crate::scaling::{afine2float, log2float, ratio2float};
use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum ScalingType {
    Log { min: f32, range_log: f32 },
    Affine { min: f32, range: f32 },
    Ratio { max: f32 },
}

impl ScalingType {
    pub fn scale(&self, val: f32) -> f32 {
        match self {
            ScalingType::Log { min, range_log } => log2float(val, *min, *range_log),
            ScalingType::Affine { min, range } => afine2float(val, *min, *range),
            ScalingType::Ratio { max } => ratio2float(val, *max),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FxParameter {
    pub index: usize,
    pub scaling: ScalingType,
    pub base_unit: BaseUnit, // To know how to interpret BPM -> Value
}

#[derive(Debug, Clone, Copy)]
pub enum BaseUnit {
    Milliseconds,
    Hertz,
}

#[derive(Debug, Clone)]
pub struct FxDefinition {
    pub parameters: Vec<FxParameter>,
}

lazy_static! {
    pub static ref FX_DATA: HashMap<&'static str, FxDefinition> = {
        let mut m = HashMap::new();

        // *** DELAYS ***
        // DLY (Stereo Delay) - fxparse1.c
        // Time: Param 2 [1, 3000] afine2float
        m.insert("DLY", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Affine { min: 1.0, range: 2999.0 }, // 3000-1
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // 3TAP (3-Tap Delay) - fxparse1.c
        // Time Base: Param 1 [1, 3000] afine2float
        // It implies the taps are relative to this base or absolute?
        // "time [1, 3000]"
        // There are also factorA/B. But typically we sync the base time.
        m.insert("3TAP", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Affine { min: 1.0, range: 2999.0 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // 4TAP - fxparse1.c
        // Time: Param 1 [1, 3000] afine2float
        m.insert("4TAP", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Affine { min: 1.0, range: 2999.0 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // *** MODULATION ***
        // CRS (Stereo Chorus) - fxparse1.c
        // Speed: Param 1 [0.05, 5] log2float(0.05, 4.605...)
        // Delay L: Param 4 [0.5, 50] log2float
        // Delay R: Param 5 [0.5, 50] log2float
        // Usually we sync Speed (Hz).
        m.insert("CRS", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Log { min: 0.05, range_log: 4.605170186 },
                    base_unit: BaseUnit::Hertz,
                }
            ]
        });

        // FLNG (Stereo Flanger) - fxparse1.c
        // Speed: Param 1 [0.05, 5] log2float
        m.insert("FLNG", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Log { min: 0.05, range_log: 4.605170186 },
                    base_unit: BaseUnit::Hertz,
                }
            ]
        });

        // PHAS (Stereo Phaser) - fxparse1.c / XPHAS
        // Speed: Param 1 [0.05, 5] log2float
        m.insert("PHAS", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Log { min: 0.05, range_log: 4.605170186 },
                    base_unit: BaseUnit::Hertz,
                }
            ]
        });

        // *** REVERBS ***
        // HALL - fxparse1.c
        // PreDelay: Param 1 [0, 200] ratio2float(200)
        // Decay: Param 2 [0.2, 5] log2float(0.2, 3.218...)
        m.insert("HALL", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Log { min: 0.2, range_log: 3.218895825 },
                    base_unit: BaseUnit::Milliseconds, // Decay is time
                }
            ]
        });

        // PLAT (Plate) - fxparse1.c
        // PreDelay: Param 1 [0, 200] ratio2float(200)
        // Decay: Param 2 [0.5, 10] log2float(0.5, 2.995...)
        m.insert("PLAT", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Log { min: 0.5, range_log: 2.995732274 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // AMBI (Ambience) - fxparse1.c
        // PreDelay: Param 1 [0, 200]
        // Decay: Param 2 [0.2, 7.3] log2float(0.2, 3.597...)
        m.insert("AMBI", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Log { min: 0.2, range_log: 3.597312261 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // ROOM - fxparse1.c
        // PreDelay: Param 1 [0, 200]
        // Decay: Param 2 [0.3, 29] log2float(0.3, 4.571...)
        m.insert("ROOM", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Log { min: 0.3, range_log: 4.571268634 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // CHAM (Chamber) - Same as ROOM for time params
        m.insert("CHAM", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Log { min: 0.3, range_log: 4.571268634 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // RPLT (Rich Plate) - Same as ROOM for time params
        m.insert("RPLT", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Log { min: 0.3, range_log: 4.571268634 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // VREV (Vintage Reverb) - fxparse1.c
        // PreDelay: Param 1 [0, 120]
        // Decay: Param 2 [0.3, 4.5] log2float
        m.insert("VREV", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 120.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Log { min: 0.3, range_log: 2.708050201 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // VRM (Vintage Room) - fxparse1.c
        // PreDelay: Param 1 [0, 200]
        // Decay: Param 2 [0.1, 20] log2float
        m.insert("VRM", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Log { min: 0.1, range_log: 5.298317367 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // GATE (Gated Reverb) - fxparse1.c
        // PreDelay: Param 1 [0, 200]
        // Decay: Param 2 [140, 1000] afine2float (linear!)
        // Attack: Param 3 [0, 30] ratio2float
        m.insert("GATE", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Affine { min: 140.0, range: 720.0 }, // 1000-140 = 860 in code?
                    // Code says: afine2float(140., 860.) -> Range is 860. Max is 1000.
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 3,
                    scaling: ScalingType::Ratio { max: 30.0 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // RVRS (Reverse Reverb) - fxparse1.c
        // PreDelay: Param 1 [0, 200]
        // Decay: Param 2 [140, 1000] afine2float(140, 860)
        // Rise: Param 3 [0, 50] ratio2float
        m.insert("RVRS", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 2,
                    scaling: ScalingType::Affine { min: 140.0, range: 860.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 3,
                    scaling: ScalingType::Ratio { max: 50.0 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // D_RV (Delay + Reverb) - fxparse1.c
        // Time: Param 1 [1, 3000] afine2float
        // PreDelay: Param 7 [0, 200] ratio2float
        // Decay: Param 8 [0.1, 5] log2float
        m.insert("D_RV", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Affine { min: 1.0, range: 2999.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 7,
                    scaling: ScalingType::Ratio { max: 200.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 8,
                    scaling: ScalingType::Log { min: 0.1, range_log: 3.912023005 },
                    base_unit: BaseUnit::Milliseconds,
                }
            ]
        });

        // D_CR (Delay + Chorus) - fxparse1.c
        // Time: Param 1 [1, 3000]
        // Speed: Param 7 [0.05, 4] log2float
        let d_cr_def = FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Affine { min: 1.0, range: 2999.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 7,
                    scaling: ScalingType::Log { min: 0.05, range_log: 4.382026635 },
                    base_unit: BaseUnit::Hertz,
                }
            ]
        };
        m.insert("D_CR", d_cr_def.clone());
        m.insert("DLY+CHO", d_cr_def);

        // D_FL (Delay + Flanger) - fxparse1.c
        // Time: Param 1 [1, 3000]
        // Speed: Param 7 [0.05, 4] log2float
        m.insert("D_FL", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Affine { min: 1.0, range: 2999.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 7,
                    scaling: ScalingType::Log { min: 0.05, range_log: 4.382026635 },
                    base_unit: BaseUnit::Hertz,
                }
            ]
        });

        // MODD (Modulation Delay) - fxparse1.c
        // Time: Param 1 [1, 3000]
        // Rate: Param 7 [0.05, 10] log2float
        // Decay: Param 10 [1, 10] afine2float(1, 9)
        m.insert("MODD", FxDefinition {
            parameters: vec![
                FxParameter {
                    index: 1,
                    scaling: ScalingType::Affine { min: 1.0, range: 2999.0 },
                    base_unit: BaseUnit::Milliseconds,
                },
                FxParameter {
                    index: 7,
                    scaling: ScalingType::Log { min: 0.05, range_log: 5.298317367 },
                    base_unit: BaseUnit::Hertz,
                },
                FxParameter {
                    index: 10,
                    scaling: ScalingType::Affine { min: 1.0, range: 9.0 },
                    base_unit: BaseUnit::Milliseconds, // Decay here is small? "1-10". Maybe seconds or repeats? Assuming Seconds if low range.
                }
            ]
        });

        m
    };
}
