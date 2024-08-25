CREATE TABLE IF NOT EXISTS anime (
    id              INT PRIMARY KEY,
    title           VARCHAR(255) NOT NULL,
    description     TEXT NOT NULL,
    mal_id          INT NOT NULL,
    al_id           INT NOT NULL,
    japanese_title  VARCHAR(255),
    synonyms        VARCHAR(255),
    image           VARCHAR(100) NOT NULL,
    category        VARCHAR(50) NOT NULL,
    rating          VARCHAR(50) NOT NULL,
    quality         VARCHAR(50) NOT NULL,
    duration        VARCHAR(50) NOT NULL,
    premiered       VARCHAR(100) NOT NULL,
    aired           VARCHAR(100) NOT NULL,
    status          VARCHAR(50) NOT NULL,
    mal_score       VARCHAR(50) NOT NULL,
    studios         TEXT NOT NULL,
    producers       TEXT NOT NULL,
    genres          TEXT NOT NULL,
    sub_episodes    INT NOT NULL,
    dub_episodes    INT NOT NULL,
    total_episodes  INT NOT NULL,
    sub_or_dub      VARCHAR(50) NOT NULL
);

CREATE TABLE IF NOT EXISTS anime_id (
    id             INT PRIMARY KEY,
    anime_name     VARCHAR(255) UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS episodes (
    id          VARCHAR(255) PRIMARY KEY,
    episode_no  INT NOT NULL,
    title       VARCHAR(255) NOT NULL,
    is_filler   BOOLEAN NOT NULL,
    anime_id    INT NOT NULL,
    FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS staff (
    mal_id      INT PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    mal_url     VARCHAR(255) NOT NULL,
    image       VARCHAR(200) NOT NULL,
    positions   TEXT[] NOT NULL
);

CREATE TABLE IF NOT EXISTS anime_staff (
    anime_id    INT NOT NULL,
    staff_id    INT NOT NULL,
    positions   TEXT[] NOT NULL,
    PRIMARY KEY (anime_id, staff_id),
    FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE CASCADE,
    FOREIGN KEY (staff_id) REFERENCES staff(mal_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS anime_promo (
    anime_id    INT NOT NULL,
    title       VARCHAR(255) NOT NULL,
    youtube_id  VARCHAR(200) NOT NULL,
    PRIMARY KEY (anime_id, youtube_id),
    FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_anime_mal_id ON anime (mal_id);
CREATE INDEX IF NOT EXISTS idx_episodes_anime_id ON episodes (anime_id);
CREATE INDEX IF NOT EXISTS idx_anime_staff_anime_id ON anime_staff (anime_id);
CREATE INDEX IF NOT EXISTS idx_anime_staff_staff_id ON anime_staff (staff_id);
