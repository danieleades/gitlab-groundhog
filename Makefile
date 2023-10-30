

.PHONY: update-schema
update-schema:
    # cargo install graphql_client_cli
	graphql-client introspect-schema https://gitlab.seebyte.com/api/graphql --output gitlab-schema.json 