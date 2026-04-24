package user

import "context"

type UserRepository interface {
	Create(ctx context.Context, user *User, password string, roles []string) (*User, error)
	RemoveSession(ctx context.Context, token string) error
	GetAll(ctx context.Context) ([]*User, error)
	GetAllByRole(ctx context.Context, role UserRole) ([]*User, error)
	GetByIDs(ctx context.Context, ids []string) ([]*User, error)
}
