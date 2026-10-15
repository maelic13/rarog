//! Experimental imported-network adapter. Search still owns draws and corrections.
use std::sync::OnceLock;

use dragon_nnue::{AccumulatorCache, Network};

use crate::board::{Board, Color, Piece};

const EMBEDDED_MODEL: &[u8] = include_bytes!("../../assets/nnue/dragon-network.nnue");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EvaluationMode {
    #[default]
    Classical,
    DragonNnue,
}

impl EvaluationMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Classical => "Classical",
            Self::DragonNnue => "Dragon NNUE",
        }
    }

    pub const fn diagnostic_name(self) -> &'static str {
        match self {
            Self::Classical => "Classical",
            Self::DragonNnue => "DragonNNUE",
        }
    }
}

/// Configuration is resolved before search; workers borrow the immutable model.
/// The loader accepts one fixed model identity, so one process-wide model suffices.
#[derive(Clone)]
pub struct EvaluationOptions {
    mode: EvaluationMode,
    requested: EvaluationMode,
    file: String,
    scale: i32,
    model: Option<&'static Network>,
}

impl Default for EvaluationOptions {
    fn default() -> Self {
        Self {
            mode: EvaluationMode::Classical,
            requested: EvaluationMode::Classical,
            file: "<embedded>".into(),
            scale: 72,
            model: None,
        }
    }
}

impl EvaluationOptions {
    pub const fn mode(&self) -> EvaluationMode {
        self.mode
    }
    pub fn file(&self) -> &str {
        &self.file
    }
    pub const fn scale(&self) -> i32 {
        self.scale
    }

    pub fn set_mode(&mut self, value: &str) -> Result<(), String> {
        let mode = match value.to_ascii_lowercase().as_str() {
            "classical" => EvaluationMode::Classical,
            "dragon nnue" => EvaluationMode::DragonNnue,
            _ => return Err("Evaluation must be Classical or Dragon NNUE".into()),
        };
        if mode == EvaluationMode::DragonNnue {
            if cfg!(feature = "texel") {
                return Err("Dragon NNUE is unavailable in HCE Texel tracing builds".into());
            }
            self.requested = mode;
            if self.model.is_none() {
                self.model = Some(load_model(&self.file)?);
            }
        }
        self.mode = mode;
        self.requested = mode;
        Ok(())
    }

    pub fn set_file(&mut self, file: &str) -> Result<(), String> {
        if file == self.file && self.mode == self.requested {
            return Ok(());
        }
        let model = if self.requested == EvaluationMode::DragonNnue {
            Some(load_model(file)?)
        } else {
            None
        };
        self.file = file.to_owned();
        self.model = model;
        self.mode = self.requested;
        Ok(())
    }

    pub fn set_scale(&mut self, scale: i32) -> Result<(), String> {
        if !(10..=500).contains(&scale) {
            return Err("Dragon NNUE Scale must be in 10..=500".into());
        }
        self.scale = scale;
        Ok(())
    }

    pub(super) fn same_configuration(&self, other: &Self) -> bool {
        self.mode == other.mode
            && self.requested == other.requested
            && self.file == other.file
            && self.scale == other.scale
    }

    pub(super) fn model(&self) -> Option<&'static Network> {
        if self.mode == EvaluationMode::DragonNnue {
            self.model
        } else {
            None
        }
    }
}

fn load_model(file: &str) -> Result<&'static Network, String> {
    static MODEL: OnceLock<Network> = OnceLock::new();
    if file == "<embedded>"
        && let Some(model) = MODEL.get()
    {
        return Ok(model);
    }
    // A changed external path is verified even if the model is already resident.
    let network = if file == "<embedded>" {
        Network::from_bytes(EMBEDDED_MODEL)
    } else {
        Network::load(file)
    }
    .map_err(|e| format!("Unable to load Dragon NNUE from {file}: {e}"))?;
    let _ = MODEL.set(network);
    Ok(MODEL.get().expect("a verified model was installed"))
}

pub(super) fn evaluate(
    model: &Network,
    board: &Board,
    scale: i32,
    cache: &mut AccumulatorCache,
) -> i32 {
    let pieces = std::array::from_fn(|i| {
        board
            .pieces(
                if i < 6 { Color::White } else { Color::Black },
                Piece::ALL[i % 6],
            )
            .0
    });
    cache
        .evaluate_scaled(model, &pieces, board.side_to_move() == Color::Black, scale)
        .expect("the engine board and configured scale are valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_reject_invalid_values_without_changing_mode() {
        let mut options = EvaluationOptions::default();
        assert_eq!(options.mode(), EvaluationMode::Classical);
        assert!(options.set_mode("unknown").is_err());
        assert!(options.set_scale(0).is_err());
        assert_eq!(options.scale(), 72);
        options.set_file("missing-dragon-network.nnue").unwrap();
        assert!(options.set_mode("Dragon NNUE").is_err());
        assert_eq!(options.mode(), EvaluationMode::Classical);
    }
}
