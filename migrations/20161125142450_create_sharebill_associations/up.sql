CREATE TABLE sharebill_associations (
    slack_name TEXT PRIMARY KEY NOT NULL,
    sharebill_account TEXT NOT NULL
);

CREATE UNIQUE INDEX sharebill_associations_slack_name
    ON sharebill_associations (slack_name);
