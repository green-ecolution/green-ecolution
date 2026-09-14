-- Managing a threshold is a different job from renaming, moving and deleting
-- organizations, so the values get their own resource instead of riding on
-- organization:update.
--
-- Only rows that still match a delivered template are touched. A hand-edited
-- role (template_key IS NULL) keeps exactly the permissions someone chose.

UPDATE roles
   SET permissions = permissions || ARRAY['setting:read','setting:update']
 WHERE template_key = 'administrator'
   AND NOT permissions @> ARRAY['setting:read'];

UPDATE roles
   SET permissions = permissions || ARRAY['setting:read']
 WHERE template_key = 'observer'
   AND NOT permissions @> ARRAY['setting:read'];
