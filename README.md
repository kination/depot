**Project for academic purpose**

## depot
Cloud-native log collector

## Description
Collecting log from various source (system log, server log, common http message..)

## Getting Started

### Prerequisites

- Rust programming language
- Cargo (Rust's package manager)

### Running the Application

1. Start the server:
```
$ cd path/to/project/root
$ cargo run --bin depot-server
...
--- Server started in ... ---
```

2. Open a new terminal window and run writer client to send a message:
// ...TODO


## Project Structure

- `/depot-server`: Contains the server application
- `/depot-client`: Contains the client command
- `/depot-common`: Contains common features which are being used by server and client
