-- +goose Up
-- +goose StatementBegin
DROP TRIGGER IF EXISTS update_images_updated_at ON images;
DROP TABLE IF EXISTS tree_images;
DROP TABLE IF EXISTS images;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
CREATE TABLE IF NOT EXISTS images (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  url TEXT NOT NULL,
  filename TEXT,
  mime_type TEXT
);

CREATE TABLE IF NOT EXISTS tree_images (
  tree_id INT,
  image_id INT,
  PRIMARY KEY (tree_id, image_id),
  FOREIGN KEY (tree_id) REFERENCES trees(id),
  FOREIGN KEY (image_id) REFERENCES images(id)
);

CREATE TRIGGER update_images_updated_at
BEFORE UPDATE ON images
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
-- +goose StatementEnd
