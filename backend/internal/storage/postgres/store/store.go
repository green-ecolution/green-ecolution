package store

import (
	"context"
	"errors"
	"log/slog"
	"reflect"

	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

type Store struct {
	*sqlc.Queries
	db *pgxpool.Pool
}

func NewStore(db *pgxpool.Pool, querier *sqlc.Queries) *Store {
	if db == nil {
		slog.Error("db is nil")
		panic("db is nil")
	}

	if querier == nil {
		slog.Error("querier is nil")
		panic("querier is nil")
	}

	return &Store{
		Queries: querier,
		db:      db,
	}
}

func (s *Store) DB() *pgxpool.Pool {
	return s.db
}

func (s *Store) MapError(err error, dbType any) error {
	if err == nil {
		return nil
	}
	rType := reflect.TypeOf(dbType)
	if rType.Kind() == reflect.Pointer {
		rType = rType.Elem()
	}

	var rName string
	switch rType.Kind() {
	case reflect.Struct:
		rName = rType.Name()
	case reflect.String:
		rName = dbType.(string)
	default:
		panic("unrechable")
	}

	if errors.Is(err, pgx.ErrNoRows) {
		return storage.ErrEntityNotFound(rName)
	}

	return err
}

func (s *Store) WithTx(ctx context.Context, fn func(*Store) error) error {
	log := logger.GetLogger(ctx)
	if fn == nil {
		return errors.New("txFn is nil")
	}

	tx, err := s.db.Begin(ctx)
	if err != nil {
		return err
	}

	qtx := s.Queries.WithTx(tx)

	store := *s
	store.Queries = qtx
	err = fn(&store)
	if err == nil {
		log.Debug("committing transaction")
		return tx.Commit(ctx)
	}

	log.Debug("rolling back transaction")
	rollbackErr := tx.Rollback(ctx)
	if rollbackErr != nil {
		log.Error("error rolling back transaction", "error", rollbackErr)
		return errors.Join(err, rollbackErr)
	}

	return err
}

func (s *Store) Close() {
	s.db.Close()
}
