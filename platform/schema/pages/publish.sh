#!/usr/bin/env bash
# Publish the logic-reference page into SurrealDB via its HTTP API.
# Run from a host that can reach the endpoint (this CI sandbox is allowlist-blocked).
#
#   SURREAL_URL=https://schemadb-06ehsj292ppah8kbsk9pmnjjbc.aws-aps1.surreal.cloud \
#   SURREAL_NS=agentbench SURREAL_DB=main \
#   SURREAL_USER=root SURREAL_PASS=*** \
#   ./publish.sh
set -euo pipefail

URL="${SURREAL_URL:?set SURREAL_URL}"
NS="${SURREAL_NS:?set SURREAL_NS}"
DB="${SURREAL_DB:?set SURREAL_DB}"
DIR="$(cd "$(dirname "$0")" && pwd)"

auth=(-u "${SURREAL_USER:-root}:${SURREAL_PASS:?set SURREAL_PASS}")
hdr=(-H "Surreal-NS: $NS" -H "Surreal-DB: $DB" -H "Accept: application/json")

# 1) Define the page table (schema).
curl -fsS "${auth[@]}" "${hdr[@]}" -H "Content-Type: text/plain" \
  --data-binary @"$DIR/../pages.surql" "$URL/sql" >/dev/null
echo "schema defined"

# 2) Upsert the page document (content bound as a JSON variable, no escaping pain).
content_json=$(python3 -c 'import json,sys;print(json.dumps(open(sys.argv[1]).read()))' \
  "$DIR/agent-bench-logic-reference.md")
curl -fsS "${auth[@]}" "${hdr[@]}" -H "Content-Type: application/json" \
  --data-binary "{
    \"query\": \"UPSERT type::thing('page',\$slug) SET slug=\$slug, title=\$title, format='markdown', content=\$content\",
    \"vars\": { \"slug\": \"agent-bench-logic-reference\",
               \"title\": \"Agent-Bench — Logic Reference\",
               \"content\": $content_json }
  }" "$URL/sql" >/dev/null
echo "page upserted"

# 3) The retrieval link (SurrealDB HTTP REST API):
echo
echo "Read it back via the SurrealDB API:"
echo "  GET $URL/key/page/agent-bench-logic-reference"
echo "      -H 'Surreal-NS: $NS' -H 'Surreal-DB: $DB' -u <user>:<pass>"
echo
echo "Or via SQL:"
echo "  POST $URL/sql   body: SELECT * FROM page:\`agent-bench-logic-reference\`"
