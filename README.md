# Callback Tester

Callback Tester is a tool designed to help testers test backend callback interfaces. It provides a simple HTTP server that allows users to send callback requests and view the latest request information. This tool is especially useful for testers who do not have a backend server or an available backend to use.

## Features

- **Event Stream**: Receive callback events in real-time via SSE (Server-Sent Events).
- **Callback Interface**: Receive and store callback request information.
- **Latest Request**: Retrieve the latest callback request information.

## Installation

Make sure you have [Rust](https://www.rust-lang.org/) and [Cargo](https://doc.rust-lang.org/cargo/) installed. Then clone this repository and build the project:

```bash
git clone https://github.com/zibo-chen/Callback-Tester
cd callback-tester
cargo build --release
```

## Running

After building, run the following command to start the server:

```bash
cargo run --release
```

By default, the server will start on `0.0.0.0:18686`. You can customize the host and port using command-line arguments:

```bash
cargo run --release -- --host <host> --port <port>
```

For example, to run the server on `127.0.0.1:8080`:

```bash
cargo run --release -- --host 127.0.0.1 --port 8080
```

## API Endpoints

### 1. Get Event Stream

**Endpoint**: `GET /events/{identification}`

**Description**: Receive callback events in real-time via SSE (Server-Sent Events).

**Note**: You can access this endpoint using the latest version of Google Chrome or Edge browser.

**Example**:

```bash
curl -N http://127.0.0.1:18686/events/test-id
```

### 2. Send Callback Request

**Endpoint**: `POST /callback/{identification}`

**Description**: Send a callback request and store the request information.

**Example**:

```bash
curl -X POST http://127.0.0.1:18686/callback/test-id -d "This is a test body"
```

### 3. Get Latest Request Information

**Endpoint**: `GET /latest/{identification}`

**Description**: Retrieve the latest callback request information.

**Example**:

```bash
curl http://127.0.0.1:18686/latest/test-id
```

## Example

Here is a complete usage example:

1. Start the server:

    ```bash
    cargo run --release
    ```

2. Subscribe to the event stream in one terminal:

    ```bash
    curl -N http://127.0.0.1:18686/events/test-id
    ```

3. Send a callback request in another terminal:

    ```bash
    curl -X POST http://127.0.0.1:18686/callback/test-id -d "This is a test body"
    ```

4. Retrieve the latest request information:

    ```bash
    curl http://127.0.0.1:18686/latest/test-id
    ```

## Contribution

Contributions are welcome! Please fork this repository and submit a PR.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
