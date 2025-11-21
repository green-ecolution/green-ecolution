package keycloak

import (
	"context"
	"errors"

	"github.com/Nerzal/gocloak/v13"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
)

var (
	ErrInvalidToken          = errors.New("invalid token")
	ErrTokenExpired          = errors.New("token expired")
	ErrTokenNotActive        = errors.New("token not active")
	ErrTokenInvalidType      = errors.New("token invalid type")
	ErrLogin                 = errors.New("failed to login to keycloak")
	ErrLogout                = errors.New("failed to logout")
	ErrEmptyUser             = errors.New("user is nil")
	ErrGetUser               = errors.New("failed to get user")
	ErrCreateUser            = errors.New("failed to create user")
	ErrSetPassword           = errors.New("failed to set password")
	ErrGetRole               = errors.New("failed to get role by name")
	ErrSetRole               = errors.New("failed to assign role")
	ErrParseID               = errors.New("failed to parse user id")
	ErrUserWithNilAttributes = errors.New("user with empty attributes")
)

type KeycloakRepository struct {
	cfg *config.IdentityAuthConfig
}

func NewKeycloakRepository(cfg *config.IdentityAuthConfig) storage.AuthRepository {
	return &KeycloakRepository{
		cfg: cfg,
	}
}

// loginRestAPIClient creates and authenticates a GoCloak client to interact with a Keycloak REST API.
//
// Parameters:
//   - ctx: The context for the request, used to cancel or track the operation.
//   - baseURL: The base URL of the Keycloak server, e.g., "https://keycloak.example.com".
//   - clientID: The client ID of the Keycloak client used for authentication.
//   - clientSecret: The secret key of the Keycloak client used for authentication.
//   - realm: The Keycloak realm where the client is registered.
//
// Returns:
//   - client: An initialized and authenticated GoCloak client that can be used for further Keycloak API interactions.
//   - token: A JWT (JSON Web Token) representing the client’s authentication. This token can be used for additional API requests.
//   - err: An error, if the authentication fails or another issue occurs.
//
// Errors:
//   - Returns `ErrLogin` combined with the original error if authentication fails.
//
// Example:
//
//	ctx := context.Background()
//	baseURL := "https://keycloak.example.com"
//	clientID := "my-client-id"
//	clientSecret := "my-client-secret"
//	realm := "my-realm"
//
//	client, token, err := loginRestAPIClient(ctx, baseURL, clientID, clientSecret, realm)
//	if err != nil {
//	    log.Fatalf("Failed to authenticate: %v", err)
//	}
//
//	fmt.Printf("Access Token: %s\n", token.AccessToken)
func loginRestAPIClient(ctx context.Context, baseURL, clientID, clientSecret, realm string) (client *gocloak.GoCloak, token *gocloak.JWT, err error) {
	log := logger.GetLogger(ctx)
	client = gocloak.NewClient(baseURL)

	token, err = client.LoginClient(ctx, clientID, clientSecret, realm)
	if err != nil {
		log.Error("failed to generate keycloak client", "error", err, "client_id", clientID, "client_secret", "*******", "realm", realm)
		return nil, nil, errors.Join(err, ErrLogin)
	}

	return
}
