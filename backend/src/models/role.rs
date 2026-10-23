use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "INVESTIGATOR")]
    Investigator,

    #[serde(rename = "SENIOR_INVESTIGATOR")]
    SeniorInvestigator,

    #[serde(rename = "ADMIN")]
    Admin,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Investigator => "INVESTIGATOR",
            Self::SeniorInvestigator => "SENIOR_INVESTIGATOR",
            Self::Admin => "ADMIN",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "INVESTIGATOR" => Some(Self::Investigator),
            "SENIOR_INVESTIGATOR" => Some(Self::SeniorInvestigator),
            "ADMIN" => Some(Self::Admin),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Role;

    #[test]
    fn parses_all_supported_roles() {
        assert_eq!(Role::parse("INVESTIGATOR"), Some(Role::Investigator));
        assert_eq!(
            Role::parse("SENIOR_INVESTIGATOR"),
            Some(Role::SeniorInvestigator)
        );
        assert_eq!(Role::parse("ADMIN"), Some(Role::Admin));
    }

    #[test]
    fn rejects_unknown_roles() {
        assert_eq!(Role::parse("ROOT"), None);
        assert_eq!(Role::parse("admin"), None);
        assert_eq!(Role::parse(""), None);
    }

    #[test]
    fn converts_roles_to_database_labels() {
        assert_eq!(Role::Investigator.as_str(), "INVESTIGATOR");
        assert_eq!(Role::SeniorInvestigator.as_str(), "SENIOR_INVESTIGATOR");
        assert_eq!(Role::Admin.as_str(), "ADMIN");
    }
}
