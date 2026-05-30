-- Page documents stored in SurrealDB (multimodel). Holds rendered content —
-- markdown, html, or a js page — addressed by slug.

DEFINE TABLE page SCHEMAFULL;
DEFINE FIELD slug       ON page TYPE string;
DEFINE FIELD title      ON page TYPE string;
DEFINE FIELD format     ON page TYPE string ASSERT $value IN ["markdown", "html", "js"];
DEFINE FIELD content    ON page TYPE string;
DEFINE FIELD updated_at ON page TYPE datetime DEFAULT time::now();
DEFINE INDEX page_slug  ON page FIELDS slug UNIQUE;
