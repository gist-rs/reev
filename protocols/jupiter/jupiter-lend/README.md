# Jupiter Lend API Server

## Introduction

This project provides a standalone server that acts as a transaction builder for the Jupiter Lend API. It's designed to be run as a separate service, exposing a simple REST endpoint. This approach was chosen to avoid introducing numerous dependencies and potential conflicts into the main workspace.

The server takes a lending request, communicates with the official Jupiter API to get a quote and construct the transaction, and then returns the serialized transaction to the client.

## Getting Started

### Prerequisites

- Rust and Cargo (see `rust-toolchain.toml` for the recommended version)

### Running the Server

1.  Navigate to the project directory:
    ```sh
    cd protocols/jupiter/jupiter-lend
    ```

2.  Run the server:
    ```sh
    cargo run
    ```

By default, the server will start and listen on `127.0.0.1:3000`.

### Configuration

You can configure the base URL for the Jupiter Lend API by setting the `API_BASE_URL` environment variable. If not set, it defaults to a known Jupiter endpoint.

```sh
API_BASE_URL=https://your-custom-jup-api.com/v1 cargo run
```

## API Usage

The server exposes a single endpoint to build a lending transaction.

### `POST /build-lend-transaction`

This endpoint fetches a quote from Jupiter's API and constructs a lending transaction based on the provided parameters.

**Request Body:**

```json
{
  "userPublicKey": "USER_WALLET_ADDRESS",
  "inputMint": "TOKEN_MINT_TO_LEND",
  "amount": 1000000
}
```

-   `userPublicKey` (string): The base58-encoded public key of the user initiating the transaction.
-   `inputMint` (string): The base58-encoded mint address of the token to be lent.
-   `amount` (number): The amount of the token to lend, in its smallest denomination (lamports).

**Example Request with `curl`:**

```sh
curl -X POST http://127.0.0.1:3000/build-lend-transaction \
-H "Content-Type: application/json" \
-d '{
  "userPublicKey": "2AQdpHJ2JpcEgPiATUXjQxA8QmafFegfQwSLWSprPicm",
  "inputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "amount": 1000000
}'
```

**Success Response (200 OK):**

The response contains the base64-encoded, serialized `VersionedTransaction` and the last valid block height.

```json
{
  "lendTransaction": "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA...==",
  "lastValidBlockHeight": 254896321
}
```

**Error Response:**

If an error occurs (e.g., invalid input, API failure), the server will return a non-200 status code with a JSON body describing the error.

```json
{
  "error": "Failed to get quote: Request failed with status 400 Bad Request: Invalid input mint"
}
```
