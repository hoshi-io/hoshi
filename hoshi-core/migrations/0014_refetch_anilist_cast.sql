-- AniList cast used to be capped at 6 main characters / 8 staff. Clearing it makes the
-- content page fetch the full cast the next time each title is opened.
UPDATE metadata SET characters = '[]', staff = '[]' WHERE source_name = 'anilist';
