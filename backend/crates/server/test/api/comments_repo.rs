use domain::{
    Id,
    comment::{CommentBody, CommentDraft, CommentReader, CommentSubject, CommentWriter},
    shared::pagination::Pagination,
};
use server::infra::pg_comment::PgCommentRepository;
use uuid::Uuid;

use crate::helpers::spawn_app;

fn draft(subject: CommentSubject, author: Uuid, body: &str) -> CommentDraft {
    CommentDraft {
        subject,
        author_id: author,
        body: CommentBody::new(body).unwrap(),
    }
}

#[tokio::test]
async fn saves_and_lists_newest_first() {
    let app = spawn_app().await;
    let repo = PgCommentRepository::new(app.db_pool.clone());
    let subject = CommentSubject::TreeCluster(Id::new_v7());
    let author = Uuid::new_v4();

    repo.save_new(draft(subject, author, "erster"))
        .await
        .unwrap();
    repo.save_new(draft(subject, author, "zweiter"))
        .await
        .unwrap();

    let page = repo
        .list_for_subject(subject, Pagination::new(1, 25))
        .await
        .unwrap();

    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].body, "zweiter");
    assert_eq!(page.items[1].body, "erster");
    assert_eq!(page.items[0].author_id, author);
    assert_eq!(page.items[0].subject, subject);
}

#[tokio::test]
async fn list_is_scoped_to_the_subject() {
    let app = spawn_app().await;
    let repo = PgCommentRepository::new(app.db_pool.clone());
    let cluster = CommentSubject::TreeCluster(Id::new_v7());
    let plan = CommentSubject::WateringPlan(Id::new_v7());

    repo.save_new(draft(cluster, Uuid::new_v4(), "an der gruppe"))
        .await
        .unwrap();
    repo.save_new(draft(plan, Uuid::new_v4(), "am plan"))
        .await
        .unwrap();

    let cluster_page = repo
        .list_for_subject(cluster, Pagination::new(1, 25))
        .await
        .unwrap();
    assert_eq!(cluster_page.total, 1);
    assert_eq!(cluster_page.items[0].body, "an der gruppe");

    let plan_page = repo
        .list_for_subject(plan, Pagination::new(1, 25))
        .await
        .unwrap();
    assert_eq!(plan_page.total, 1);
    assert_eq!(plan_page.items[0].body, "am plan");
}

#[tokio::test]
async fn list_paginates() {
    let app = spawn_app().await;
    let repo = PgCommentRepository::new(app.db_pool.clone());
    let subject = CommentSubject::WateringPlan(Id::new_v7());
    for i in 0..3 {
        repo.save_new(draft(subject, Uuid::new_v4(), &format!("nr {i}")))
            .await
            .unwrap();
    }

    let page = repo
        .list_for_subject(subject, Pagination::new(2, 2))
        .await
        .unwrap();

    assert_eq!(page.total, 3);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].body, "nr 0");
}

#[tokio::test]
async fn by_id_returns_the_aggregate_and_not_found_after_delete() {
    let app = spawn_app().await;
    let repo = PgCommentRepository::new(app.db_pool.clone());
    let subject = CommentSubject::TreeCluster(Id::new_v7());
    let author = Uuid::new_v4();

    let created = repo
        .save_new(draft(subject, author, "zu löschen"))
        .await
        .unwrap();

    let loaded = repo.by_id(created.id).await.unwrap();
    assert_eq!(loaded.author_id, author);
    assert_eq!(loaded.subject, subject);
    assert_eq!(loaded.body.as_str(), "zu löschen");

    repo.delete(created.id).await.unwrap();
    assert!(repo.by_id(created.id).await.is_err());
    assert!(repo.delete(created.id).await.is_err());
}

#[tokio::test]
async fn delete_for_subject_removes_only_that_subject() {
    let app = spawn_app().await;
    let repo = PgCommentRepository::new(app.db_pool.clone());
    let cluster = CommentSubject::TreeCluster(Id::new_v7());
    let plan = CommentSubject::WateringPlan(Id::new_v7());

    repo.save_new(draft(cluster, Uuid::new_v4(), "a"))
        .await
        .unwrap();
    repo.save_new(draft(cluster, Uuid::new_v4(), "b"))
        .await
        .unwrap();
    repo.save_new(draft(plan, Uuid::new_v4(), "c"))
        .await
        .unwrap();

    let removed = repo.delete_for_subject(cluster).await.unwrap();
    assert_eq!(removed, 2);

    assert_eq!(
        repo.list_for_subject(cluster, Pagination::new(1, 25))
            .await
            .unwrap()
            .total,
        0
    );
    assert_eq!(
        repo.list_for_subject(plan, Pagination::new(1, 25))
            .await
            .unwrap()
            .total,
        1
    );
}
