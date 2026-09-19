use domain::{
    shared::pagination::Pagination,
    tree::{TreeSearchQuery, repository::TreeReader},
};
use server::{
    bench::{
        scale::Scale,
        seed::{SeedPlan, seed_core},
    },
    infra::pg_tree::PgTreeRepository,
};

use crate::helpers::spawn_app;

fn xs_plan() -> SeedPlan {
    SeedPlan {
        scale: Scale::Xs,
        history_days: 2,
        seed: 42,
    }
}

#[tokio::test]
async fn seeding_xs_produces_rows_the_readers_can_consume() {
    let app = spawn_app().await;

    let counts = seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed against a migrated schema");

    assert_eq!(counts.trees, 1_000);
    assert_eq!(counts.clusters, 20);

    let repo = PgTreeRepository::new(app.db_pool.clone());
    let page = repo
        .view_search(TreeSearchQuery::default(), Pagination::new(1, 25))
        .await
        .expect("the seeded rows must be readable through the repository");

    assert_eq!(page.page.items.len(), 25);
    assert!(page.page.total >= 1_000, "got {}", page.page.total);
}

#[tokio::test]
async fn seeded_tree_ids_are_uuid_v7() {
    let app = spawn_app().await;

    seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed");

    // pg_tree.rs derives TreeView::created_at from the id's embedded v7
    // timestamp and `expect`s it to be there. A v4 id would panic every read.
    let non_v7: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM trees WHERE uuid_extract_version(id) IS DISTINCT FROM 7",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("the version probe must run");

    assert_eq!(non_v7, 0, "every seeded tree id must be a uuid v7");
}

#[tokio::test]
async fn seeded_tree_numbers_exercise_the_default_sort() {
    let app = spawn_app().await;

    seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed");

    // The list's default sort splits `number` into a letter prefix and a digit
    // part by regex. Numbers of only one shape would exercise half the path.
    let with_prefix: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trees WHERE number ~ '^[A-Z]'")
        .fetch_one(&app.db_pool)
        .await
        .expect("the prefix probe must run");
    let without_prefix: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM trees WHERE number ~ '^[0-9]'")
            .fetch_one(&app.db_pool)
            .await
            .expect("the digit probe must run");

    assert!(with_prefix > 0, "no lettered tree numbers were seeded");
    assert!(
        without_prefix > 0,
        "no bare numeric tree numbers were seeded"
    );
}

#[tokio::test]
async fn seeding_twice_tops_up_instead_of_duplicating() {
    let app = spawn_app().await;

    seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("first seed must succeed");
    let second = seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("second seed must succeed");

    // The campaign grows through the scales by re-running the seeder, so a
    // repeated call at the same scale has to be a no-op rather than a doubling.
    assert_eq!(second.trees, 1_000);
    assert_eq!(second.clusters, 20);
}
