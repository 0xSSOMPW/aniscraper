-- Drop indexes first
DROP INDEX IF EXISTS idx_anime_mal_id;
DROP INDEX IF EXISTS idx_episodes_anime_id;
DROP INDEX IF EXISTS idx_anime_staff_anime_id;
DROP INDEX IF EXISTS idx_anime_staff_staff_id;

-- Drop the 'anime_promo' table first due to foreign key dependency
DROP TABLE IF EXISTS anime_promo;

-- Drop the 'anime_staff' table
DROP TABLE IF EXISTS anime_staff;

-- Drop the 'staff' table
DROP TABLE IF EXISTS staff;

-- Drop the 'episodes' table
DROP TABLE IF EXISTS episodes;

-- Drop the 'anime_id' table
DROP TABLE IF EXISTS anime_id;

-- Drop the 'anime' table
DROP TABLE IF EXISTS anime;
