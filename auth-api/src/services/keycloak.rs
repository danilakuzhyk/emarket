use config_types::keycloak::{BaseUrl, ClientId, ClientSecret, Realm};

#[derive(Clone)]
pub(crate) struct KeycloakState {
    base_url: BaseUrl,
    realm: Realm,
    client_id: ClientId,
    client_secret: ClientSecret,
}

impl KeycloakState {
    pub fn new(
        base_url: BaseUrl,
        realm: Realm,
        client_id: ClientId,
        client_secret: ClientSecret,
    ) -> Self {
        Self {
            base_url,
            realm,
            client_id,
            client_secret,
        }
    }
}
