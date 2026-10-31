use jiff::tz::TimeZone;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplaySettings {
    language: String,
    time_zone: String,
    region: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid display settings")]
pub struct InvalidDisplaySettings;

#[derive(Debug, Default)]
pub struct DisplayPatch {
    language: Option<String>,
    time_zone: Option<String>,
    region: Option<String>,
}
impl DisplayPatch {
    pub fn parse(
        language: Option<String>,
        time_zone: Option<String>,
        region: Option<String>,
    ) -> Result<Self, InvalidDisplaySettings> {
        DisplaySettings::parse("en", "UTC", "en-US")?.patch(
            language.as_deref(),
            time_zone.as_deref(),
            region.as_deref(),
        )?;
        Ok(Self {
            language,
            time_zone,
            region,
        })
    }
    pub fn resolve(
        &self,
        defaults: &DisplaySettings,
    ) -> Result<DisplaySettings, InvalidDisplaySettings> {
        defaults.patch(
            self.language.as_deref(),
            self.time_zone.as_deref(),
            self.region.as_deref(),
        )
    }
}

impl DisplaySettings {
    pub fn parse(
        language: &str,
        time_zone: &str,
        region: &str,
    ) -> Result<Self, InvalidDisplaySettings> {
        if !matches!(language, "en" | "ko" | "ja" | "fr")
            || !matches!(region, "en-US" | "ko-KR" | "ja-JP" | "fr-FR")
            || time_zone.len() > 128
            || time_zone.is_empty()
        {
            return Err(InvalidDisplaySettings);
        }
        let zone = TimeZone::get(time_zone).map_err(|_| InvalidDisplaySettings)?;
        let name = zone.iana_name().ok_or(InvalidDisplaySettings)?;
        Ok(Self {
            language: language.to_owned(),
            time_zone: name.to_owned(),
            region: region.to_owned(),
        })
    }
    pub fn from_environment() -> Result<Self, InvalidDisplaySettings> {
        let zone = TimeZone::try_system().map_err(|_| InvalidDisplaySettings)?;
        let name = zone.iana_name().ok_or(InvalidDisplaySettings)?;
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|key| std::env::var(key).ok().filter(|value| !value.is_empty()))
            .unwrap_or_default();
        let (language, region) = match locale.split(['_', '-', '.']).next() {
            Some("ko") => ("ko", "ko-KR"),
            Some("ja") => ("ja", "ja-JP"),
            Some("fr") => ("fr", "fr-FR"),
            _ => ("en", "en-US"),
        };
        Self::parse(language, name, region)
    }
    pub fn language(&self) -> &str {
        &self.language
    }
    pub fn time_zone(&self) -> &str {
        &self.time_zone
    }
    pub fn region(&self) -> &str {
        &self.region
    }
    pub fn patch(
        &self,
        language: Option<&str>,
        time_zone: Option<&str>,
        region: Option<&str>,
    ) -> Result<Self, InvalidDisplaySettings> {
        Self::parse(
            language.unwrap_or(self.language()),
            time_zone.unwrap_or(self.time_zone()),
            region.unwrap_or(self.region()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_values_and_validation() -> Result<(), InvalidDisplaySettings> {
        let value = DisplaySettings::parse("ko", "America/New_York", "fr-FR")?;
        let changed = value.patch(Some("ja"), None, None)?;
        assert_eq!(changed.language(), "ja");
        assert_eq!(changed.region(), "fr-FR");
        assert_eq!(changed.time_zone(), "America/New_York");
        assert!(DisplaySettings::parse("xx", "UTC", "en-US").is_err());
        assert!(DisplaySettings::parse("en", "Not/AZone", "en-US").is_err());
        assert!(DisplaySettings::parse("en", "+09:00", "en-US").is_err());
        Ok(())
    }
}
