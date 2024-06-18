CREATE EXTENSION if not exists pg_uuidv7;

-- "Admins", users who are able to change data related to the website
CREATE TABLE IF NOT EXISTS users
(
    username   Text      NOT NULL PRIMARY KEY,
    pass       Text      NOT NULL,
    token      Text      NOT NULL,
    tok_expire Timestamp NOT NULL DEFAULT NOW() + INTERVAL '7 days'
);

-- Users can create other user accounts
-- On install there will be a default user with a generated password that will be printed to console

CREATE index on users using hash (token);
CREATE index on users using hash (username);

CREATE TABLE IF NOT EXISTS faculties
(
    id   SERIAL NOT NULL PRIMARY KEY,
    name Text   NOT NULL
);

CREATE TABLE IF NOT EXISTS students
(
    nr_mat     Text       NOT NULL PRIMARY KEY,
    email      Text       NOT NULL UNIQUE,
    cnp3       varchar(3) NOT NULL,
    faculty    SERIAL     NOT NULL references faculties (id) ON DELETE CASCADE,
    token      Text       NOT NULL,
    tok_expire Timestamp  NOT NULL DEFAULT NOW() + INTERVAL '7 days'
);

DO
$$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'semester') THEN
            CREATE TYPE Semester AS ENUM ('First', 'Second');
        END IF;
    END
$$;

CREATE TABLE IF NOT EXISTS classes
(
    id           SERIAL   NOT NULL PRIMARY KEY,
    name         Text     NOT NULL,
    descr        Text     NOT NULL,
    faculty      SERIAL   NOT NULL references faculties (id) ON DELETE CASCADE,
    semester     Semester NOT NULL,
    disabled     boolean  NOT NULL default false,
    requirements Text,
    prof         Text     NOT NULL
);

CREATE TABLE IF NOT EXISTS classes_files
(
    id         SERIAL NOT NULL,
    name       Text   NOT NULL,
    classes_id SERIAL NOT NULL references classes (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS choices
(
    nr_mat        Text references students (nr_mat) NOT NULL PRIMARY KEY,
    first_choice  Serial                            NOT NULL references classes (id) ON DELETE CASCADE,
    second_choice Serial                            NOT NULL references classes (id) ON DELETE CASCADE,
    created       timestamp                         NOT NULL DEFAULT NOW(),
    updated       timestamp                                  DEFAULT NULL,
    CHECK (first_choice <> second_choice) -- Checks if they are different
);

-- Used to store past choices so a student wont be able to pick a class he already attended in the previous years
CREATE TABLE IF NOT EXISTS old_choices
(
    id     UUID   NOT NULL DEFAULT uuid_generate_v7() PRIMARY KEY,
    nr_mat Text   NOT NULL references students (nr_mat) ON DELETE CASCADE,
    choice Serial NOT NULL references classes (id) ON DELETE CASCADE
);


-- Timestampz to encode the timezone into the type
CREATE TABLE IF NOT EXISTS choices_open_date
(
    id int NOT NULL DEFAULT 0 PRIMARY KEY,
    start_date timestamptz NOT NULL,
    end_date timestamptz NOT NULL
);


CREATE index on old_choices using hash (nr_mat);
