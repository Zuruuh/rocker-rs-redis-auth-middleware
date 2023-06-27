small rust & redis project with (really) simple auth manager 

*setup*
```bash
docker compose up -d
cargo run
```

rocket should boot up the server on http://localhost:8000  
Make a first request to the `POST /register` endpoint, which should return a token (was too lazy to setup serde & json serializer)  
Then, head to `GET /private`, and add your token to the `Authorization` header, with the `Bearer ` prefix. 
You should only be able to get your user id if you are using a valid token 👍
