use ndarray::ArrayView1;

pub mod ppg;
mod filter;
mod tests;
mod ecg;

pub struct Analysis {
    config: AnalysisConfig,
    rt: tokio::runtime::Runtime,
}

pub struct AnalysisConfig {
    //ecg_parameters: ECGParameters,
    //ppg_parameters: PPGParameters,
}

pub struct ECGParameters {}



impl Analysis {
    pub fn new(config: AnalysisConfig) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        Self {
            config,
            rt,
        }
    }
}

