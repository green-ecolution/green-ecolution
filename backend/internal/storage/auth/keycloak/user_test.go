package keycloak

import (
	"context"
	"testing"
	"time"

	"github.com/Nerzal/gocloak/v13"
	"github.com/google/uuid"
	"github.com/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/stretchr/testify/assert"
)

var testUser = suite.TestUserToCreateFunc()

func TestKeyCloakUserRepo_Create(t *testing.T) {
	type fields struct {
		cfg *config.IdentityAuthConfig
	}
	type args struct {
		ctx      context.Context
		user     *entities.User
		password string
		roles    []string
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    *entities.User
		wantErr bool
	}{
		{
			name: "should create user successfully",
			fields: fields{
				cfg: suite.IdentityConfig(t, context.Background()),
			},
			args: args{
				ctx:      context.Background(),
				user:     testUser[0],
				password: "test",
				roles:    []string{},
			},
			wantErr: false,
		},
		{
			name: "should return error when create with same email",
			fields: fields{
				cfg: suite.IdentityConfig(t, context.Background()),
			},
			args: args{
				ctx:      context.Background(),
				user:     testUser[0],
				password: "test",
				roles:    []string{},
			},
			wantErr: true,
		},
		{
			name: "should return error when create with same username",
			fields: fields{
				cfg: suite.IdentityConfig(t, context.Background()),
			},
			args: args{
				ctx: context.Background(),
				user: &entities.User{
					Username:    testUser[0].Username,
					FirstName:   testUser[1].FirstName,
					LastName:    testUser[1].LastName,
					Email:       testUser[1].Email,
					EmployeeID:  testUser[1].EmployeeID,
					PhoneNumber: testUser[1].PhoneNumber,
				},
				password: "test",
				roles:    []string{},
			},
			wantErr: true,
		},
		{
			name: "should create user successfully with role",
			fields: fields{
				cfg: suite.IdentityConfig(t, context.Background()),
			},
			args: args{
				ctx:      context.Background(),
				user:     testUser[2],
				password: "test",
				roles:    []string{"admin"},
			},
			wantErr: false,
		},
		{
			name: "should create user successfully when roles are nil",
			fields: fields{
				cfg: suite.IdentityConfig(t, context.Background()),
			},
			args: args{
				ctx:      context.Background(),
				user:     testUser[3],
				password: "test",
				roles:    nil,
			},
			wantErr: false,
		},
		{
			name: "should return error when user is nil",
			fields: fields{
				cfg: suite.IdentityConfig(t, context.Background()),
			},
			args: args{
				ctx:      context.Background(),
				user:     nil,
				password: "test",
				roles:    nil,
			},
			wantErr: true,
		},
		{
			name: "should return error when role not exist",
			fields: fields{
				cfg: suite.IdentityConfig(t, context.Background()),
			},
			args: args{
				ctx:      context.Background(),
				user:     testUser[4],
				password: "test",
				roles:    []string{"not-exist"},
			},
			wantErr: true,
		},
		{
			name: "should return error when failed to set password",
			fields: fields{
				cfg: suite.IdentityConfig(t, context.Background()),
			},
			args: args{
				ctx:      context.Background(),
				user:     testUser[5],
				password: "",
				roles:    nil,
			},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &UserRepository{
				cfg: tt.fields.cfg,
			}
			got, err := r.Create(tt.args.ctx, tt.args.user, tt.args.password, tt.args.roles)
			if tt.wantErr {
				assert.Error(t, err)
				assert.Nil(t, got)
			} else {
				assert.NoError(t, err)
				assert.NotNil(t, got)
				assert.Equal(t, tt.args.user.Username, got.Username)
				assert.Equal(t, tt.args.user.FirstName, got.FirstName)
				assert.Equal(t, tt.args.user.LastName, got.LastName)
				assert.Equal(t, tt.args.user.Email, got.Email)
				assert.Equal(t, tt.args.user.EmployeeID, got.EmployeeID)
				assert.Equal(t, tt.args.user.PhoneNumber, got.PhoneNumber)
				assert.NotNil(t, got.ID)
				assert.NotZero(t, got.ID)
				assert.NotZero(t, got.CreatedAt)
			}
		})
	}
}

func TestKeyCloakUserRepo_RemoveSession(t *testing.T) {
	t.Run("should remove session successfully", func(t *testing.T) {
		// given
		identityConfig := suite.IdentityConfig(t, context.Background())
		userRepo := NewUserRepository(identityConfig)
		user := &entities.User{
			Username:    "should-remove-session",
			FirstName:   "Toni",
			LastName:    "Tester",
			Email:       "should-remove-session@green-ecolution.de",
			EmployeeID:  "123456",
			PhoneNumber: "+49 123456",
		}
		suite.EnsureUserExists(t, user)
		token := suite.LoginUser(t, user)

		// when
		err := userRepo.RemoveSession(context.Background(), token.RefreshToken)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error when failed to remove session", func(t *testing.T) {
		// given
		identityConfig := &config.IdentityAuthConfig{}
		userRepo := NewUserRepository(identityConfig)

		// when
		err := userRepo.RemoveSession(context.Background(), "invalid-token")

		// then
		assert.Error(t, err)
	})
}

func TestKeyCloakUserRepo_KeyCloakUserToUser(t *testing.T) {
	t.Run("should convert keycloak user to user successfully", func(t *testing.T) {
		// given
		uuid, _ := uuid.NewRandom()
		user := &gocloak.User{
			ID:               gocloak.StringP(uuid.String()),
			CreatedTimestamp: gocloak.Int64P(123456),
			Username:         gocloak.StringP("test"),
			FirstName:        gocloak.StringP("Toni"),
			LastName:         gocloak.StringP("Tester"),
			Email:            gocloak.StringP("dev@green-ecolution.de"),
			Attributes: &map[string][]string{
				"phone_number": {"+49 123456"},
				"employee_id":  {"123456"},
			},
		}

		// when
		got, err := keyCloakUserToUser(context.Background(), user)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, uuid.String(), got.ID.String())
		assert.Equal(t, "test", got.Username)
		assert.Equal(t, "Toni", got.FirstName)
		assert.Equal(t, "Tester", got.LastName)
		assert.Equal(t, "dev@green-ecolution.de", got.Email)
		assert.Equal(t, "+49 123456", got.PhoneNumber)
		assert.Equal(t, "123456", got.EmployeeID)
	})

	t.Run("should return error when failed to parse user id", func(t *testing.T) {
		// given
		user := &gocloak.User{
			ID:               gocloak.StringP("invalid-id"),
			CreatedTimestamp: gocloak.Int64P(123456),
			Username:         gocloak.StringP("test"),
			FirstName:        gocloak.StringP("Toni"),
			LastName:         gocloak.StringP("Tester"),
			Email:            gocloak.StringP("dev@green-ecolution.de"),
			Attributes: &map[string][]string{
				"phone_number": {"+49 123456"},
				"employee_id":  {"123456"},
			},
		}

		// when
		got, err := keyCloakUserToUser(context.Background(), user)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestKeyCloakUserRepo_UserToKeyCloakUser(t *testing.T) {
	t.Run("should convert user to keycloak user successfully", func(t *testing.T) {
		// given
		uuid, _ := uuid.NewRandom()
		user := &entities.User{
			ID:          uuid,
			CreatedAt:   time.Unix(123456, 0),
			Username:    "test",
			FirstName:   "Toni",
			LastName:    "Tester",
			Email:       "dev@green-ecolution.de",
			PhoneNumber: "+49 123456",
			EmployeeID:  "123456",
		}

		// when
		got := userToKeyCloakUser(user)

		// then
		assert.NotNil(t, got)
		assert.NotNil(t, got.ID)
		assert.NotNil(t, got.Username)
		assert.NotNil(t, got.FirstName)
		assert.NotNil(t, got.LastName)
		assert.NotNil(t, got.Email)
		assert.NotNil(t, got.Attributes)

		assert.Equal(t, uuid.String(), *got.ID)
		assert.Equal(t, "test", *got.Username)
		assert.Equal(t, "Toni", *got.FirstName)
		assert.Equal(t, "Tester", *got.LastName)
		assert.Equal(t, "dev@green-ecolution.de", *got.Email)
		assert.Equal(t, "+49 123456", (*got.Attributes)["phone_number"][0])
		assert.Equal(t, "123456", (*got.Attributes)["employee_id"][0])
	})
}

func TestKeyCloakUserRepo_GetAll(t *testing.T) {
	t.Run("should get all users successfully", func(t *testing.T) {
		// given
		identityConfig := suite.IdentityConfig(t, context.Background())
		userRepo := NewUserRepository(identityConfig)
		user1 := &entities.User{
			Username:    "user1",
			FirstName:   "John",
			LastName:    "Doe",
			Email:       "john.doe@green-ecolution.de",
			EmployeeID:  "EMP001",
			PhoneNumber: "+49 123456789",
		}
		user2 := &entities.User{
			Username:    "user2",
			FirstName:   "Jane",
			LastName:    "Doe",
			Email:       "jane.doe@green-ecolution.de",
			EmployeeID:  "EMP002",
			PhoneNumber: "+49 987654321",
		}

		suite.EnsureUserExists(t, user1)
		suite.EnsureUserExists(t, user2)

		// when
		users, err := userRepo.GetAll(context.Background())

		// then
		assert.NoError(t, err)
		assert.NotNil(t, users)
		assert.GreaterOrEqual(t, len(users), 2)
		assert.True(t, containsUser(users, *user1), "user1 should be in the list")
		assert.True(t, containsUser(users, *user2), "user2 should be in the list")
	})

	t.Run("should return error when login fails", func(t *testing.T) {
		// given
		identityConfig := suite.InvalidIdentityConfig(t, context.Background())
		userRepo := NewUserRepository(identityConfig)

		// when
		users, err := userRepo.GetAll(context.Background())

		// then
		assert.Error(t, err)
		assert.ErrorIs(t, err, ErrLogin)
		assert.Nil(t, users)
	})
}

func TestKeyCloakUserRepo_GetAllByRole(t *testing.T) {
	t.Run("should get all users by role successfully", func(t *testing.T) {
		// given
		identityConfig := suite.IdentityConfig(t, context.Background())
		userRepo := NewUserRepository(identityConfig)
		user3 := &entities.User{
			Username:    "user3",
			FirstName:   "John",
			LastName:    "Doe",
			Email:       "user3@green-ecolution.de",
			EmployeeID:  "EMP001",
			PhoneNumber: "+49 123456789",
			Roles:       []entities.UserRole{entities.UserRoleSmarteGrenzregion},
		}
		user4 := &entities.User{
			Username:    "user4",
			FirstName:   "Jane",
			LastName:    "Doe",
			Email:       "user4@green-ecolution.de",
			EmployeeID:  "EMP002",
			PhoneNumber: "+49 987654321",
			Roles:       []entities.UserRole{entities.UserRoleTbz},
		}

		suite.EnsureUserExists(t, user3)
		suite.EnsureUserExists(t, user4)

		// when
		users, err := userRepo.GetAllByRole(context.Background(), entities.UserRoleGreenEcolution)

		// then
		assert.NoError(t, err)
		assert.Empty(t, users)
		assert.GreaterOrEqual(t, len(users), 0)
		assert.False(t, containsUser(users, *user3), "user3 should not be in the list")
		assert.False(t, containsUser(users, *user4), "user4 should not be in the list")
	})

	t.Run("should return error when login fails", func(t *testing.T) {
		// given
		identityConfig := suite.InvalidIdentityConfig(t, context.Background())
		userRepo := NewUserRepository(identityConfig)

		// when
		users, err := userRepo.GetAllByRole(context.Background(), entities.UserRoleGreenEcolution)

		// then
		assert.Error(t, err)
		assert.Nil(t, users)
	})
}

func TestKeyCloakUserRepo_GetByID(t *testing.T) {
	t.Run("should get user by id successfully", func(t *testing.T) {
		// given
		identityConfig := suite.IdentityConfig(t, context.Background())
		userRepo := NewUserRepository(identityConfig)

		user := &entities.User{
			CreatedAt:   time.Unix(123456, 0),
			Username:    "test02",
			FirstName:   "Toni",
			LastName:    "Tester",
			Email:       "test02@green-ecolution.de",
			PhoneNumber: "+49 123456",
			EmployeeID:  "123456",
		}

		userID := suite.EnsureUserExists(t, user)

		// when
		users, err := userRepo.GetByIDs(context.Background(), []string{userID})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, users)
		assert.Equal(t, 1, len(users))
		for i, user := range users {
			assert.Equal(t, users[i].ID, user.ID)
			assert.Equal(t, users[i].Email, user.Email)
			assert.Equal(t, users[i].Username, user.Username)
			assert.Equal(t, users[i].FirstName, user.FirstName)
			assert.Equal(t, users[i].LastName, user.LastName)
			assert.Equal(t, users[i].PhoneNumber, user.PhoneNumber)
		}
	})

	t.Run("should return error when login fails", func(t *testing.T) {
		// given
		identityConfig := suite.InvalidIdentityConfig(t, context.Background())
		userRepo := NewUserRepository(identityConfig)

		uuid, _ := uuid.NewRandom()

		// when
		users, err := userRepo.GetByIDs(context.Background(), []string{uuid.String()})

		// then
		assert.Error(t, err)
		assert.ErrorIs(t, err, ErrLogin)
		assert.Nil(t, users)
	})
}

func containsUser(userList []*entities.User, targetUser entities.User) bool {
	for _, user := range userList {
		if user.Username == targetUser.Username &&
			user.FirstName == targetUser.FirstName &&
			user.LastName == targetUser.LastName &&
			user.Email == targetUser.Email &&
			user.EmployeeID == targetUser.EmployeeID &&
			user.PhoneNumber == targetUser.PhoneNumber {
			return true
		}
	}
	return false
}
