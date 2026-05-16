#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocationQuery {
    pub near: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaymentMethodQuery {
    pub method: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventQuery {
    pub location: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FacilityQuery {
    pub facility: Option<String>,
}
