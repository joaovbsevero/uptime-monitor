pub(crate) enum Frequency {
    Hourly,
    Daily,
    Weekly,
}

pub(crate) enum HTTPMethod {
    HEAD,
    GET,
}

pub(crate) struct Check {
    pub(crate) _id: String,
    pub(crate) frequency: Frequency,
    pub(crate) url: String,
    pub(crate) hook: Option<String>,
}

pub(crate) struct History {
    pub(crate) _id: String,
    pub(crate) check_id: String,
    pub(crate) status: bool,
    pub(crate) created_at: String
}