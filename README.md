# Framework-e-learning-platform

## Installation

Requires Diesel CLI installed with "postgres".

```sh
diesel migration run
cargo run
```

(if the programm does not keep open there is a known diesel bug: https://github.com/diesel-rs/diesel/discussions/2947)

(set USE_SEEDER="true" on first start, to create DB values)
src/assets/task_schemas -> example schemas
src/assets/task -> example tasks

## Environment variables

```sh
DATABASE_URL="postgres://postgres:password@localhost/ur_db"
MONGODB_DATABASE_URL="mongodb://localhost:27017"
MONGODB_DATABASE_NAME="db_name"
JWT_SECRET="secret_key"
USE_SEEDER="false"
```

## Missing Features

- Endpoints for creating and managaging roles
- Endpoints for deleting some structurs 
- Unittest for statistic endpoint
- Unittest for deleting endpoints
- Cronjob for deleting database entries

## License

MIT

