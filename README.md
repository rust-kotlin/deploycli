# DeployCli
A tool to simplify the process of my new server init configuration.

Since when I move to a new VPS, some init process of apps seems boring and the process to upload and modify the config files manually is painfull.
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

## Server use
The script will make a directory and put the binary file in `/root/deploycli`. Touch a new `config.toml` file in the directory and follow the `config.toml` schema in this repo. You can simply copy it and edit the server port and password for client to connect to. Reverse proxy with https enabled is suggested since the password and package file isn't encrypted during the network transportion.

## Client Use
You must edit the config created by the client cli after your first use. It is in the `/etc/deploycli/config.toml`.
```bash
CLI client for task management

Usage: client <COMMAND>

Commands:
  new     Create a new task
  get     Get tasks or a specific task by index
  post    Upload a task
  delete  Delete a task
  update  Update Database Index
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## TODOS
- [ ] Add Run.sh preview before run. (In fact every user must deploy his own server and ensure the safety of package by himself.)
- [ ] Encrypt the password
- [ ] More to do...
