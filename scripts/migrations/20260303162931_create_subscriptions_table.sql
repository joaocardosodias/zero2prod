CREATE TABLE subscriptions (
    id uuid NOT NULL,
    primary key (id),
    email text NOT NULL UNIQUE,
    name text NOT NULL,
    subscribed_at timestamptz NOT NULL
);