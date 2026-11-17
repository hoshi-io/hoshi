-- Score of the related title, so cards for sequels that aren't imported yet can show it.
ALTER TABLE content_relations ADD COLUMN target_rating REAL;
