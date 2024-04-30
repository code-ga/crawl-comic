-- FUNCTION: public.notify_update_or_kreatesample_query()

-- DROP FUNCTION IF EXISTS public.notify_update_or_kreatesample_query();

CREATE OR REPLACE FUNCTION public.notify_update_or_kreatesample_query()
    RETURNS trigger
    LANGUAGE 'plpgsql'
    COST 100
    VOLATILE NOT LEAKPROOF
AS $BODY$
BEGIN
  PERFORM pg_notify('new_update_or_create_' || TG_TABLE_NAME, row_to_json(NEW)::text);
  RETURN NULL;
END;
$BODY$;

ALTER FUNCTION public.notify_update_or_kreatesample_query()
    OWNER TO postgres;

-- Trigger: urls_update_trigger

-- DROP TRIGGER IF EXISTS urls_update_trigger ON public."Urls";

CREATE OR REPLACE TRIGGER urls_update_trigger
    AFTER UPDATE 
    ON public."Urls"
    FOR EACH ROW
    EXECUTE FUNCTION public.notify_update_or_kreatesample_query();

-- Trigger: comic_update_or_create

-- DROP TRIGGER IF EXISTS comic_update_or_create ON public."Comic";

CREATE OR REPLACE TRIGGER comic_update_or_create
    AFTER INSERT OR UPDATE 
    ON public."Comic"
    FOR EACH ROW
    EXECUTE FUNCTION public.notify_update_or_kreatesample_query();

CREATE OR REPLACE FUNCTION public.notify_delete_query()
    RETURNS trigger
    LANGUAGE 'plpgsql'
    COST 100
    VOLATILE NOT LEAKPROOF
AS $BODY$
BEGIN
  PERFORM pg_notify('delete_' || TG_TABLE_NAME, row_to_json(OLD)::text);
  RETURN NULL;
END;
$BODY$;

ALTER FUNCTION public.notify_delete_query()
    OWNER TO postgres;

-- Trigger: comic_delete_trigger

-- DROP TRIGGER IF EXISTS comic_delete_trigger ON public."Comic";

CREATE OR REPLACE TRIGGER comic_delete_trigger
    AFTER DELETE 
    ON public."Comic"
    FOR EACH ROW
    EXECUTE FUNCTION public.notify_delete_query();