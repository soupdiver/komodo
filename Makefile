.PHONY: setup backend frontend

setup:
	cargo install cargo-watch
	cd client/core/ts && yarn && yarn build && yarn link
	cd frontend && yarn link komodo_client && yarn

backend:
	docker compose -f dev.compose.yaml up -d ferretdb
	KOMODO_HOST=http://localhost:9120 \
	KOMODO_DATABASE_ADDRESS=localhost:27017 \
	KOMODO_ENABLE_NEW_USERS=true \
	KOMODO_LOCAL_AUTH=true \
	KOMODO_JWT_SECRET=a_random_secret \
	KOMODO_FRONTEND_PATH=./frontend/dist \
	cargo watch -x 'run --release -p komodo_core'

frontend:
	cd frontend && yarn build --watch
