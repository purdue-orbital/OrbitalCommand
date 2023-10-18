#[derive(Clone, Copy)]
pub enum ModulationTypes {
    QPSK,
    BPSK,
    FSK,
    ASK,
}

#[derive(Clone, Copy)]
pub struct RadioSetting {
    pub sample_rate: f32,
    pub baud_rate: f32,
    pub lo_frequency: f32,
    pub modulation_type: ModulationTypes,
}

impl RadioSetting {
    pub fn new(sample_rate: f32, baud_rate: f32, lo_frequency: f32, modulation_type: ModulationTypes) -> RadioSetting {
        RadioSetting {
            sample_rate,
            baud_rate,
            lo_frequency,
            modulation_type,
        }
    }
}