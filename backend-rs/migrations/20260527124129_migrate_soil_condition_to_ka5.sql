-- Migrate tree_soil_condition from 4 coarse textures to the 31 KA5 Feinbodenarten
-- + 3 Reinsande (Bodenkundliche Kartieranleitung, 5. Aufl.). Existing coarse
-- values are remapped to a representative KA5 code per Hauptbodenart.
ALTER TYPE tree_soil_condition RENAME TO tree_soil_condition_old;

CREATE TYPE tree_soil_condition AS ENUM (
  'Ss','Sl2','Sl3','Sl4','Slu','St2','St3','Su2','Su3','Su4',
  'Ls2','Ls3','Ls4','Lt2','Lt3','Lts','Lu',
  'Uu','Uls','Us','Ut2','Ut3','Ut4',
  'Tt','Tl','Tu2','Tu3','Tu4','Ts2','Ts3','Ts4',
  'fS','mS','gS',
  'unknown'
);

ALTER TABLE tree_clusters ALTER COLUMN soil_condition DROP DEFAULT;
ALTER TABLE tree_clusters
  ALTER COLUMN soil_condition TYPE tree_soil_condition
  USING (CASE soil_condition::text
    WHEN 'sandig'    THEN 'Ss'
    WHEN 'schluffig' THEN 'Uu'
    WHEN 'lehmig'    THEN 'Lu'
    WHEN 'tonig'     THEN 'Tt'
    ELSE 'unknown'
  END::tree_soil_condition);
ALTER TABLE tree_clusters ALTER COLUMN soil_condition SET DEFAULT 'unknown';

DROP TYPE tree_soil_condition_old;
