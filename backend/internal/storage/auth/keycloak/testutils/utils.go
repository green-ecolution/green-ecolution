package testutils

import (
	"context"
	"fmt"
	"testing"

	"github.com/Nerzal/gocloak/v13"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
)

func (s *KeycloakTestSuite) LoginAdminAndGetToken(t testing.TB) *gocloak.JWT {
	t.Helper()
	identityConfig := s.IdentityConfig(t, context.Background())
	client := gocloak.NewClient(identityConfig.OidcProvider.BaseURL)
	token, err := client.Login(context.Background(), identityConfig.OidcProvider.Backend.ClientID, identityConfig.OidcProvider.Backend.ClientSecret, identityConfig.OidcProvider.DomainName, s.User, s.Password)
	if err != nil {
		t.Fatalf("loginAdminAndGetToken::failed to get token: %v", err)
	}
	return token
}

func (s *KeycloakTestSuite) LoginUser(t testing.TB, user *entities.User) *gocloak.JWT {
	t.Helper()
	identityConfig := s.IdentityConfig(t, context.Background())
	client := gocloak.NewClient(identityConfig.OidcProvider.BaseURL)
	token, err := client.Login(context.Background(), identityConfig.OidcProvider.Frontend.ClientID, identityConfig.OidcProvider.Frontend.ClientSecret, identityConfig.OidcProvider.DomainName, user.Username, "test")
	if err != nil {
		t.Fatalf("loginUser::failed to get token: %v", err)
	}

	return token
}

func (s *KeycloakTestSuite) EnsureUserExists(t testing.TB, user *entities.User) string {
	t.Helper()
	identityConfig := s.IdentityConfig(t, context.Background())
	client := gocloak.NewClient(identityConfig.OidcProvider.BaseURL)
	token, err := client.LoginClient(context.Background(), identityConfig.OidcProvider.Backend.ClientID, identityConfig.OidcProvider.Backend.ClientSecret, identityConfig.OidcProvider.DomainName)
	if err != nil {
		t.Fatalf("ensureUserExists::failed to get token: %v", err)
	}

	attribute := make(map[string][]string)
	attribute["phone_number"] = []string{user.PhoneNumber}
	attribute["employee_id"] = []string{user.EmployeeID}

	kcUser := gocloak.User{
		ID:         gocloak.StringP(user.ID.String()),
		Username:   gocloak.StringP(user.Username),
		FirstName:  gocloak.StringP(user.FirstName),
		LastName:   gocloak.StringP(user.LastName),
		Email:      gocloak.StringP(user.Email),
		Enabled:    gocloak.BoolP(true),
		Attributes: &attribute,
	}

	userID, err := client.CreateUser(context.Background(), token.AccessToken, identityConfig.OidcProvider.DomainName, kcUser)
	if err != nil {
		t.Log("ensureUserExists::failed to create user. maybe user already exists. error: ", err)
	}

	if err = client.SetPassword(context.Background(), token.AccessToken, userID, identityConfig.OidcProvider.DomainName, "test", false); err != nil {
		t.Fatalf("ensureUserExists::failed to set password: %v", err)
	}

	return userID
}

func (s *KeycloakTestSuite) TestUserToCreateFunc() []*entities.User {
	n := 20
	users := make([]*entities.User, n)
	for i := 0; i < n; i++ {
		users[i] = &entities.User{
			Username:    fmt.Sprintf("test%d", i),
			FirstName:   "Toni",
			LastName:    "Tester",
			Email:       fmt.Sprintf("test%d@green-ecolution.de", i),
			EmployeeID:  "123456",
			PhoneNumber: "+49 123456",
		}
	}
	return users
}
