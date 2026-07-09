#[derive(Clone)]
pub(crate) struct KeycloakState {
    base_url: String,
    realm: String,
    client_id: String,
    client_secret: String,
}

impl KeycloakState {
    pub fn new(
        base_url: String,
        realm: String,
        client_id: String,
        client_secret: String,
    ) -> KeycloakState {
        KeycloakState {
            base_url,
            realm,
            client_id,
            client_secret,
        }
    }
}