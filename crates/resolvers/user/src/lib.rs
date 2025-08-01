use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use common_models::StringIdObject;
use database_models::{access_link, integration, notification_platform, user};
use dependent_models::{CachedResponse, UserDetailsResult, UserMetadataRecommendationsResponse};
use media_models::{
    AuthUserInput, CreateAccessLinkInput, CreateOrUpdateUserIntegrationInput,
    CreateUserNotificationPlatformInput, LoginResult, OidcTokenOutput, ProcessAccessLinkInput,
    ProcessAccessLinkResult, RegisterResult, RegisterUserInput,
    UpdateUserNotificationPlatformInput, UserResetResult,
};
use traits::AuthProvider;
use user_models::{UpdateUserInput, UserPreferences};
use user_service::UserService;

#[derive(Default)]
pub struct UserQuery;

impl AuthProvider for UserQuery {}

#[Object]
impl UserQuery {
    /// Get metadata recommendations for the currently logged in user.
    async fn user_metadata_recommendations(
        &self,
        gql_ctx: &Context<'_>,
    ) -> Result<CachedResponse<UserMetadataRecommendationsResponse>> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.user_metadata_recommendations(&user_id).await
    }

    /// Get all access links generated by the currently logged in user.
    async fn user_access_links(&self, gql_ctx: &Context<'_>) -> Result<Vec<access_link::Model>> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.user_access_links(&user_id).await
    }

    /// Get details about all the users in the service.
    async fn users_list(
        &self,
        gql_ctx: &Context<'_>,
        query: Option<String>,
    ) -> Result<Vec<user::Model>> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        service.users_list(query).await
    }

    /// Get details about the currently logged in user.
    async fn user_details(&self, gql_ctx: &Context<'_>) -> Result<UserDetailsResult> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let token = self.user_auth_token_from_ctx(gql_ctx)?;
        service.user_details(&token).await
    }

    /// Get all the integrations for the currently logged in user.
    async fn user_integrations(&self, gql_ctx: &Context<'_>) -> Result<Vec<integration::Model>> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.user_integrations(&user_id).await
    }

    /// Get all the notification platforms for the currently logged in user.
    async fn user_notification_platforms(
        &self,
        gql_ctx: &Context<'_>,
    ) -> Result<Vec<notification_platform::Model>> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.user_notification_platforms(&user_id).await
    }

    /// Get an authorization URL using the configured OIDC client.
    async fn get_oidc_redirect_url(&self, gql_ctx: &Context<'_>) -> Result<String> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        service.get_oidc_redirect_url().await
    }

    /// Get an access token using the configured OIDC client.
    async fn get_oidc_token(&self, gql_ctx: &Context<'_>, code: String) -> Result<OidcTokenOutput> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        service.get_oidc_token(code).await
    }

    /// Get user by OIDC issuer ID.
    async fn user_by_oidc_issuer_id(
        &self,
        gql_ctx: &Context<'_>,
        oidc_issuer_id: String,
    ) -> Result<Option<String>> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        service.user_by_oidc_issuer_id(oidc_issuer_id).await
    }
}

#[derive(Default)]
pub struct UserMutation;

impl AuthProvider for UserMutation {
    fn is_mutation(&self) -> bool {
        true
    }
}

#[Object]
impl UserMutation {
    /// Create or edit an access link.
    async fn create_access_link(
        &self,
        gql_ctx: &Context<'_>,
        input: CreateAccessLinkInput,
    ) -> Result<StringIdObject> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.create_access_link(input, user_id).await
    }

    /// Get an access token using an access link.
    async fn process_access_link(
        &self,
        gql_ctx: &Context<'_>,
        input: ProcessAccessLinkInput,
    ) -> Result<ProcessAccessLinkResult> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        service.process_access_link(input).await
    }

    /// Revoke an access link.
    async fn revoke_access_link(
        &self,
        gql_ctx: &Context<'_>,
        access_link_id: String,
    ) -> Result<bool> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        self.user_id_from_ctx(gql_ctx).await?;
        service.revoke_access_link(access_link_id).await
    }

    /// Delete a user. The account deleting the user must be an `Admin`.
    async fn delete_user(&self, gql_ctx: &Context<'_>, to_delete_user_id: String) -> Result<bool> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.delete_user(user_id, to_delete_user_id).await
    }

    /// Reset a user by deleting and recreating them with the same ID. The account
    /// resetting the user must be an `Admin`.
    async fn reset_user(
        &self,
        gql_ctx: &Context<'_>,
        to_reset_user_id: String,
    ) -> Result<UserResetResult> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.reset_user(user_id, to_reset_user_id).await
    }

    /// Create a new user for the service. Also set their `lot` as admin if
    /// they are the first user.
    async fn register_user(
        &self,
        gql_ctx: &Context<'_>,
        input: RegisterUserInput,
    ) -> Result<RegisterResult> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        service.register_user(input).await
    }

    /// Login a user using their username and password and return an auth token.
    async fn login_user(&self, gql_ctx: &Context<'_>, input: AuthUserInput) -> Result<LoginResult> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        service.login_user(input).await
    }

    /// Update a user's profile details.
    async fn update_user(
        &self,
        gql_ctx: &Context<'_>,
        input: UpdateUserInput,
    ) -> Result<StringIdObject> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await.ok();
        service.update_user(user_id, input).await
    }

    /// Change a user's preferences.
    async fn update_user_preference(
        &self,
        gql_ctx: &Context<'_>,
        input: UserPreferences,
    ) -> Result<bool> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.update_user_preference(user_id, input).await
    }

    /// Create or update an integration for the currently logged in user.
    async fn create_or_update_user_integration(
        &self,
        gql_ctx: &Context<'_>,
        input: CreateOrUpdateUserIntegrationInput,
    ) -> Result<bool> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service
            .create_or_update_user_integration(user_id, input)
            .await
    }

    /// Delete an integration for the currently logged in user.
    async fn delete_user_integration(
        &self,
        gql_ctx: &Context<'_>,
        integration_id: String,
    ) -> Result<bool> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service
            .delete_user_integration(user_id, integration_id)
            .await
    }

    /// Add a notification platform for the currently logged in user.
    async fn create_user_notification_platform(
        &self,
        gql_ctx: &Context<'_>,
        input: CreateUserNotificationPlatformInput,
    ) -> Result<String> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service
            .create_user_notification_platform(user_id, input)
            .await
    }

    /// Edit a notification platform for the currently logged in user.
    async fn update_user_notification_platform(
        &self,
        gql_ctx: &Context<'_>,
        input: UpdateUserNotificationPlatformInput,
    ) -> Result<bool> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service
            .update_user_notification_platform(user_id, input)
            .await
    }

    /// Delete a notification platform for the currently logged in user.
    async fn delete_user_notification_platform(
        &self,
        gql_ctx: &Context<'_>,
        notification_id: String,
    ) -> Result<bool> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service
            .delete_user_notification_platform(user_id, notification_id)
            .await
    }

    /// Test all notification platforms for the currently logged in user.
    async fn test_user_notification_platforms(&self, gql_ctx: &Context<'_>) -> Result<bool> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.test_user_notification_platforms(&user_id).await
    }

    /// Generate an auth token without any expiry.
    async fn generate_auth_token(&self, gql_ctx: &Context<'_>) -> Result<String> {
        let service = gql_ctx.data_unchecked::<Arc<UserService>>();
        let user_id = self.user_id_from_ctx(gql_ctx).await?;
        service.generate_auth_token(user_id).await
    }
}
