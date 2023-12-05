CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during tstzrange)
RETURNS TABLE (LIKE rsvp.reservations) AS $$
BEGIN
  IF uid is NULL AND rid IS NULL THEN
    RETURN QUERY SELECT * FROM rsvp.reservations WHERE timspan && during;
  ELSEIF uid IS NULL THEN
    RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND during @> timspan;
  ELSEIF rid IS NULL THEN
    RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND during @> timspan;
  ELSE
    RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND user_id = uid AND during @> timspan;
  END IF;
END;
$$ LANGUAGE plpgsql;
