ALTER TABLE recipes ADD COLUMN added_by_user_id TEXT;

CREATE INDEX IF NOT EXISTS idx_recipes_added_by_user_id ON recipes(added_by_user_id);

