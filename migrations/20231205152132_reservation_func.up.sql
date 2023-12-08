CREATE OR REPLACE FUNCTION rsvp.query(
  uid text,
  rid text,
  during TSTZRANGE,
  status rsvp.reservation_status,
  page integer DEFAULT 1,
  is_desc bool DEFAULT FALSE,
  page_size integer DEFAULT 10
) RETURNS TABLE (LIKE rsvp.reservations) AS $$
DECLARE
  _sql text;
BEGIN
  IF page_size < 10 OR page_size > 100 THEN
    page_size := 10;
  END IF;
  IF page < 1 THEN
    page := 1;
  END IF;
  -- format the query based on parameters
  _sql := format(
      'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND status = %L AND %s ORDER BY lower(timespan) %s LIMIT
       %s OFFSET %s',
       during,
       status,
      CASE
        WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
        WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
        WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
        ELSE 'user_id = ' || quote_literal(uid) || ' AND resource_id = ' || quote_literal(rid)
      END,
      CASE
        WHEN is_desc THEN 'DESC'
        ELSE 'ASC'
      END,
      page_size,
      (page - 1) * page_size
  );
  RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;
