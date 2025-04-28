package testutils

import (
	"database/sql"
	"fmt"
	"log"
	"testing"

	"github.com/green-ecolution/backend/internal/utils"
	"github.com/pressly/goose/v3"
)

func ExecMigration(db *sql.DB) {
	log.Println("Executing migration...")
	rootDir := utils.RootDir()
	migrationPath := fmt.Sprintf("%s/internal/storage/postgres/migrations/", rootDir)

	if err := goose.Up(db, migrationPath); err != nil {
		log.Fatalf("Could not execute migration: %s", err)
	}
}

// ResetMigration resets the database migration
func ResetMigration(db *sql.DB) {
	log.Println("Resetting migration...")

	rootDir := utils.RootDir()
	migrationPath := fmt.Sprintf("%s/internal/storage/postgres/migrations/", rootDir)

	if err := goose.Reset(db, migrationPath); err != nil {
		log.Fatalf("Could not reset migration: %s", err)
	}

	if err := goose.Up(db, migrationPath); err != nil {
		log.Fatalf("Could not execute migration: %s", err)
	}
}

// InsertSeed inserts test seed data
func InsertSeed(db *sql.DB) {
	log.Println("Inserting seed data...")

	rootDir := utils.RootDir()
	seedPath := fmt.Sprintf("%s/internal/storage/postgres/seed/test", rootDir)

	if err := goose.Up(db, seedPath, goose.WithNoVersioning()); err != nil {
		log.Fatalf("Could not insert seed data: %s", err)
	}
}

// InsertSeed inserts test seed data based on the seed path. Seed path is relative to the root directory.
func (s *PostgresTestSuite) InsertSeed(t testing.TB, dir string) {
	t.Helper()
	db, err := sql.Open("pgx", s.URL)
	if err != nil {
		t.Fatalf("Could not connect to postgres: %s", err)
	}
	defer db.Close()

	rootDir := utils.RootDir()
	seedPath := fmt.Sprintf("%s/%s", rootDir, dir)

	if err := goose.Up(db, seedPath, goose.WithNoVersioning()); err != nil {
		t.Fatalf("Could not insert seed data: %s", err)
	}
}
