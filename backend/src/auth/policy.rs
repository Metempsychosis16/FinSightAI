use axum::http::Method;

use crate::models::role::Role;

/// Returns whether a role may perform an operation on a case endpoint.
pub fn is_authorized(role: Role, method: &Method, path: &str) -> bool {
    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();

    let is_case_item = segments.len() == 2
        && segments[0] == "cases"
        && !segments[1].is_empty()
        && segments[1] != "high-risk";

    let is_case_status = segments.len() == 3
        && segments[0] == "cases"
        && !segments[1].is_empty()
        && segments[2] == "status";

    let is_case_investigator = segments.len() == 3
        && segments[0] == "cases"
        && !segments[1].is_empty()
        && segments[2] == "investigator";

    match (method.as_str(), path) {
        ("GET", "/cases") => true,
        ("GET", "/cases/high-risk") => true,
        ("POST", "/cases") => true,

        _ if is_case_item && *method == Method::GET => true,

        _ if (is_case_item || is_case_status) && *method == Method::PATCH => {
            matches!(role, Role::SeniorInvestigator | Role::Admin)
        }

        _ if is_case_item && *method == Method::DELETE => role == Role::Admin,

        _ if is_case_investigator && *method == Method::PATCH => {
            matches!(role, Role::SeniorInvestigator | Role::Admin)
        }

        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_authorized;
    use crate::models::role::Role;
    use axum::http::Method;

    #[test]
    fn investigator_can_read_and_create_cases() {
        assert!(is_authorized(Role::Investigator, &Method::GET, "/cases"));
        assert!(is_authorized(Role::Investigator, &Method::POST, "/cases"));
        assert!(is_authorized(Role::Investigator, &Method::GET, "/cases/42"));
    }

    #[test]
    fn investigator_cannot_update_or_delete_cases() {
        assert!(!is_authorized(
            Role::Investigator,
            &Method::PATCH,
            "/cases/42"
        ));
        assert!(!is_authorized(
            Role::Investigator,
            &Method::PATCH,
            "/cases/42/status"
        ));
        assert!(!is_authorized(
            Role::Investigator,
            &Method::DELETE,
            "/cases/42"
        ));
    }

    #[test]
    fn senior_investigator_can_update_but_not_delete() {
        assert!(is_authorized(
            Role::SeniorInvestigator,
            &Method::PATCH,
            "/cases/42"
        ));
        assert!(is_authorized(
            Role::SeniorInvestigator,
            &Method::PATCH,
            "/cases/42/status"
        ));
        assert!(!is_authorized(
            Role::SeniorInvestigator,
            &Method::DELETE,
            "/cases/42"
        ));
    }

    #[test]
    fn admin_can_delete_cases() {
        assert!(is_authorized(Role::Admin, &Method::DELETE, "/cases/42"));
    }

    #[test]
    fn rejects_unlisted_routes() {
        assert!(!is_authorized(Role::Admin, &Method::GET, "/admin/secrets"));
        assert!(!is_authorized(
            Role::Investigator,
            &Method::DELETE,
            "/cases"
        ));
    }
}

#[test]
fn only_senior_investigators_and_admins_can_assign_cases() {
    assert!(!is_authorized(
        Role::Investigator,
        &Method::PATCH,
        "/cases/42/investigator"
    ));

    assert!(is_authorized(
        Role::SeniorInvestigator,
        &Method::PATCH,
        "/cases/42/investigator"
    ));

    assert!(is_authorized(
        Role::Admin,
        &Method::PATCH,
        "/cases/42/investigator"
    ));
}
