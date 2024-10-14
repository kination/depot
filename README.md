**Project for academic purpose**

## depot
Goal: depot is cloud-native message queue system

## Description
This is based on 2 main components
- depot-server: server application which receive/send message from/to client
- depot-server: client application to read/write data

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
```
$ cd path/to/project/root
$ cargo run --bin depot-client -- write
...
--- Write client started... ---
```

3. Open a new terminal window and run reader client to read  message from server queue:
```
$ cd path/to/project/root
$ cargo run --bin depot-client -- read
...
--- Write client started... ---
```

## Project Structure

- `/depot-server`: Contains the server application
- `/depot-client`: Contains the client command
- `/depot-common`: Contains common features which are being used by server and client
