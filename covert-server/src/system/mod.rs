mod entity;
mod initialize;
mod lease;
mod mount;
mod policy;
mod seal;
mod status;
mod token;
mod unseal;

use std::sync::Arc;

use covert_framework::{
    create, create_with_config, delete, extract::Extension, read, read_with_config, revoke, update,
    Backend, RouteConfig, Router,
};
use covert_types::{
    auth::AuthPolicy,
    backend::{BackendCategory, BackendType},
    state::VaultState,
};

use crate::{
    store::{identity_store::IdentityStore, policy_store::PolicyStore, token_store::TokenStore},
    ExpirationManager,
};

use self::{
    entity::{
        handle_attach_entity_alias, handle_attach_entity_policy, handle_entity_create,
        handle_remove_entity_alias, handle_remove_entity_policy,
    },
    initialize::handle_initialize,
    lease::{
        handle_lease_lookup, handle_lease_renew, handle_lease_revocation,
        handle_lease_revocation_by_mount, handle_list_leases,
    },
    mount::{handle_mount, handle_mount_disable, handle_mounts_list, handle_update_mount},
    policy::{handle_create_policy, handle_delete_policy, handle_list_policies},
    seal::handle_seal,
    status::handle_status,
    token::handle_token_revocation,
    unseal::handle_unseal,
};
pub use token::RevokeTokenParams;

pub fn new_system_backend(
    token_store: Arc<TokenStore>,
    policy_store: Arc<PolicyStore>,
    identity_store: Arc<IdentityStore>,
    expiration_manager: Arc<ExpirationManager>,
) -> Backend {
    let router = Router::new()
        .route(
            "/unseal",
            create_with_config(
                handle_unseal,
                RouteConfig {
                    policy: AuthPolicy::Unauthenticated,
                    state: vec![VaultState::Sealed],
                },
            )
            .update_with_config(
                handle_unseal,
                RouteConfig {
                    policy: AuthPolicy::Unauthenticated,
                    state: vec![VaultState::Sealed],
                },
            ),
        )
        .route(
            "/seal",
            create_with_config(
                handle_seal,
                RouteConfig {
                    policy: AuthPolicy::Unauthenticated,
                    state: vec![VaultState::Unsealed],
                },
            )
            .update_with_config(
                handle_seal,
                RouteConfig {
                    policy: AuthPolicy::Unauthenticated,
                    state: vec![VaultState::Unsealed],
                },
            ),
        )
        .route(
            "/init",
            create_with_config(
                handle_initialize,
                RouteConfig {
                    policy: AuthPolicy::Unauthenticated,
                    state: vec![VaultState::Uninitialized],
                },
            )
            .update_with_config(
                handle_initialize,
                RouteConfig {
                    policy: AuthPolicy::Unauthenticated,
                    state: vec![VaultState::Uninitialized],
                },
            ),
        )
        .route(
            "/status",
            read_with_config(
                handle_status,
                RouteConfig {
                    policy: AuthPolicy::Unauthenticated,
                    state: vec![
                        VaultState::Uninitialized,
                        VaultState::Sealed,
                        VaultState::Unsealed,
                    ],
                },
            ),
        )
        .route("/mounts", read(handle_mounts_list))
        .route(
            "/mounts/*path",
            create(handle_mount)
                .update(handle_update_mount)
                .delete(handle_mount_disable),
        )
        .route(
            "/policies",
            update(handle_create_policy)
                .create(handle_create_policy)
                .read(handle_list_policies),
        )
        .route("/policies/*name", delete(handle_delete_policy))
        .route("/token/revoke", revoke(handle_token_revocation))
        .route("/leases/revoke/*lease_id", update(handle_lease_revocation))
        .route("/leases/renew/*lease_id", update(handle_lease_renew))
        .route("/leases/lookup/*lease_id", read(handle_lease_lookup))
        .route(
            "/leases/revoke-mount/*prefix",
            update(handle_lease_revocation_by_mount),
        )
        .route("/leases/lookup-mount/*prefix", read(handle_list_leases))
        .route("/entity", create(handle_entity_create))
        .route("/entity/policy", update(handle_attach_entity_policy))
        .route("/entity/policy/*name", update(handle_remove_entity_policy))
        .route("/entity/alias", update(handle_attach_entity_alias))
        .route("/entity/alias/*name", update(handle_remove_entity_alias))
        .layer(Extension(expiration_manager))
        .layer(Extension(token_store))
        .layer(Extension(policy_store))
        .layer(Extension(identity_store))
        .build()
        .into_service();

    Backend {
        handler: router,
        category: BackendCategory::Logical,
        variant: BackendType::System,
        migrations: vec![],
    }
}
