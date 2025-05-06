# DeployCli
## Binary Release
### Sever
Automated install/update  (don't forget to always verify what you're piping into bash):

```sh
curl -L https://github.com/rust-kotlin/deploycli/raw/master/install_server.sh | bash
```
The script installs downloaded binary to `/root/deploycli` directory by default, but it can be changed by setting `DIR` environment variable.

### Client

```sh
curl -L https://github.com/rust-kotlin/deploycli/raw/master/install_client.sh | bash
```
The script installs downloaded binary to `/usr/local/bin` directory by default, but it can be changed by setting `DIR` environment variable.
