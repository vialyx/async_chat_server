# Async TCP Chat Server with TLS (Rust + Tokio + rustls)

This project is a **mini async chat server** built with Rust.  
It uses:

- [Tokio](https://tokio.rs/) for async runtime  
- [tokio-stream](https://docs.rs/tokio-stream) for stream handling  
- [rustls](https://github.com/rustls/rustls) for TLS support  
- [async-trait](https://docs.rs/async-trait) to simplify async trait usage  

The server allows multiple clients to connect via TCP over TLS and broadcast messages to each other in real time.

---

## Features
- ✅ Async TCP server with Tokio  
- ✅ TLS encryption with rustls  
- ✅ Multi-client broadcast using `tokio::sync::broadcast`  
- ✅ Example self-signed certificate for local testing  

---

## Getting Started

### 1. Clone the repo
```bash
git clone https://github.com/your-username/async-tcp-chat-tls.git
cd async-tcp-chat-tls
```

### 2. Generate TLS certificates
For testing, you can generate a self-signed certificate:

```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes   -subj "/CN=localhost"
```

This will create:
- `cert.pem` → certificate  
- `key.pem` → private key  

Place them in the project root.

---

### 3. Run the server
```bash
cargo run
```

By default, the server listens on `127.0.0.1:8080`.

---

### 4. Connect with a TLS client
You can use **openssl s_client** for testing:

```bash
openssl s_client -connect 127.0.0.1:8080 -CAfile cert.pem
```

Type messages and see them broadcast to all connected clients.

---

## Project Structure
```
.
├── Cargo.toml
├── src
│   └── main.rs
├── cert.pem
├── key.pem
└── README.md
```

---

## Example Session
- Start the server  
- Connect with 2+ clients (`openssl s_client ...`)  
- Send messages from one client → all other clients receive it  

---

## Next Steps / Improvements
- Add authentication (e.g., username/password or client certs)  
- Support private messages  
- Add async-std version for comparison  
- Create a simple Rust-based CLI client  

---

## License
MIT License. Use freely for learning and projects.
