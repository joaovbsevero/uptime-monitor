# uptimer-monitor

This is part of a programming challenge to develop an uptime monitor that enable users to monitor URLs for up-time and response time and provide alerts if there is an issue.

## Features

- Timed requests: Set a schedule for every check
- Body Validation: Validate the response body with a expected body
- Webhook: Send a post request to a URL when a check fails or recovers with the check information and details.
- History: Store history of checks for later retrieval and analysis

## Configuration

The configuration is done via a `.env` file. Below is an example of the configuration options.

```dotenv
# Address to run the server on
ADDRESS=0.0.0.0

# Port to run the server on
PORT=8080

# Level of logging to use (trace, debug, info, warn, error)
LOG_LEVEL=info

# Indicates which database to use
DB_URI=mongodb://localhost:27017

# Specifies the database name to use
DB_NAME=uptime-monitor
```

## Running

To run the application, you can rely on docker compose to setup the environment for you. Just run: `docker compose up api`

It will spin up a MongoDB instance and an API instance.