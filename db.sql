--
-- PostgreSQL database dump
--

-- Dumped from database version 16.2
-- Dumped by pg_dump version 16.2

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: add_academic_plan(character varying, character varying, character varying, bigint, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_academic_plan(specialization_name character varying, subject_name character varying, certification_type_name character varying, hours bigint, semester bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO academic_plan (id_specialization, id_subject, id_certification, hours, semester)
    VALUES (
        (SELECT id FROM specialization WHERE full_name = specialization_name LIMIT 1),
        (SELECT id FROM subject WHERE name = subject_name LIMIT 1),
        (SELECT id FROM certification_type WHERE name = certification_type_name LIMIT 1),
        hours,
        semester
    )
    ON CONFLICT DO NOTHING;
END;
$$;


ALTER FUNCTION public.add_academic_plan(specialization_name character varying, subject_name character varying, certification_type_name character varying, hours bigint, semester bigint) OWNER TO postgres;

--
-- Name: add_academic_plan_trigger(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_academic_plan_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Добавить записи в student_card для всех студентов соответствующей специализации
    INSERT INTO student_card (id_student, id_plan, mark)
    SELECT s.id, NEW.id, NULL
    FROM student s
    JOIN "group" g ON s.id_group = g.id
    WHERE g.id_specialization = NEW.id_specialization;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.add_academic_plan_trigger() OWNER TO postgres;

--
-- Name: add_group(bigint, bigint, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_group(p_year bigint, p_numer bigint, p_cifr character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_id_specialization BIGINT;
BEGIN
    -- Получаем id_specialization по cifr
    SELECT "id" INTO v_id_specialization
    FROM "specialization"
    WHERE "cifr" = p_cifr;

    -- Проверяем, существует ли специализация
    IF v_id_specialization IS NULL THEN
        RAISE EXCEPTION 'Специализация с шифром % не найдена', p_cifr;
    END IF;

    -- Добавляем группу
    INSERT INTO "group" ("year", "numer", "id_specialization")
    VALUES (p_year, p_numer, v_id_specialization);
END;
$$;


ALTER FUNCTION public.add_group(p_year bigint, p_numer bigint, p_cifr character varying) OWNER TO postgres;

--
-- Name: add_plan_for_teacher(character varying, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_plan_for_teacher(p_login character varying, plan_id bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Проверка существования пользователя и его роли
    IF NOT EXISTS (SELECT 1 FROM users WHERE login = p_login AND id_role = 4) THEN
        RAISE EXCEPTION 'Teacher with login % does not exist or is not a teacher', p_login;
    END IF;

    -- Проверка существования плана
    IF NOT EXISTS (SELECT 1 FROM academic_plan WHERE id = plan_id) THEN
        RAISE EXCEPTION 'Academic plan with id % does not exist', plan_id;
    END IF;

    -- Связывание логина преподавателя с id плана
    BEGIN
        INSERT INTO login_teacher (login, id_plan_record)
        VALUES (p_login, plan_id);
    EXCEPTION WHEN unique_violation THEN
        -- Логика в случае, если запись уже существует (например, можно игнорировать)
        RAISE NOTICE 'Login % is already linked to plan %', p_login, plan_id;
    END;
END;
$$;


ALTER FUNCTION public.add_plan_for_teacher(p_login character varying, plan_id bigint) OWNER TO postgres;

--
-- Name: add_specialization(character varying, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_specialization(p_cifr character varying, p_full_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO "specialization" ("cifr", "full_name")
    VALUES (p_cifr, p_full_name);
END;
$$;


ALTER FUNCTION public.add_specialization(p_cifr character varying, p_full_name character varying) OWNER TO postgres;

--
-- Name: add_student(character varying, character varying, character varying, character varying, bigint, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_student(p_first_name character varying, p_second_name character varying, p_middle_name character varying, p_cifr character varying, p_year bigint, p_numer bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_id_group BIGINT;
BEGIN
    -- Получаем id_group по cifr, year, numer
    SELECT g.id INTO v_id_group
    FROM "group" g
    JOIN "specialization" s ON g.id_specialization = s.id
    WHERE s."cifr" = p_cifr AND g."year" = p_year AND g."numer" = p_numer;

    -- Если группа не найдена, выбрасываем исключение
    IF v_id_group IS NULL THEN
        RAISE EXCEPTION 'Группа с такими параметрами (cifr=% year=% numer=%) не найдена', p_cifr, p_year, p_numer;
    END IF;

    -- Добавляем студента
    INSERT INTO "student" ("first_name", "second_name", "middle_name", "id_group")
    VALUES (p_first_name, p_second_name, p_middle_name, v_id_group);
END;
$$;


ALTER FUNCTION public.add_student(p_first_name character varying, p_second_name character varying, p_middle_name character varying, p_cifr character varying, p_year bigint, p_numer bigint) OWNER TO postgres;

--
-- Name: add_student_card(bigint, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_student_card(student_id bigint, subject_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    plan_id BIGINT;
BEGIN
    -- Найти учебный план для студента и предмета
    SELECT ap.id INTO plan_id
    FROM academic_plan ap
    JOIN student s ON s.id = student_id
    JOIN "group" g ON s.id_group = g.id
    WHERE ap.id_specialization = g.id_specialization
      AND EXISTS (
          SELECT 1 FROM subject sub WHERE sub.id = ap.id_subject AND sub.name = subject_name
      );

    -- Проверка, найден ли учебный план
    IF plan_id IS NULL THEN
        RAISE EXCEPTION 'No academic plan found for student % and subject %', student_id, subject_name;
    END IF;

    -- Добавление записи в зачетную книжку
    INSERT INTO student_card (id_student, id_plan, mark)
    VALUES (student_id, plan_id, NULL);
END;
$$;


ALTER FUNCTION public.add_student_card(student_id bigint, subject_name character varying) OWNER TO postgres;

--
-- Name: add_student_card(bigint, character varying, bigint, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_student_card(student_id bigint, subject_name character varying, semester bigint, certification_type_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    plan_id BIGINT;
BEGIN
    -- Найти учебный план для студента, предмета, семестра и типа аттестации
    SELECT ap.id INTO plan_id
    FROM academic_plan ap
    JOIN student s ON s.id = student_id
    JOIN "group" g ON s.id_group = g.id
    JOIN certification_type ct ON ap.id_certification = ct.id
    WHERE ap.id_specialization = g.id_specialization
      AND EXISTS (
          SELECT 1 FROM subject sub WHERE sub.id = ap.id_subject AND sub.name = subject_name
      )
      AND ap.semester = semester
      AND ct.name = certification_type_name;

    -- Проверка, найден ли учебный план
    IF plan_id IS NULL THEN
        RAISE EXCEPTION 'No academic plan found for student %, subject %, semester %, and certification type %', student_id, subject_name, semester, certification_type_name;
    END IF;

    -- Добавление записи в зачетную книжку
    INSERT INTO student_card (id_student, id_plan, mark)
    VALUES (student_id, plan_id, NULL);
END;
$$;


ALTER FUNCTION public.add_student_card(student_id bigint, subject_name character varying, semester bigint, certification_type_name character varying) OWNER TO postgres;

--
-- Name: add_student_trigger(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_student_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Добавить записи в student_card для всех предметов учебного плана группы студента
    INSERT INTO student_card (id_student, id_plan, mark)
    SELECT NEW.id, ap.id, NULL
    FROM academic_plan ap
    JOIN "group" g ON NEW.id_group = g.id
    WHERE ap.id_specialization = g.id_specialization;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.add_student_trigger() OWNER TO postgres;

--
-- Name: add_subject(character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.add_subject(p_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO "subject" ("name")
    VALUES (p_name);
END;
$$;


ALTER FUNCTION public.add_subject(p_name character varying) OWNER TO postgres;

--
-- Name: check_student_card_mark(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.check_student_card_mark() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    cert_type INTEGER;
BEGIN
    -- Получаем id_certification для соответствующего academic_plan
    SELECT ap.id_certification
    INTO cert_type
    FROM academic_plan ap
    JOIN student s ON s.id = NEW.id_student
    JOIN "group" g ON g.id = s.id_group
    WHERE ap.id = NEW.id_plan
      AND ap.id_specialization = g.id_specialization;

    -- Проверяем значение mark в зависимости от id_certification
    IF cert_type = 1 THEN
        -- Если id_certification равно 1
        IF NEW.mark NOT IN ('Зачет', 'Не зачет', 'Не явка') THEN
            RAISE EXCEPTION 'Недопустимое значение mark для id_certification = 1. Допустимые значения: "Зачет", "Не зачет", "Не явка"';
        END IF;
    ELSE
        -- Для всех остальных id_certification
        IF NEW.mark NOT IN ('Не явка', 'Не удовлетворительно', 'Удовлетворительно', 'Хорошо', 'Отлично') THEN
            RAISE EXCEPTION 'Недопустимое значение mark для id_certification != 1. Допустимые значения: "Не явка", "Не удовлетворительно", "Удовлетворительно", "Хорошо", "Отлично"';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;


ALTER FUNCTION public.check_student_card_mark() OWNER TO postgres;

--
-- Name: create_student_login(character varying, character varying, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.create_student_login(p_login character varying, password character varying, id_student bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Проверка существования студента
    IF NOT EXISTS (SELECT 1 FROM student WHERE id = id_student) THEN
        RAISE EXCEPTION 'Student with id % does not exist', id_student;
    END IF;

    -- Добавление пользователя в таблицу users
    INSERT INTO users (login, password, id_role)
    VALUES (p_login, password, 3)
    ON CONFLICT (login) DO NOTHING;

    -- Связывание логина с id студента
    INSERT INTO login_student (login, id_student)
    VALUES (p_login, id_student)
    ON CONFLICT (login) DO NOTHING;
END;
$$;


ALTER FUNCTION public.create_student_login(p_login character varying, password character varying, id_student bigint) OWNER TO postgres;

--
-- Name: create_teacher_login(character varying, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.create_teacher_login(p_login character varying, password character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Добавление пользователя в таблицу users
    INSERT INTO users (login, password, id_role)
    VALUES (p_login, password, 4)
    ON CONFLICT (login) DO NOTHING;
END;
$$;


ALTER FUNCTION public.create_teacher_login(p_login character varying, password character varying) OWNER TO postgres;

--
-- Name: create_user(character varying, character varying, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.create_user(p_login character varying, p_password character varying, p_role_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_role_id bigint;
BEGIN
    -- Получаем id роли по названию
    SELECT id INTO v_role_id
    FROM role
    WHERE name = p_role_name
    LIMIT 1;

    -- Проверка, существует ли такая роль
    IF v_role_id IS NULL THEN
        RAISE EXCEPTION 'Role "%" not found', p_role_name;
    END IF;

    -- Создание нового пользователя
    INSERT INTO users (login, password, id_role)
    VALUES (p_login, p_password, v_role_id);
END;
$$;


ALTER FUNCTION public.create_user(p_login character varying, p_password character varying, p_role_name character varying) OWNER TO postgres;

--
-- Name: delete_academic_plan(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_academic_plan(plan_id bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM academic_plan
    WHERE id = plan_id;
END;
$$;


ALTER FUNCTION public.delete_academic_plan(plan_id bigint) OWNER TO postgres;

--
-- Name: delete_academic_plan_trigger(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_academic_plan_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Удалить все записи из student_card для удаляемого учебного плана
    DELETE FROM student_card WHERE id_plan = OLD.id;
    RETURN OLD;
END;
$$;


ALTER FUNCTION public.delete_academic_plan_trigger() OWNER TO postgres;

--
-- Name: delete_group(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_group(p_id bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM "group" WHERE "id" = p_id;
END;
$$;


ALTER FUNCTION public.delete_group(p_id bigint) OWNER TO postgres;

--
-- Name: delete_specialization(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_specialization(p_id bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM "specialization" WHERE "id" = p_id;
END;
$$;


ALTER FUNCTION public.delete_specialization(p_id bigint) OWNER TO postgres;

--
-- Name: delete_student(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_student(p_student_id bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Удаляем студента по id
    DELETE FROM "student" WHERE "id" = p_student_id;
END;
$$;


ALTER FUNCTION public.delete_student(p_student_id bigint) OWNER TO postgres;

--
-- Name: delete_student_card(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_student_card(id bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM student_card WHERE id_student = id;
END;
$$;


ALTER FUNCTION public.delete_student_card(id bigint) OWNER TO postgres;

--
-- Name: delete_student_login(character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_student_login(p_login character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Удаление записи из login_student
    DELETE FROM login_student WHERE login = p_login;

    -- Удаление записи из users
    DELETE FROM users WHERE login = p_login AND id_role = 3;
END;
$$;


ALTER FUNCTION public.delete_student_login(p_login character varying) OWNER TO postgres;

--
-- Name: delete_student_trigger(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_student_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Удалить все записи из student_card для удаляемого студента
    DELETE FROM student_card WHERE id_student = OLD.id;
    RETURN OLD;
END;
$$;


ALTER FUNCTION public.delete_student_trigger() OWNER TO postgres;

--
-- Name: delete_subject(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_subject(p_id bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM "subject" WHERE "id" = p_id;
END;
$$;


ALTER FUNCTION public.delete_subject(p_id bigint) OWNER TO postgres;

--
-- Name: delete_teacher_login(character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_teacher_login(p_login character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Удаление записи из login_teacher
    DELETE FROM login_teacher WHERE login = p_login;

    -- Удаление записи из users
    DELETE FROM users WHERE login = p_login AND id_role = 4;
END;
$$;


ALTER FUNCTION public.delete_teacher_login(p_login character varying) OWNER TO postgres;

--
-- Name: delete_user(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.delete_user(p_user_id bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Удаляем пользователя по id
    DELETE FROM users
    WHERE id = p_user_id;
    
    -- Проверка, был ли найден пользователь для удаления
    IF NOT FOUND THEN
        RAISE EXCEPTION 'User with id "%" not found', p_user_id;
    END IF;
END;
$$;


ALTER FUNCTION public.delete_user(p_user_id bigint) OWNER TO postgres;

--
-- Name: get_academic_plan_for_teacher(character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.get_academic_plan_for_teacher(p_login character varying) RETURNS TABLE(plan_id bigint, specialization_name character varying, subject_name character varying, certification_type character varying, hours bigint, semester bigint)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        ap.id::BIGINT AS plan_id,
        s.full_name AS specialization_name,
        sub.name AS subject_name,
        ct.name AS certification_type,
        ap.hours::BIGINT AS hours,
        ap.semester::BIGINT AS semester
    FROM
        login_teacher lt
    JOIN academic_plan ap ON lt.id_plan_record = ap.id
    JOIN specialization s ON ap.id_specialization = s.id
    JOIN subject sub ON ap.id_subject = sub.id
    JOIN certification_type ct ON ap.id_certification = ct.id
    WHERE
        lt.login = p_login;
END;
$$;


ALTER FUNCTION public.get_academic_plan_for_teacher(p_login character varying) OWNER TO postgres;

--
-- Name: get_groups_by_name(character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.get_groups_by_name(specialization_name character varying) RETURNS TABLE(cifr character varying, year bigint, number bigint)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.cifr,
        g.year::BIGINT AS year,
        g.numer::BIGINT AS number
    FROM
        "group" g
    JOIN specialization s ON g.id_specialization = s.id
    WHERE
        s.full_name = specialization_name;
END;
$$;


ALTER FUNCTION public.get_groups_by_name(specialization_name character varying) OWNER TO postgres;

--
-- Name: get_marks_student(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.get_marks_student(student_id bigint) RETURNS TABLE(subject_name character varying, semester bigint, hours bigint, certification_type character varying, mark character varying)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        sub.name AS subject_name,
        ap.semester::BIGINT AS semester,
        ap.hours::BIGINT AS hours,
        ct.name AS certification_type,
        sc.mark
    FROM
        student_card sc
    JOIN academic_plan ap ON sc.id_plan = ap.id
    JOIN subject sub ON ap.id_subject = sub.id
    JOIN certification_type ct ON ap.id_certification = ct.id
    WHERE
        sc.id_student = student_id;
END;
$$;


ALTER FUNCTION public.get_marks_student(student_id bigint) OWNER TO postgres;

--
-- Name: get_student_card_for(character varying, bigint, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.get_student_card_for(group_cifr character varying, group_year bigint, group_number bigint) RETURNS TABLE(student_id integer, first_name character varying, second_name character varying, subject_name character varying, semester bigint, certification_type character varying, mark character varying)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT 
        s.id AS student_id,
        s.first_name,
        s.second_name,
        sub.name AS subject_name,
        ap.semester,
        ct.name AS certification_type,
        sc.mark
    FROM student s
    JOIN "group" g ON s.id_group = g.id
    JOIN academic_plan ap ON g.id_specialization = ap.id_specialization
    JOIN subject sub ON ap.id_subject = sub.id
    JOIN certification_type ct ON ap.id_certification = ct.id
    LEFT JOIN student_card sc ON s.id = sc.id_student AND ap.id = sc.id_plan
    WHERE g.year = group_year AND g.numer = group_number AND EXISTS (
        SELECT 1 FROM specialization sp WHERE sp.cifr = group_cifr AND sp.id = g.id_specialization
    );
END;
$$;


ALTER FUNCTION public.get_student_card_for(group_cifr character varying, group_year bigint, group_number bigint) OWNER TO postgres;

--
-- Name: get_student_info(character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.get_student_info(p_login character varying) RETURNS TABLE(id_student integer, first_name character varying, second_name character varying, middle_name character varying, id_group bigint, cifr character varying, year bigint, group_number bigint, id_specialization bigint, specialization_name character varying)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.id AS id_student,                    -- id студента
        s.first_name,                          -- имя студента
        s.second_name,                         -- фамилия студента
        s.middle_name,                         -- отчество студента
        s.id_group,                            -- id группы
        sp.cifr,                               -- цифр специальности
        g.year,                                -- год поступления в группу
        g.numer AS group_number,               -- номер группы
        g.id_specialization,                   -- id специальности
        sp.full_name AS specialization_name    -- название специальности
    FROM
        users u
    JOIN
        login_student ls ON u.login = ls.login
    JOIN
        student s ON ls.id_student = s.id
    JOIN
        "group" g ON s.id_group = g.id
    JOIN
        specialization sp ON g.id_specialization = sp.id
    WHERE
        u.login = p_login
        AND u.id_role = (SELECT id FROM role WHERE name = 'студент');  -- Роль студента
END;
$$;


ALTER FUNCTION public.get_student_info(p_login character varying) OWNER TO postgres;

--
-- Name: get_student_plan(bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.get_student_plan(student_id bigint) RETURNS TABLE(subject_name character varying, semester bigint, hours bigint, certification_type character varying)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT 
        sub.name AS subject_name,
        ap.semester,
        ap.hours,
        ct.name AS certification_type
    FROM academic_plan ap
    JOIN subject sub ON ap.id_subject = sub.id
    JOIN certification_type ct ON ap.id_certification = ct.id
    JOIN student s ON s.id = student_id
    JOIN "group" g ON s.id_group = g.id
    WHERE ap.id_specialization = g.id_specialization;
END;
$$;


ALTER FUNCTION public.get_student_plan(student_id bigint) OWNER TO postgres;

--
-- Name: get_students_by_plan_in_group(bigint, character varying, bigint, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.get_students_by_plan_in_group(plan_id bigint, group_cifr character varying, group_year bigint, group_number bigint) RETURNS TABLE(student_id bigint, first_name character varying, second_name character varying, middle_name character varying, certification_name character varying, mark character varying)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        st.id::BIGINT AS student_id,
        st.first_name,
        st.second_name,
        st.middle_name,
        ct.name AS certification_name,
        sc.mark
    FROM
        student st
    JOIN "group" g ON st.id_group = g.id
    JOIN academic_plan ap ON ap.id_specialization = g.id_specialization
    JOIN certification_type ct ON ap.id_certification = ct.id
    LEFT JOIN student_card sc ON sc.id_student = st.id AND sc.id_plan = plan_id
    WHERE
        ap.id = plan_id
        AND g.id_specialization = (SELECT id FROM specialization WHERE cifr = group_cifr)
        AND g.year = group_year
        AND g.numer = group_number;
END;
$$;


ALTER FUNCTION public.get_students_by_plan_in_group(plan_id bigint, group_cifr character varying, group_year bigint, group_number bigint) OWNER TO postgres;

--
-- Name: get_students_in_group(character varying, bigint, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.get_students_in_group(group_cifr character varying, group_year bigint, group_number bigint) RETURNS TABLE(student_id bigint, first_name character varying, second_name character varying, middle_name character varying)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT 
        s.id::BIGINT AS student_id,
        s.first_name,
        s.second_name,
        s.middle_name
    FROM student s
    JOIN "group" g ON s.id_group = g.id
    WHERE g.year = group_year AND g.numer = group_number AND EXISTS (
        SELECT 1 FROM specialization sp WHERE sp.cifr = group_cifr AND sp.id = g.id_specialization
    );
END;
$$;


ALTER FUNCTION public.get_students_in_group(group_cifr character varying, group_year bigint, group_number bigint) OWNER TO postgres;

--
-- Name: mark_student(bigint, bigint, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.mark_student(plan_id bigint, student_id bigint, mark character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Проверка существования студента и плана
    IF NOT EXISTS (SELECT 1 FROM student WHERE id = student_id) THEN
        RAISE EXCEPTION 'Student with id % does not exist', student_id;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM academic_plan WHERE id = plan_id) THEN
        RAISE EXCEPTION 'Academic plan with id % does not exist', plan_id;
    END IF;

    -- Вставка или обновление оценки в student_card
    INSERT INTO student_card (id_student, id_plan, mark)
    VALUES (student_id, plan_id, mark)
    ON CONFLICT (id_student, id_plan)
    DO UPDATE SET mark = EXCLUDED.mark;
END;
$$;


ALTER FUNCTION public.mark_student(plan_id bigint, student_id bigint, mark character varying) OWNER TO postgres;

--
-- Name: update_academic_plan(bigint, character varying, character varying, character varying, bigint, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_academic_plan(plan_id bigint, specialization_name character varying, subject_name character varying, certification_type_name character varying, h bigint, s bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE academic_plan
    SET 
        id_specialization = (SELECT id FROM specialization WHERE full_name = specialization_name),
        id_subject = (SELECT id FROM subject WHERE name = subject_name),
        id_certification = (SELECT id FROM certification_type WHERE name = certification_type_name),
        hours = h,
        semester = s
    WHERE id = plan_id;
END;
$$;


ALTER FUNCTION public.update_academic_plan(plan_id bigint, specialization_name character varying, subject_name character varying, certification_type_name character varying, h bigint, s bigint) OWNER TO postgres;

--
-- Name: update_academic_plan_trigger(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_academic_plan_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Удалить старые записи из student_card, которые не соответствуют обновленной записи
    DELETE FROM student_card
    WHERE id_plan = OLD.id;

    -- Добавить новые записи в student_card для всех студентов соответствующей специализации
    INSERT INTO student_card (id_student, id_plan, mark)
    SELECT s.id, NEW.id, NULL
    FROM student s
    JOIN "group" g ON s.id_group = g.id
    WHERE g.id_specialization = NEW.id_specialization;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_academic_plan_trigger() OWNER TO postgres;

--
-- Name: update_group(bigint, bigint, bigint, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_group(p_id bigint, p_year bigint, p_numer bigint, p_cifr character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_id_specialization BIGINT;
BEGIN
    -- Получаем id_specialization по cifr
    SELECT "id" INTO v_id_specialization
    FROM "specialization"
    WHERE "cifr" = p_cifr;

    -- Проверяем, существует ли специализация
    IF v_id_specialization IS NULL THEN
        RAISE EXCEPTION 'Специализация с шифром % не найдена', p_cifr;
    END IF;

    -- Обновляем группу
    UPDATE "group"
    SET "year" = p_year, "numer" = p_numer, "id_specialization" = v_id_specialization
    WHERE "id" = p_id;
END;
$$;


ALTER FUNCTION public.update_group(p_id bigint, p_year bigint, p_numer bigint, p_cifr character varying) OWNER TO postgres;

--
-- Name: update_specialization(bigint, character varying, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_specialization(p_id bigint, p_cifr character varying, p_full_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE "specialization"
    SET "cifr" = p_cifr, "full_name" = p_full_name
    WHERE "id" = p_id;
END;
$$;


ALTER FUNCTION public.update_specialization(p_id bigint, p_cifr character varying, p_full_name character varying) OWNER TO postgres;

--
-- Name: update_student(bigint, character varying, character varying, character varying, character varying, bigint, bigint); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_student(p_student_id bigint, p_first_name character varying, p_second_name character varying, p_middle_name character varying, p_cifr character varying, p_year bigint, p_numer bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_id_group BIGINT;
BEGIN
    -- Получаем id_group по cifr, year, numer
    SELECT g.id INTO v_id_group
    FROM "group" g
    JOIN "specialization" s ON g.id_specialization = s.id
    WHERE s."cifr" = p_cifr AND g."year" = p_year AND g."numer" = p_numer;

    -- Если группа не найдена, выбрасываем исключение
    IF v_id_group IS NULL THEN
        RAISE EXCEPTION 'Группа с такими параметрами (cifr=% year=% numer=%) не найдена', p_cifr, p_year, p_numer;
    END IF;

    -- Обновляем данные студента
    UPDATE "student"
    SET "first_name" = p_first_name, 
        "second_name" = p_second_name, 
        "middle_name" = p_middle_name,
        "id_group" = v_id_group
    WHERE "id" = p_student_id;
END;
$$;


ALTER FUNCTION public.update_student(p_student_id bigint, p_first_name character varying, p_second_name character varying, p_middle_name character varying, p_cifr character varying, p_year bigint, p_numer bigint) OWNER TO postgres;

--
-- Name: update_student_card(bigint, bigint, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_student_card(id bigint, student_id bigint, subject_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    plan_id BIGINT;
BEGIN
    -- Найти учебный план для студента и предмета
    SELECT ap.id INTO plan_id
    FROM academic_plan ap
    JOIN student s ON s.id = student_id
    JOIN "group" g ON s.id_group = g.id
    WHERE ap.id_specialization = g.id_specialization
      AND EXISTS (
          SELECT 1 FROM subject sub WHERE sub.id = ap.id_subject AND sub.name = subject_name
      );

    -- Проверка, найден ли учебный план
    IF plan_id IS NULL THEN
        RAISE EXCEPTION 'No academic plan found for student % and subject %', student_id, subject_name;
    END IF;

    -- Обновление записи в зачетной книжке
    UPDATE student_card
    SET id_student = student_id, id_plan = plan_id
    WHERE id_student = id AND id_plan = plan_id;
END;
$$;


ALTER FUNCTION public.update_student_card(id bigint, student_id bigint, subject_name character varying) OWNER TO postgres;

--
-- Name: update_student_card(bigint, bigint, character varying, bigint, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_student_card(id bigint, student_id bigint, subject_name character varying, semester bigint, certification_type_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    plan_id BIGINT;
BEGIN
    -- Найти учебный план для студента, предмета, семестра и типа аттестации
    SELECT ap.id INTO plan_id
    FROM academic_plan ap
    JOIN student s ON s.id = student_id
    JOIN "group" g ON s.id_group = g.id
    JOIN certification_type ct ON ap.id_certification = ct.id
    WHERE ap.id_specialization = g.id_specialization
      AND EXISTS (
          SELECT 1 FROM subject sub WHERE sub.id = ap.id_subject AND sub.name = subject_name
      )
      AND ap.semester = semester
      AND ct.name = certification_type_name;

    -- Проверка, найден ли учебный план
    IF plan_id IS NULL THEN
        RAISE EXCEPTION 'No academic plan found for student %, subject %, semester %, and certification type %', student_id, subject_name, semester, certification_type_name;
    END IF;

    -- Обновление записи в зачетной книжке
    UPDATE student_card
    SET id_student = student_id, id_plan = plan_id
    WHERE id_student = id AND id_plan = plan_id;
END;
$$;


ALTER FUNCTION public.update_student_card(id bigint, student_id bigint, subject_name character varying, semester bigint, certification_type_name character varying) OWNER TO postgres;

--
-- Name: update_student_group_trigger(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_student_group_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Удалить записи из student_card, которые не соответствуют новому учебному плану группы
    DELETE FROM student_card
    WHERE id_student = NEW.id
      AND id_plan NOT IN (
          SELECT ap.id
          FROM academic_plan ap
          JOIN "group" g ON NEW.id_group = g.id
          WHERE ap.id_specialization = g.id_specialization
      );

    -- Добавить записи в student_card для новых предметов учебного плана группы студента
    INSERT INTO student_card (id_student, id_plan, mark)
    SELECT NEW.id, ap.id, NULL
    FROM academic_plan ap
    JOIN "group" g ON NEW.id_group = g.id
    WHERE ap.id_specialization = g.id_specialization
      AND ap.id NOT IN (
          SELECT id_plan FROM student_card WHERE id_student = NEW.id
      );
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_student_group_trigger() OWNER TO postgres;

--
-- Name: update_subject(bigint, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_subject(p_id bigint, p_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE "subject"
    SET "name" = p_name
    WHERE "id" = p_id;
END;
$$;


ALTER FUNCTION public.update_subject(p_id bigint, p_name character varying) OWNER TO postgres;

--
-- Name: update_user(character varying, character varying, character varying, character varying); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_user(p_login character varying, p_new_login character varying, p_new_password character varying, p_role_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_role_id bigint;
BEGIN
    -- Получаем id роли по названию
    SELECT id INTO v_role_id
    FROM role
    WHERE name = p_role_name
    LIMIT 1;

    -- Проверка, существует ли такая роль
    IF v_role_id IS NULL THEN
        RAISE EXCEPTION 'Role "%" not found', p_role_name;
    END IF;

    -- Обновляем данные пользователя
    UPDATE users
    SET login = p_new_login, password = p_new_password
    WHERE login = p_login AND id_role = v_role_id;
    
    -- Проверка, был ли найден пользователь для изменения
    IF NOT FOUND THEN
        RAISE EXCEPTION 'User "%" with role "%" not found', p_login, p_role_name;
    END IF;
END;
$$;


ALTER FUNCTION public.update_user(p_login character varying, p_new_login character varying, p_new_password character varying, p_role_name character varying) OWNER TO postgres;

--
-- Name: validate_student_card(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.validate_student_card() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    plan_id BIGINT;
BEGIN
    -- Найти учебный план для студента и предмета
    SELECT ap.id INTO plan_id
    FROM academic_plan ap
    JOIN student s ON s.id = NEW.id_student
    JOIN "group" g ON s.id_group = g.id
    WHERE ap.id_specialization = g.id_specialization
      AND ap.id = NEW.id_plan;

    -- Проверка, найден ли учебный план
    IF plan_id IS NULL THEN
        RAISE EXCEPTION 'No academic plan found for student % and plan %', NEW.id_student, NEW.id_plan;
    END IF;

    RETURN NEW;
END;
$$;


ALTER FUNCTION public.validate_student_card() OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: academic_plan; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.academic_plan (
    id integer NOT NULL,
    id_specialization bigint NOT NULL,
    id_subject bigint NOT NULL,
    id_certification bigint NOT NULL,
    semester bigint NOT NULL,
    hours bigint NOT NULL
);


ALTER TABLE public.academic_plan OWNER TO postgres;

--
-- Name: academic_plan_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.academic_plan_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.academic_plan_id_seq OWNER TO postgres;

--
-- Name: academic_plan_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.academic_plan_id_seq OWNED BY public.academic_plan.id;


--
-- Name: certification_type; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.certification_type (
    id integer NOT NULL,
    name character varying(255) NOT NULL
);


ALTER TABLE public.certification_type OWNER TO postgres;

--
-- Name: certification_type_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.certification_type_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.certification_type_id_seq OWNER TO postgres;

--
-- Name: certification_type_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.certification_type_id_seq OWNED BY public.certification_type.id;


--
-- Name: specialization; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.specialization (
    id integer NOT NULL,
    cifr character varying(10) NOT NULL,
    full_name character varying(255) NOT NULL
);


ALTER TABLE public.specialization OWNER TO postgres;

--
-- Name: subject; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.subject (
    id integer NOT NULL,
    name character varying(255) NOT NULL
);


ALTER TABLE public.subject OWNER TO postgres;

--
-- Name: get_academic_plan; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.get_academic_plan AS
 SELECT ap.id AS plan_id,
    sp.full_name AS specialization_name,
    subj.name AS subject_name,
    ct.name AS certification_type,
    ap.hours,
    ap.semester
   FROM (((public.academic_plan ap
     JOIN public.specialization sp ON ((ap.id_specialization = sp.id)))
     JOIN public.subject subj ON ((ap.id_subject = subj.id)))
     JOIN public.certification_type ct ON ((ap.id_certification = ct.id)));


ALTER VIEW public.get_academic_plan OWNER TO postgres;

--
-- Name: group; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public."group" (
    id integer NOT NULL,
    year bigint NOT NULL,
    numer bigint NOT NULL,
    id_specialization bigint NOT NULL
);


ALTER TABLE public."group" OWNER TO postgres;

--
-- Name: get_group; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.get_group AS
 SELECT g.id,
    concat(s.cifr, '-', g.year, '-', g.numer) AS name,
    s.full_name AS specialization
   FROM (public."group" g
     JOIN public.specialization s ON ((g.id_specialization = s.id)));


ALTER VIEW public.get_group OWNER TO postgres;

--
-- Name: student; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.student (
    id integer NOT NULL,
    id_group bigint NOT NULL,
    first_name character varying(255) NOT NULL,
    second_name character varying(255) NOT NULL,
    middle_name character varying(255) NOT NULL
);


ALTER TABLE public.student OWNER TO postgres;

--
-- Name: get_students; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.get_students AS
 SELECT s.id AS student_id,
    s.first_name,
    s.second_name,
    s.middle_name,
    g.year AS group_year,
    g.numer AS group_number,
    sp.cifr AS group_cifr
   FROM ((public.student s
     JOIN public."group" g ON ((s.id_group = g.id)))
     JOIN public.specialization sp ON ((g.id_specialization = sp.id)));


ALTER VIEW public.get_students OWNER TO postgres;

--
-- Name: group_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.group_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.group_id_seq OWNER TO postgres;

--
-- Name: group_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.group_id_seq OWNED BY public."group".id;


--
-- Name: login_student; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.login_student (
    login character varying(255) NOT NULL,
    id_student bigint NOT NULL
);


ALTER TABLE public.login_student OWNER TO postgres;

--
-- Name: login_teacher; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.login_teacher (
    login character varying(255) NOT NULL,
    id_plan_record bigint NOT NULL
);


ALTER TABLE public.login_teacher OWNER TO postgres;

--
-- Name: role; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.role (
    id integer NOT NULL,
    name character varying(255) NOT NULL
);


ALTER TABLE public.role OWNER TO postgres;

--
-- Name: role_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.role_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.role_id_seq OWNER TO postgres;

--
-- Name: role_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.role_id_seq OWNED BY public.role.id;


--
-- Name: specialization_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.specialization_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.specialization_id_seq OWNER TO postgres;

--
-- Name: specialization_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.specialization_id_seq OWNED BY public.specialization.id;


--
-- Name: student_card; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.student_card (
    id_student integer NOT NULL,
    id_plan bigint NOT NULL,
    mark character varying(50)
);


ALTER TABLE public.student_card OWNER TO postgres;

--
-- Name: student_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.student_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.student_id_seq OWNER TO postgres;

--
-- Name: student_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.student_id_seq OWNED BY public.student.id;


--
-- Name: subject_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.subject_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.subject_id_seq OWNER TO postgres;

--
-- Name: subject_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.subject_id_seq OWNED BY public.subject.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.users (
    login character varying(255) NOT NULL,
    password character varying(255) NOT NULL,
    id_role integer NOT NULL
);


ALTER TABLE public.users OWNER TO postgres;

--
-- Name: academic_plan id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.academic_plan ALTER COLUMN id SET DEFAULT nextval('public.academic_plan_id_seq'::regclass);


--
-- Name: certification_type id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.certification_type ALTER COLUMN id SET DEFAULT nextval('public.certification_type_id_seq'::regclass);


--
-- Name: group id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public."group" ALTER COLUMN id SET DEFAULT nextval('public.group_id_seq'::regclass);


--
-- Name: role id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.role ALTER COLUMN id SET DEFAULT nextval('public.role_id_seq'::regclass);


--
-- Name: specialization id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.specialization ALTER COLUMN id SET DEFAULT nextval('public.specialization_id_seq'::regclass);


--
-- Name: student id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.student ALTER COLUMN id SET DEFAULT nextval('public.student_id_seq'::regclass);


--
-- Name: subject id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.subject ALTER COLUMN id SET DEFAULT nextval('public.subject_id_seq'::regclass);


--
-- Data for Name: academic_plan; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.academic_plan (id, id_specialization, id_subject, id_certification, semester, hours) FROM stdin;
5	1	1	1	1	144
2	1	1	2	2	144
3	2	1	1	1	144
4	2	1	2	2	144
6	1	3	1	1	72
7	2	3	1	1	72
11	3	2	1	3	72
13	1	2	1	3	72
\.


--
-- Data for Name: certification_type; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.certification_type (id, name) FROM stdin;
1	Зачет
2	Экзамен
3	Диф. Зачет
4	Курсовая работа
\.


--
-- Data for Name: group; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public."group" (id, year, numer, id_specialization) FROM stdin;
1	22	1	1
2	22	2	1
3	22	1	2
4	22	1	3
\.


--
-- Data for Name: login_student; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.login_student (login, id_student) FROM stdin;
ivanovii	5
\.


--
-- Data for Name: login_teacher; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.login_teacher (login, id_plan_record) FROM stdin;
matematika	6
matematika	7
prog	5
prog	2
prog	3
prog	4
\.


--
-- Data for Name: role; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.role (id, name) FROM stdin;
1	администратор
2	сотрудник деканата
3	студент
4	преподаватель
\.


--
-- Data for Name: specialization; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.specialization (id, cifr, full_name) FROM stdin;
1	АСУб	Автоматизация систем обработки информации и управления
2	ЭВМб	Вычислительные машины, комплексы, системы и сети
3	ИСИб	Интеллектуальные системы обработки информации и управления
\.


--
-- Data for Name: student; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.student (id, id_group, first_name, second_name, middle_name) FROM stdin;
5	1	Иван	Иванов	Иванович
6	2	Петр	Петров	Петрович
7	1	Владимир	Ульянов	Ильич
\.


--
-- Data for Name: student_card; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.student_card (id_student, id_plan, mark) FROM stdin;
5	5	\N
6	5	\N
6	2	\N
6	6	\N
5	6	Зачет
5	2	Хорошо
7	6	Зачет
7	2	Отлично
7	5	Зачет
5	13	\N
6	13	\N
7	13	\N
\.


--
-- Data for Name: subject; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.subject (id, name) FROM stdin;
1	Программирование
2	Базы данных
3	Математика
4	Информатика
6	Базы данных
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.users (login, password, id_role) FROM stdin;
admin	5755620910692865178	1
decan	1651628317410464532	2
ivanovii	15384363274080894204	3
matematika	14093112783739491881	4
prog	8912923737864084367	4
\.


--
-- Name: academic_plan_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.academic_plan_id_seq', 13, true);


--
-- Name: certification_type_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.certification_type_id_seq', 4, true);


--
-- Name: group_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.group_id_seq', 5, true);


--
-- Name: role_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.role_id_seq', 4, true);


--
-- Name: specialization_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.specialization_id_seq', 6, true);


--
-- Name: student_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.student_id_seq', 7, true);


--
-- Name: subject_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.subject_id_seq', 6, true);


--
-- Name: academic_plan academic_plan_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.academic_plan
    ADD CONSTRAINT academic_plan_pkey PRIMARY KEY (id);


--
-- Name: certification_type certification_type_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.certification_type
    ADD CONSTRAINT certification_type_pkey PRIMARY KEY (id);


--
-- Name: group group_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public."group"
    ADD CONSTRAINT group_pkey PRIMARY KEY (id);


--
-- Name: login_student login_student_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.login_student
    ADD CONSTRAINT login_student_pkey PRIMARY KEY (login);


--
-- Name: login_teacher login_teacher_unique; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.login_teacher
    ADD CONSTRAINT login_teacher_unique UNIQUE (login, id_plan_record);


--
-- Name: role role_name_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.role
    ADD CONSTRAINT role_name_key UNIQUE (name);


--
-- Name: role role_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.role
    ADD CONSTRAINT role_pkey PRIMARY KEY (id);


--
-- Name: specialization specialization_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.specialization
    ADD CONSTRAINT specialization_pkey PRIMARY KEY (id);


--
-- Name: student_card student_card_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.student_card
    ADD CONSTRAINT student_card_pkey PRIMARY KEY (id_student, id_plan);


--
-- Name: student student_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.student
    ADD CONSTRAINT student_pkey PRIMARY KEY (id);


--
-- Name: subject subject_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.subject
    ADD CONSTRAINT subject_pkey PRIMARY KEY (id);


--
-- Name: users unique_login; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT unique_login UNIQUE (login) INCLUDE (login);


--
-- Name: student_card unique_student_card; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.student_card
    ADD CONSTRAINT unique_student_card UNIQUE (id_student, id_plan);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (login);


--
-- Name: academic_plan academic_plan_add_trigger; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER academic_plan_add_trigger AFTER INSERT ON public.academic_plan FOR EACH ROW EXECUTE FUNCTION public.add_academic_plan_trigger();


--
-- Name: academic_plan academic_plan_delete_trigger; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER academic_plan_delete_trigger BEFORE DELETE ON public.academic_plan FOR EACH ROW EXECUTE FUNCTION public.delete_academic_plan_trigger();


--
-- Name: academic_plan academic_plan_update_trigger; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER academic_plan_update_trigger AFTER UPDATE ON public.academic_plan FOR EACH ROW EXECUTE FUNCTION public.update_academic_plan_trigger();


--
-- Name: student student_add_trigger; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER student_add_trigger AFTER INSERT ON public.student FOR EACH ROW EXECUTE FUNCTION public.add_student_trigger();


--
-- Name: student_card student_card_validation; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER student_card_validation BEFORE INSERT OR UPDATE ON public.student_card FOR EACH ROW EXECUTE FUNCTION public.validate_student_card();


--
-- Name: student student_delete_trigger; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER student_delete_trigger BEFORE DELETE ON public.student FOR EACH ROW EXECUTE FUNCTION public.delete_student_trigger();


--
-- Name: student student_update_trigger; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER student_update_trigger AFTER UPDATE OF id_group ON public.student FOR EACH ROW EXECUTE FUNCTION public.update_student_group_trigger();


--
-- Name: student_card trigger_check_mark; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER trigger_check_mark BEFORE INSERT OR UPDATE ON public.student_card FOR EACH ROW EXECUTE FUNCTION public.check_student_card_mark();


--
-- Name: academic_plan academic_plan_fk1; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.academic_plan
    ADD CONSTRAINT academic_plan_fk1 FOREIGN KEY (id_specialization) REFERENCES public.specialization(id);


--
-- Name: academic_plan academic_plan_fk2; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.academic_plan
    ADD CONSTRAINT academic_plan_fk2 FOREIGN KEY (id_subject) REFERENCES public.subject(id);


--
-- Name: academic_plan academic_plan_fk3; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.academic_plan
    ADD CONSTRAINT academic_plan_fk3 FOREIGN KEY (id_certification) REFERENCES public.certification_type(id);


--
-- Name: users fk_role; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT fk_role FOREIGN KEY (id_role) REFERENCES public.role(id);


--
-- Name: group group_fk3; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public."group"
    ADD CONSTRAINT group_fk3 FOREIGN KEY (id_specialization) REFERENCES public.specialization(id);


--
-- Name: login_student login_student_fk1; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.login_student
    ADD CONSTRAINT login_student_fk1 FOREIGN KEY (id_student) REFERENCES public.student(id);


--
-- Name: login_teacher login_teacher_fk1; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.login_teacher
    ADD CONSTRAINT login_teacher_fk1 FOREIGN KEY (id_plan_record) REFERENCES public.academic_plan(id) NOT VALID;


--
-- Name: student_card student_card_fk0; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.student_card
    ADD CONSTRAINT student_card_fk0 FOREIGN KEY (id_student) REFERENCES public.student(id);


--
-- Name: student_card student_card_fk1; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.student_card
    ADD CONSTRAINT student_card_fk1 FOREIGN KEY (id_plan) REFERENCES public.academic_plan(id);


--
-- Name: student student_fk1; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.student
    ADD CONSTRAINT student_fk1 FOREIGN KEY (id_group) REFERENCES public."group"(id);


--
-- PostgreSQL database dump complete
--

