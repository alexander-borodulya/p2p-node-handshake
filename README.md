# Bitcoin Peer-to-Peer Node Handshake (p2p-node-handshake)

# 1. Introduction

`p2p-node-handshake` is an application that implements basic peer-to-peer Bitcoin handshake protocol
between local and remote nodes.

According to the [Bitcoin p2p network specification](https://developer.bitcoin.org/devguide/p2p_network.html):

> When started for the first time, programs don’t know the IP addresses of any active full nodes. In order to discover some IP addresses, they query one or more DNS names (called DNS seeds) hardcoded into Bitcoin Core and BitcoinJ. The response to the lookup should include one or more DNS A records with the IP addresses of full nodes that may accept new incoming connections.

The application should proceed through several steps to achieve the handshake:

    1. Pick the DNS Seed address;
    2. Pick the IP address of the active Bitcoin node;
    3. Run the application with the target IP node address.

After the handshake process is finished the terminal output displays the status of the handshake.



# 2. Implementation Details

`p2p-node-handshake` consists of two crates: __*binary*__ and __*library*__.

The __library__ crate contains all the implementation details of the protocol handshake and exposes only a few public methods.

__Binary__ crate accepts user input from the user in a command line way, instantiates, and runs config instance that performs handshake asynchronously.

## Config
In the first step, the application creates an instance of `Config` struct that parses command line arguments.
No well-known crates are used to implement argument parsing, like clap or structopt in the current implementation.
The `Config` struct provides minimal implementation to quickly fetch user input and parse arguments.

## DnsSeedManager
The application provides `DnsSeedManager`. The instance of `DnsSeedManager` incapsulates default DNS URL addresses
that might be used to discover active node IP addresses.
According to the user input, the `DnsSeedManager` picks the DNS seed address that will be used to discover active node IP addresses. 
Also, it can print all available DNS seed addresses, so, the user can choose which DNS seed he wishes to use for the discover step.

Shortly, the output of the `DnsSeedManager` instance can be interpreted as an input for the HandshakeManager instance.

## HandshakeManager
The `HandshakeManager` provides functionality that performs handshake namely.

The primary function - `establish_handshake` - accepts the already resolved IP node address and tries to exchange 
messages in order how the documentation explains.

Basically, the handshake process is a [message exchange](https://en.bitcoin.it/wiki/Version_Handshake) events that might be visualized by the following lines:

```
    L -> R: Send version message with the local peer's version
    R -> L: Send version message back
    R -> L: Send verack message
    R:      Sets version to the minimum of the 2 versions
    L -> R: Send verack message after receiving version message from R
    L:      Sets version to the minimum of the 2 versions
```

The `network_messages.rs` file contains functions that build messages:

    - new_version_message_serialised
    - make_verack_message_serialised

The return values of that function would be used by `HandshakeManager` to send messages and receive responses from the remote node.

The `establish_handshake` function will report how successful or not the handshake message exchange was.

For the error handling functionality was used `error-stack` crate, which is slightly more verbose in the 
term of writing line numbers comparing to `thiserror` or `anyshow`. But, `error-stack` crate allows to visualize the error that has occurred in a hierarchical form, which will allow to quickly understand the root cause of the error.



# 3. Build Details

Build command: `cargo build`, or `cargo build --release` for the release version.

Run command: 
    
    cargo run -- <ARGUMENTS>



# 4. CLI Arguments

Supported arguments:

`-l` - Prints a list of available DNS resolvers;
    
Example output:
```    
    > cargo run -- -l

    0 - https://dns-resolver-url-0.com
    1 - https://dns-resolver-url-1.com
    2 - https://dns-resolver-url-2.com
```

`-r <DNS URL>` - Resolves remote peer URLs by specified DNS Seed URL;

`-hbu <REMOTE PEER URL>` - Performs a handshake with a specified node URL;

```
    > cargo run -- -r <DNS URL>
```

`-hbi <DNS URL INDEX> <REMOTE PEER URL INDEX>` - Performs a handshake with a remote peer by specified URL indexes. 

The `<DNS URL INDEX>` index corresponds to the URL index in the list of DNS Seed URLs.

The `<REMOTE PEER URL INDEX>` index corresponds to the URL index in the list of resolved active node URLs.

A list of resolved URLs can be obtained by running: `cargo run -- -r <DNS URL>`



# 5. Output Examples

## Print available DNS Seed URLs:

Command: 
    
    `cargo run -- -l`

Output:

```
[2023-02-13T02:01:56Z INFO  p2p_node_handshake::config] DNS Resolvers:
0: seed.bitcoin.sipa.be.
1: dnsseed.bluematt.me.
2: dnsseed.bitcoin.dashjr.org.
3: seed.bitcoinstats.com.
4: seed.bitcoin.jonasschnelli.ch.
5: seed.btc.petertodd.org.
6: seed.bitcoin.sprovoost.nl.
7: dnsseed.emzy.de.
8: seed.bitcoin.wiz.biz.
```

## Resolve node IP addresses by DNS Seed URL:

Command: 
    
    `cargo run -- -r 0`

Output:

```
[2023-02-13T02:03:43Z INFO  p2p_node_handshake::config] Active IP node URLs:
0: 139.59.92.87:8333
1: 23.138.176.127:8333
2: 87.244.68.246:8333
...
36: [2001:67c:26b4:ff00::44]:8333
37: [2001:470:88ff:2e::1]:8333
38: [2a01:4f8:251:2fe6::2]:8333
```

## Run handshake with node IP addresse:

Command: 
    
    `cargo run -- -hbu 87.244.68.246:8333`

Output:
```
[2023-02-13T02:05:25Z INFO  p2p_node_handshake::config] Handshake by IP URL...
[2023-02-13T02:05:25Z INFO  p2p_node_handshake::handshake_manager] Send version message 70015 to 87.244.68.246:8333
[2023-02-13T02:05:25Z INFO  p2p_node_handshake::handshake_manager] Recv version message 70015 from 87.244.68.246:8333
[2023-02-13T02:05:25Z INFO  p2p_node_handshake::handshake_manager] Sent VerAck message to 87.244.68.246:8333
[2023-02-13T02:05:25Z INFO  p2p_node_handshake::handshake_manager] Recv VerAck message from 87.244.68.246:8333: Verack
[2023-02-13T02:05:25Z INFO  p2p_node_handshake::config] handshake completed successfully with node: 87.244.68.246:8333
```

## Run handshake with DNS Seed and node IP indexes:

Command: 
    
    `cargo run -- -hbi 3 5`

Output:
```
[2023-02-13T02:06:48Z INFO  p2p_node_handshake::config] Handshake by DNS seed and IP indexes...
[2023-02-13T02:06:49Z INFO  p2p_node_handshake::handshake_manager] Send version message 70015 to 24.17.248.42:8333
[2023-02-13T02:06:49Z INFO  p2p_node_handshake::handshake_manager] Recv version message 70016 from 24.17.248.42:8333
[2023-02-13T02:06:49Z INFO  p2p_node_handshake::handshake_manager] Sent VerAck message to 24.17.248.42:8333
[2023-02-13T02:06:49Z INFO  p2p_node_handshake::handshake_manager] Recv VerAck message from 24.17.248.42:8333: Verack
[2023-02-13T02:06:49Z INFO  p2p_node_handshake::config] Handshake with IP 24.17.248.42:8333 evaluated from DNS seed index 3 and IP index 5, completed
```

## Run handshake with initially invalid IP address:

Command: 
    
    `cargo run -- -hbu 12.34.56.26:8333`

Output:

```
[2023-02-13T02:09:47Z INFO  p2p_node_handshake::config] Handshake by IP URL...
Handshake with remote peer 12.34.56.26:8333 failed with error: 
Hhandshake error
├╴at /home/alexander/github/p2p-node-handshake/src/handshake_manager.rs:135:14
├╴Handshake timed out after 2000ms
│
╰─▶ deadline has elapsed
    ╰╴at /home/alexander/github/p2p-node-handshake/src/handshake_manager.rs:134:14
```
