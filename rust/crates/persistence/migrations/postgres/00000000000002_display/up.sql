ALTER TABLE users ADD COLUMN language TEXT NOT NULL DEFAULT '' CHECK (language IN ('','en','ko','ja','fr'));
ALTER TABLE users ADD COLUMN time_zone TEXT NOT NULL DEFAULT '';
ALTER TABLE users ADD COLUMN region TEXT NOT NULL DEFAULT '' CHECK (region IN ('','en-US','ko-KR','ja-JP','fr-FR'));
CREATE TABLE display_defaults (
    singleton INTEGER PRIMARY KEY CHECK (singleton=1),
    language TEXT NOT NULL CHECK (language IN ('en','ko','ja','fr')),
    time_zone TEXT NOT NULL CHECK (length(time_zone)>0),
    region TEXT NOT NULL CHECK (region IN ('en-US','ko-KR','ja-JP','fr-FR'))
);
