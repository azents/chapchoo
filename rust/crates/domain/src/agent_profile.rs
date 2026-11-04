/// Product palette identity. Actual appearance tokens belong to presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentPalette {
    Butter,
    Sage,
    Apricot,
    Teal,
    Lilac,
    Rose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Invalid Agent palette")]
pub struct InvalidAgentPalette;

impl AgentPalette {
    /// Fixed initial ordering for one-time existing-Agent default assignment.
    /// This is not catalogue display order and must not be reordered.
    pub const INITIAL: [Self; 6] = [
        Self::Butter,
        Self::Sage,
        Self::Apricot,
        Self::Teal,
        Self::Lilac,
        Self::Rose,
    ];

    pub fn parse(value: &str) -> Result<Self, InvalidAgentPalette> {
        match value {
            "butter" => Ok(Self::Butter),
            "sage" => Ok(Self::Sage),
            "apricot" => Ok(Self::Apricot),
            "teal" => Ok(Self::Teal),
            "lilac" => Ok(Self::Lilac),
            "rose" => Ok(Self::Rose),
            _ => Err(InvalidAgentPalette),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Butter => "butter",
            Self::Sage => "sage",
            Self::Apricot => "apricot",
            Self::Teal => "teal",
            Self::Lilac => "lilac",
            Self::Rose => "rose",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_keys_have_stable_order_and_roundtrip() -> Result<(), InvalidAgentPalette> {
        let keys = AgentPalette::INITIAL.map(AgentPalette::as_str);
        assert_eq!(keys, ["butter", "sage", "apricot", "teal", "lilac", "rose"]);
        for palette in AgentPalette::INITIAL {
            assert_eq!(AgentPalette::parse(palette.as_str())?, palette);
        }
        Ok(())
    }

    #[test]
    fn only_product_palette_keys_are_accepted() {
        for value in ["", "Butter", " butter", "#edca6e", "custom", "sage "] {
            assert_eq!(AgentPalette::parse(value), Err(InvalidAgentPalette));
        }
    }
}
