**Project for academic purpose**

## depot
Cloud-native log collector

## Description
Goal: Collecting log from various source (system log, server log, common http message..)

![depot-diagram drawio](https://github.com/user-attachments/assets/9886f26d-b959-495a-a20d-d8502ddedf75)


### Support data
- File
- HTTP log
- Common raw message


## Getting Started

### Prerequisites

- Rust programming language
- Cargo (Rust package manager)

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
$ cargo run --bin depot-client  -- runner --config-file <config-file-path>
...
--- Read new lines of log file: <target-file-path> ---
...
```

## Configuration

### client
| Name | Description | Type | Note |
|----------|----------|----------|----------|----------|
| Row 1    | Data 1   | Data 2   | Data 3   | Data 4   |
| Row 2    | Data 5   | Data 6   | Data 7   | Data 8   |

### server
| Name | Description | Type | Note |
|----------|----------|----------|----------|----------|
| Row 1    | Data 1   | Data 2   | Data 3   | Data 4   |
| Row 2    | Data 5   | Data 6   | Data 7   | Data 8   |

## Project Structure

- `/depot-server`: Contains the server application
- `/depot-client`: Contains the client command
- `/depot-common`: Contains common features which are being used by server and client

### client
- Capture file updates, http log, or else
- Send captured result to server, in raw format
- Put tag on each types, to make server know how to handle it

### server
- Receive messages from client
- Based on tag, parse & filter messages and re-send to other systems (Kafka, Opensearch, or else)
