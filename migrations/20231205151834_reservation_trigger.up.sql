-- reservation change queue
CREATE TABLE rsvp.reservation_changes (
  id SERIAL NOT NULL,
  reservation_id uuid NOT NULL,
  op rsvp.reservation_update_type NOT NULL
);

-- trigger for add/update/delete a reservation
CREATE OR REPLACE FUNCTION rsvp.reservations_trigger() RETURNS TRIGGER AS $$
BEGIN
  IF TG_OP = 'INSERT' THEN
    INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES
    (NEW.id, 'create');
  ELSEIF TG_OP = 'UPDATE' THEN
    IF OLD.status <> NEW.status THEN
      INSERT INTO rsvp.reservation_chanegs (reservation_id,op)
      VALUES (NEW.id, 'update');
    END IF;
  ELSEIF TG_OP = 'DELETE' THEN
    INSERT INTO rsvp.reservation_changes (reservation_id,op)
    VALUES (OLD.id, 'delete');
  END IF;
  -- notify a channel called reservation_update
  NOTIFY reservation_update;
  RETURN NULL;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservations_trigger
  AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations
  FOR EACH ROW EXECUTE PROCEDURE rsvp.reservations_trigger();
