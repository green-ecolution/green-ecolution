package keycloak

import (
	"context"
	"os"
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/storage/auth/keycloak/testutils"
	"github.com/stretchr/testify/assert"
)

var (
	suite *testutils.KeycloakTestSuite
)

func TestMain(m *testing.M) {
	code := 1
	ctx := context.Background()
	defer func() { os.Exit(code) }()
	suite = testutils.SetupKeycloakTestSuite(ctx)
	defer suite.Terminate(ctx)

	code = m.Run()
}

func TestKeycloakRepository_AuthClient(t *testing.T) {
	t.Run("should login to keycloak successfully with backend client", func(t *testing.T) {
		// given
		ctx := context.Background()
		baseURL := suite.GetAuthServerURL(t, ctx)
		backendClient := suite.GetBackendClient(t, ctx)
		assert.NotNil(t, backendClient)
		assert.NotNil(t, backendClient.ClientID)
		assert.NotNil(t, backendClient.Secret)

		// when
		client, token, err := loginRestAPIClient(ctx, baseURL, *backendClient.ClientID, *backendClient.Secret, suite.RealmName)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, client)
		assert.NotNil(t, token)
	})

	t.Run("should return error when login to keycloak with frontend client", func(t *testing.T) {
		// given
		ctx := context.Background()
		baseURL := suite.GetAuthServerURL(t, ctx)
		frontendClient := suite.GetFrontendClient(t, ctx)
		assert.NotNil(t, frontendClient)
		assert.NotNil(t, frontendClient.ClientID)
		assert.NotNil(t, frontendClient.Secret)

		// when
		client, token, err := loginRestAPIClient(ctx, baseURL, *frontendClient.ClientID, *frontendClient.Secret, suite.RealmName)

		// then
		assert.Error(t, err)
		assert.Nil(t, client)
		assert.Nil(t, token)
	})

	t.Run("should return error when login to keycloak failed", func(t *testing.T) {
		// given
		ctx := context.Background()
		baseURL := suite.GetAuthServerURL(t, ctx)
		backendClient := suite.GetBackendClient(t, ctx)
		assert.NotNil(t, backendClient)
		assert.NotNil(t, backendClient.ClientID)
		assert.NotNil(t, backendClient.Secret)

		// when
		client, token, err := loginRestAPIClient(ctx, baseURL, *backendClient.ClientID, "invalid-secret", suite.RealmName)

		// then
		assert.Error(t, err)
		assert.Nil(t, client)
		assert.Nil(t, token)
	})
}
