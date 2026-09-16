-- The "Meine Organisation" entry needs `setting:read` and `organization:read`
-- together: the page behind it loads the organization tree regardless, so
-- `setting:read` alone lands on a load error. Observer got `setting:read` in
-- 20260914074231 but never the second half.
--
-- Only rows that still match a delivered template are touched. A hand-edited
-- role (template_key IS NULL) keeps exactly the permissions someone chose.

UPDATE roles
   SET permissions = permissions || ARRAY['organization:read']
 WHERE template_key = 'observer'
   AND NOT permissions @> ARRAY['organization:read'];
